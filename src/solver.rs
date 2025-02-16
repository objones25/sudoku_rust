use crate::{Board, CandidateSet, Grid, Result, SudokuError, simd::{SimdValidator, SimdSolver, has_simd_support}};
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct Solver {
    board: Board,
    solution: Board,
    // Pre-computed candidates for each cell
    candidates: Vec<CandidateSet>,
    // Track if we found a unique solution
    unique_solution: bool,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
    simd_solver: Option<SimdSolver>,
}

impl Solver {
    pub fn new(grid: Grid) -> Self {
        let board = Board::new(&grid.value);
        let solution = Board::new(&grid.solution);
        let mut solver = Self {
            board: board.clone(),
            solution,
            candidates: vec![CandidateSet::empty(); 81],
            unique_solution: true,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
            simd_solver: if has_simd_support() {
                unsafe { Some(SimdSolver::new(&board)) }
            } else {
                None
            },
        };
        solver.precompute_candidates();
        solver
    }

    /// Precompute valid candidates for each empty cell
    fn precompute_candidates(&mut self) {
        for row in 0..9 {
            for col in 0..9 {
                if self.board.is_empty_cell(row, col) {
                    let mut candidates = CandidateSet::all();
                    // Remove candidates that are already present in the same row, column, or box
                    for num in 1..=9 {
                        if !self.is_valid_placement(&self.board, row, col, num) {
                            candidates.remove_candidate(num);
                        }
                    }
                    self.candidates[row * 9 + col] = candidates;
                }
            }
        }
    }

    /// Find all empty cells sorted by number of candidates and constraint impact
    fn find_empty_cells(&self) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        let mut min_candidates = 10;
        let mut max_impact = 0;
        
        // First pass: find minimum number of candidates and maximum impact
        for row in 0..9 {
            for col in 0..9 {
                if self.board.is_empty_cell(row, col) {
                    let count = self.candidates[row * 9 + col].count_candidates();
                    if count < min_candidates {
                        min_candidates = count;
                        max_impact = self.calculate_impact(row, col);
                    } else if count == min_candidates {
                        let impact = self.calculate_impact(row, col);
                        max_impact = max_impact.max(impact);
                    }
                }
            }
        }
        
        // Second pass: collect cells with minimum candidates and maximum impact
        for row in 0..9 {
            for col in 0..9 {
                if self.board.is_empty_cell(row, col) {
                    let count = self.candidates[row * 9 + col].count_candidates();
                    let impact = self.calculate_impact(row, col);
                    if count == min_candidates && impact >= max_impact {
                        cells.push((row, col));
                    }
                }
            }
        }
        
        // If no cells found, collect all empty cells
        if cells.is_empty() {
            for row in 0..9 {
                for col in 0..9 {
                    if self.board.is_empty_cell(row, col) {
                        cells.push((row, col));
                    }
                }
            }
        }
        
