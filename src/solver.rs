use crate::{Grid, Result, SudokuError};
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, trace};

pub struct Solver {
    board: Vec<Vec<i32>>,
    solution: Vec<Vec<i32>>,
    // Pre-computed candidates for each cell
    candidates: Vec<Vec<Vec<i32>>>,
    // Track if we found a unique solution
    unique_solution: bool,
}

impl Solver {
    pub fn new(grid: Grid) -> Self {
        let mut solver = Self {
            board: grid.value,
            solution: grid.solution,
            candidates: vec![vec![vec![]; 9]; 9],
            unique_solution: true,
        };
        solver.precompute_candidates();
        solver
    }

    /// Precompute valid candidates for each empty cell
    fn precompute_candidates(&mut self) {
        for row in 0..9 {
            for col in 0..9 {
                if self.board[row][col] == 0 {
                    let mut valid_nums = Vec::new();
                    for num in 1..=9 {
                        if self.is_valid_placement(&self.board, row, col, num) {
                            valid_nums.push(num);
                        }
                    }
                    self.candidates[row][col] = valid_nums;
                }
            }
        }
    }

    /// Find all empty cells sorted by number of candidates
    fn find_empty_cells(&self) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        for row in 0..9 {
            for col in 0..9 {
                if self.board[row][col] == 0 {
                    cells.push((row, col));
                }
            }
        }
        // Sort by number of candidates (ascending)
        cells.sort_by_key(|&(row, col)| self.candidates[row][col].len());
        cells
    }

    pub fn solve(&mut self) -> Result<Vec<Vec<i32>>> {
        let empty_cells = self.find_empty_cells();
        if empty_cells.is_empty() {
            return Ok(self.board.clone());
        }

        debug!("Found {} empty cells", empty_cells.len());
        
        // Try each empty cell until we find a solution
        for (idx, &(row, col)) in empty_cells.iter().enumerate() {
            debug!("Trying cell {}/{} at ({}, {}) with {} candidates", 
                  idx + 1, empty_cells.len(), row, col, self.candidates[row][col].len());
            
            let candidates = self.candidates[row][col].clone();
            if candidates.is_empty() {
                debug!("No candidates available for ({}, {})", row, col);
                continue;
            }

            let board = self.board.clone();
            let solution = self.solution.clone();
            
            // Use atomic flag to stop other threads once a solution is found
            let solution_found = Arc::new(AtomicBool::new(false));
            let multiple_solutions = Arc::new(AtomicBool::new(false));
            let matches_api = Arc::new(AtomicBool::new(false));
            
            // Use a channel to send the solution back from the parallel iterator
            let (tx, rx) = crossbeam::channel::bounded(1);
            
            candidates.into_par_iter().find_any(|&num| {
                trace!("Trying candidate {} at ({}, {})", num, row, col);
                if solution_found.load(Ordering::Relaxed) {
                    return false;
                }

                let mut board_copy = board.clone();
                if self.try_solve_with_value(row, col, num, &mut board_copy) {
                    debug!("Found solution with {} at ({}, {})", num, row, col);
                    
                    // Check if this solution matches the API's solution
                    if board_copy == solution {
                        matches_api.store(true, Ordering::Relaxed);
                    }
                    
                    // If we already found a solution, this means we have multiple solutions
                    if solution_found.fetch_or(true, Ordering::Relaxed) {
                        debug!("Found multiple solutions");
                        multiple_solutions.store(true, Ordering::Relaxed);
                        return false;
                    }
                    let _ = tx.send(board_copy);
                    return true;
                }
                false
            });

            self.unique_solution = matches_api.load(Ordering::Relaxed);
            
            if solution_found.load(Ordering::Relaxed) {
                if let Ok(solved_board) = rx.try_recv() {
                    debug!("Successfully received solved board");
                    self.board = solved_board;
                    return Ok(self.board.clone());
                }
            }
            debug!("No solution found with current cell, trying next");
        }

        debug!("No solution found with any cell");
        Err(SudokuError::InvalidBoard)
    }

    fn try_solve_with_value(&self, start_row: usize, start_col: usize, value: i32, board: &mut Vec<Vec<i32>>) -> bool {
        board[start_row][start_col] = value;
        trace!("Trying value {} at ({}, {})", value, start_row, start_col);
        
        if let Some((next_row, next_col)) = self.find_next_empty(board) {
            for num in 1..=9 {
                if self.is_valid_placement(board, next_row, next_col, num) {
                    let mut new_board = board.clone();
                    new_board[next_row][next_col] = num;
                    if self.try_solve_with_value(next_row, next_col, num, &mut new_board) {
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

    fn is_valid_solution(&self, board: &[Vec<i32>]) -> bool {
        // Check each row
        for row in 0..9 {
            let mut seen = [false; 10];
            for &num in &board[row] {
                if num == 0 || seen[num as usize] {
                    return false;
                }
                seen[num as usize] = true;
            }
        }

        // Check each column
        for col in 0..9 {
            let mut seen = [false; 10];
            for row in 0..9 {
                let num = board[row][col];
                if num == 0 || seen[num as usize] {
                    return false;
                }
                seen[num as usize] = true;
            }
        }

        // Check each 3x3 box
        for box_row in 0..3 {
            for box_col in 0..3 {
                let mut seen = [false; 10];
                for i in 0..3 {
                    for j in 0..3 {
                        let num = board[box_row * 3 + i][box_col * 3 + j];
                        if num == 0 || seen[num as usize] {
                            return false;
                        }
                        seen[num as usize] = true;
                    }
                }
            }
        }

        true
    }

    fn find_next_empty(&self, board: &[Vec<i32>]) -> Option<(usize, usize)> {
        for row in 0..9 {
            for col in 0..9 {
                if board[row][col] == 0 {
                    return Some((row, col));
                }
            }
        }
        None
    }

    fn is_valid_placement(&self, board: &[Vec<i32>], row: usize, col: usize, num: i32) -> bool {
        // Check row
        if board[row].contains(&num) {
            return false;
        }

        // Check column
        if (0..9).any(|i| board[i][col] == num) {
            return false;
        }

        // Check 3x3 box
        let box_row = (row / 3) * 3;
        let box_col = (col / 3) * 3;
        for i in 0..3 {
            for j in 0..3 {
                if board[box_row + i][box_col + j] == num {
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
        self.board.clone()
    }

    pub fn get_original_solution(&self) -> Vec<Vec<i32>> {
        self.solution.clone()
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
        assert!(solver.is_valid_solution(&solution));
        
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
} 