use crate::{Grid, Result, SudokuError};

pub struct Solver {
    board: Vec<Vec<i32>>,
    solution: Vec<Vec<i32>>,
}

impl Solver {
    pub fn new(grid: Grid) -> Self {
        Self {
            board: grid.value,
            solution: grid.solution,
        }
    }

    pub fn solve(&mut self) -> Result<Vec<Vec<i32>>> {
        if self.solve_recursive(0, 0) {
            Ok(self.board.clone())
        } else {
            Err(SudokuError::InvalidBoard)
        }
    }

    pub fn verify_solution(&self) -> bool {
        self.board == self.solution
    }

    fn solve_recursive(&mut self, row: i32, col: i32) -> bool {
        if col as usize == 9 {
            return self.solve_recursive(row + 1, 0);
        }

        if row as usize == 9 {
            return true;
        }

        if self.board[row as usize][col as usize] != 0 {
            return self.solve_recursive(row, col + 1);
        }

        for num in 1..=9 {
            if self.is_valid(row as usize, col as usize, num) {
                self.board[row as usize][col as usize] = num;
                
                if self.solve_recursive(row, col + 1) {
                    return true;
                }
                
                self.board[row as usize][col as usize] = 0;
            }
        }
        
        false
    }

    fn is_valid(&self, row: usize, col: usize, num: i32) -> bool {
        // Check row
        if self.board[row].contains(&num) {
            return false;
        }

        // Check column
        if (0..9).any(|i| self.board[i][col] == num) {
            return false;
        }

        // Check 3x3 box
        let box_row = (row / 3) * 3;
        let box_col = (col / 3) * 3;
        
        for i in 0..3 {
            for j in 0..3 {
                if self.board[box_row + i][box_col + j] == num {
                    return false;
                }
            }
        }

        true
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
} 