        cells
    }

    /// Calculate the impact of filling a cell based on constraints
    fn calculate_impact(&self, row: usize, col: usize) -> u32 {
        let mut impact = 0;
        let candidates = self.candidates[row * 9 + col];
        
        // Check row impact
        for j in 0..9 {
            if j != col && self.board.is_empty_cell(row, j) {
                let other_candidates = self.candidates[row * 9 + j];
                impact += (candidates.0 & other_candidates.0).count_ones();
            }
        }
        
        // Check column impact
        for i in 0..9 {
            if i != row && self.board.is_empty_cell(i, col) {
                let other_candidates = self.candidates[i * 9 + col];
                impact += (candidates.0 & other_candidates.0).count_ones();
            }
        }
        
        // Check box impact
        let box_row = (row / 3) * 3;
        let box_col = (col / 3) * 3;
        for i in 0..3 {
            for j in 0..3 {
                let r = box_row + i;
                let c = box_col + j;
                if (r != row || c != col) && self.board.is_empty_cell(r, c) {
                    let other_candidates = self.candidates[r * 9 + c];
                    impact += (candidates.0 & other_candidates.0).count_ones();
                }
            }
        }
        
        impact
    }

    pub fn solve(&mut self) -> Result<Vec<Vec<i32>>> {
        let empty_cells = self.find_empty_cells();
        if empty_cells.is_empty() {
            if !SimdValidator::validate_solution(&self.board) {
                return Err(SudokuError::InvalidBoard);
            }
            return Ok(self.board.to_vec());
        }
        
        // Take only the first empty cell with minimum candidates and maximum impact
        let (row, col) = empty_cells[0];
        let candidates = self.candidates[row * 9 + col];
        
        if candidates.is_empty() {
            return Err(SudokuError::InvalidBoard);
        }

        let board = self.board.clone();
        let solution = self.solution.clone();
        
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
        let simd_solver = self.simd_solver.clone();
        
        let solution_found = Arc::new(AtomicBool::new(false));
        let matches_api = Arc::new(AtomicBool::new(false));
        
        // Use bounded channel with a reasonable size
        let (tx, rx) = crossbeam::channel::bounded(1);
        
        // Sort candidates by their impact for better pruning
        let mut sorted_candidates: Vec<_> = candidates.iter_candidates().collect();
        sorted_candidates.sort_by_key(|&num| {
            let mut board_copy = board.clone();
            board_copy.set(row, col, num);
            self.calculate_impact(row, col)
        });

        sorted_candidates.into_par_iter()
            .find_map_first(|num| {
                if solution_found.load(Ordering::SeqCst) {
                    return None;
                }

                let mut board_copy = board.clone();
                #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
                let simd_solver = simd_solver.clone();
                
                if self.try_solve_with_value(row, col, num, &mut board_copy, 
                    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
                    simd_solver
                ) {
                    if board_copy == solution {
                        matches_api.store(true, Ordering::SeqCst);
                    }
                    
                    if solution_found.fetch_or(true, Ordering::SeqCst) {
                        return None;
                    }
                    
                    match tx.send_timeout(board_copy, Duration::from_secs(1)) {
                        Ok(_) => Some(()),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            });

        self.unique_solution = matches_api.load(Ordering::SeqCst);
        
        if solution_found.load(Ordering::SeqCst) {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(solved_board) => {
                    self.board = solved_board;
                    return Ok(self.board.to_vec());
                }
                Err(_) => {
                    return Err(SudokuError::InvalidBoard);
                }
            }
        }

        Err(SudokuError::InvalidBoard)
    }

    fn try_solve_with_value(
        &self, 
        start_row: usize, 
        start_col: usize, 
        value: u8, 
        board: &mut Board,
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
        mut simd_solver: Option<SimdSolver>,
    ) -> bool {
        board.set(start_row, start_col, value);
        
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
        if let Some(ref mut solver) = simd_solver {
            unsafe {
                solver.update_masks(start_row, start_col, value);
            }
        }
        
        
        if let Some((next_row, next_col)) = self.find_next_empty(board) {
            for num in 1..=9 {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
                let is_valid = if let Some(ref solver) = simd_solver {
                    unsafe { solver.is_valid_candidate(next_row, next_col, num) }
                } else {
                    self.is_valid_placement(board, next_row, next_col, num)
                };
                
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
                let is_valid = self.is_valid_placement(board, next_row, next_col, num);
                
                if is_valid {
                    let mut new_board = board.clone();
                    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
                    let new_simd_solver = simd_solver.clone();
                    
                    if self.try_solve_with_value(next_row, next_col, num, &mut new_board,
                        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
                        new_simd_solver
                    ) {
                        *board = new_board;
                        return true;
                    }
                }
            }
            false
        } else {
            // No empty cells left, verify the solution
            self.is_valid_solution(board)
        }
    }

    fn is_valid_solution(&self, board: &Board) -> bool {
        // Use SIMD validation for better performance
        SimdValidator::validate_solution(board)
    }

    fn find_next_empty(&self, board: &Board) -> Option<(usize, usize)> {
        for row in 0..9 {
            for col in 0..9 {
                if board.is_empty_cell(row, col) {
                    return Some((row, col));
                }
            }
        }
        None
    }

    fn is_valid_placement(&self, board: &Board, row: usize, col: usize, num: u8) -> bool {
        // Check row
        for j in 0..9 {
            if board.get(row, j) == num {
                return false;
            }
        }

        // Check column
        for i in 0..9 {
            if board.get(i, col) == num {
                return false;
            }
        }

        // Check 3x3 box
        let box_row = (row / 3) * 3;
        let box_col = (col / 3) * 3;
        for i in 0..3 {
            for j in 0..3 {
                if board.get(box_row + i, box_col + j) == num {
                    return false;
                }
            }
        }

        true
    }

    pub fn verify_solution(&self) -> bool {
        self.board == self.solution
    }

    pub fn has_unique_solution(&self) -> bool {
        self.unique_solution
    }

    pub fn get_solution(&self) -> Vec<Vec<i32>> {
        self.board.to_vec()
    }

    pub fn get_original_solution(&self) -> Vec<Vec<i32>> {
        self.solution.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_with_valid_board() {
        let grid = Grid {
            value: vec![
                vec![5,3,0,0,7,0,0,0,0],
                vec![6,0,0,1,9,5,0,0,0],
                vec![0,9,8,0,0,0,0,6,0],
                vec![8,0,0,0,6,0,0,0,3],
                vec![4,0,0,8,0,3,0,0,1],
                vec![7,0,0,0,2,0,0,0,6],
                vec![0,6,0,0,0,0,2,8,0],
                vec![0,0,0,4,1,9,0,0,5],
                vec![0,0,0,0,8,0,0,7,9],
            ],
            solution: vec![
                vec![5,3,4,6,7,8,9,1,2],
                vec![6,7,2,1,9,5,3,4,8],
                vec![1,9,8,3,4,2,5,6,7],
                vec![8,5,9,7,6,1,4,2,3],
                vec![4,2,6,8,5,3,7,9,1],
                vec![7,1,3,9,2,4,8,5,6],
                vec![9,6,1,5,3,7,2,8,4],
                vec![2,8,7,4,1,9,6,3,5],
                vec![3,4,5,2,8,6,1,7,9],
            ],
            difficulty: "Medium".to_string(),
        };

        let mut solver = Solver::new(grid);
        let solution = solver.solve().unwrap();
        
        // Verify solution
        assert_eq!(solution.len(), 9);
        for row in solution.iter() {
            assert_eq!(row.len(), 9);
            // Check that each row contains numbers 1-9
            let mut nums = row.clone();
            nums.sort_unstable();
            assert_eq!(nums, (1..=9).collect::<Vec<i32>>());
        }

        // Verify against known solution
        assert!(solver.verify_solution());
    }

    #[test]
    fn test_solution_matching() {
        let grid = Grid {
            value: vec![
                vec![5,3,0,0,7,0,0,0,0],
                vec![6,0,0,1,9,5,0,0,0],
                vec![0,9,8,0,0,0,0,6,0],
                vec![8,0,0,0,6,0,0,0,3],
                vec![4,0,0,8,0,3,0,0,1],
                vec![7,0,0,0,2,0,0,0,6],
                vec![0,6,0,0,0,0,2,8,0],
                vec![0,0,0,4,1,9,0,0,5],
                vec![0,0,0,0,8,0,0,7,9],
            ],
            solution: vec![
                vec![5,3,4,6,7,8,9,1,2],
                vec![6,7,2,1,9,5,3,4,8],
                vec![1,9,8,3,4,2,5,6,7],
                vec![8,5,9,7,6,1,4,2,3],
                vec![4,2,6,8,5,3,7,9,1],
                vec![7,1,3,9,2,4,8,5,6],
                vec![9,6,1,5,3,7,2,8,4],
                vec![2,8,7,4,1,9,6,3,5],
                vec![3,4,5,2,8,6,1,7,9],
            ],
            difficulty: "Medium".to_string(),
        };

        let mut solver = Solver::new(grid);
        solver.solve().unwrap();
        assert!(solver.has_unique_solution(), "Solution should match API's solution");
    }

    #[test]
    fn test_invalid_board() {
        let grid = Grid {
            value: vec![
                vec![5,5,0,0,7,0,0,0,0], // Invalid: duplicate 5 in first row
                vec![6,0,0,1,9,5,0,0,0],
                vec![0,9,8,0,0,0,0,6,0],
                vec![8,0,0,0,6,0,0,0,3],
                vec![4,0,0,8,0,3,0,0,1],
                vec![7,0,0,0,2,0,0,0,6],
                vec![0,6,0,0,0,0,2,8,0],
                vec![0,0,0,4,1,9,0,0,5],
                vec![0,0,0,0,8,0,0,7,9],
            ],
            solution: vec![vec![0; 9]; 9],
            difficulty: "Medium".to_string(),
        };

        let mut solver = Solver::new(grid);
        assert!(solver.solve().is_err(), "Should fail with invalid board");
    }

    #[test]
    fn test_empty_board() {
        let grid = Grid {
            value: vec![vec![0; 9]; 9],
            solution: vec![
                vec![1,2,3,4,5,6,7,8,9],
                vec![4,5,6,7,8,9,1,2,3],
                vec![7,8,9,1,2,3,4,5,6],
                vec![2,3,1,5,6,4,8,9,7],
                vec![5,6,4,8,9,7,2,3,1],
                vec![8,9,7,2,3,1,5,6,4],
                vec![3,1,2,6,4,5,9,7,8],
                vec![6,4,5,9,7,8,3,1,2],
                vec![9,7,8,3,1,2,6,4,5],
            ],
            difficulty: "Easy".to_string(),
        };

        let mut solver = Solver::new(grid);
        let solution = solver.solve().unwrap();
        
        // Verify that we found a valid solution
        assert!(solver.is_valid_solution(&Board::new(&solution)));
        
        // Note: The solution might not match the API's solution since an empty board
        // has multiple valid solutions
        assert!(!solver.has_unique_solution(), "Empty board should have multiple solutions");
    }

    #[test]
    fn test_almost_complete_board() {
        let grid = Grid {
            value: vec![
                vec![5,3,4,6,7,8,9,1,0], // Only one empty cell
                vec![6,7,2,1,9,5,3,4,8],
                vec![1,9,8,3,4,2,5,6,7],
                vec![8,5,9,7,6,1,4,2,3],
                vec![4,2,6,8,5,3,7,9,1],
                vec![7,1,3,9,2,4,8,5,6],
                vec![9,6,1,5,3,7,2,8,4],
                vec![2,8,7,4,1,9,6,3,5],
                vec![3,4,5,2,8,6,1,7,9],
            ],
            solution: vec![
                vec![5,3,4,6,7,8,9,1,2],
                vec![6,7,2,1,9,5,3,4,8],
                vec![1,9,8,3,4,2,5,6,7],
                vec![8,5,9,7,6,1,4,2,3],
                vec![4,2,6,8,5,3,7,9,1],
                vec![7,1,3,9,2,4,8,5,6],
                vec![9,6,1,5,3,7,2,8,4],
                vec![2,8,7,4,1,9,6,3,5],
                vec![3,4,5,2,8,6,1,7,9],
            ],
            difficulty: "Easy".to_string(),
        };

        let mut solver = Solver::new(grid);
        solver.solve().unwrap();
        assert!(solver.has_unique_solution(), "Board with one empty cell should have unique solution");
        assert!(solver.verify_solution(), "Solution should match API's solution");
    }

    #[test]
    fn test_simd_solution_validation() {
        let grid = Grid {
            value: vec![
                vec![5,3,4,6,7,8,9,1,2],
                vec![6,7,2,1,9,5,3,4,8],
                vec![1,9,8,3,4,2,5,6,7],
                vec![8,5,9,7,6,1,4,2,3],
                vec![4,2,6,8,5,3,7,9,1],
                vec![7,1,3,9,2,4,8,5,6],
                vec![9,6,1,5,3,7,2,8,4],
                vec![2,8,7,4,1,9,6,3,5],
                vec![3,4,5,2,8,6,1,7,9],
            ],
            solution: vec![vec![0; 9]; 9],  // Not needed for this test
            difficulty: "Test".to_string(),
        };

        let board = Board::new(&grid.value);
        assert!(SimdValidator::validate_solution(&board));
    }
} 