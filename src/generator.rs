use crate::{Grid, Result};
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::collections::HashSet;

pub struct BoardGenerator {
    rng: SmallRng,
    difficulty_weights: [(u32, &'static str); 3],
}

impl BoardGenerator {
    pub fn new() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
            difficulty_weights: [
                (4, "Easy"),
                (62, "Medium"),
                (34, "Hard"),
            ],
        }
    }

    pub fn generate(&mut self) -> Result<Grid> {
        let mut solution = vec![vec![0; 9]; 9];
        
        // Generate solved board
        self.fill_board(&mut solution, 0, 0);
        let mut board = solution.clone();

        // Remove numbers based on difficulty
        let difficulty = self.get_weighted_difficulty();
        let cells_to_remove = match difficulty {
            "Easy" => 30..=35,
            "Medium" => 40..=50,
            "Hard" => 51..=60,
            _ => 45..=50,
        };

        let remove_count = self.rng.gen_range(cells_to_remove);
        self.remove_numbers(&mut board, remove_count);

        Ok(Grid {
            value: board,
            solution,
            difficulty: difficulty.to_string(),
        })
    }

    fn fill_board(&mut self, board: &mut Vec<Vec<i32>>, row: usize, col: usize) -> bool {
        if row == 9 {
            return true;
        }

        let next_row = if col == 8 { row + 1 } else { row };
        let next_col = if col == 8 { 0 } else { col + 1 };

        if board[row][col] != 0 {
            return self.fill_board(board, next_row, next_col);
        }

        let mut numbers: Vec<i32> = (1..=9).collect();
        numbers.shuffle(&mut self.rng);

        for &num in &numbers {
            if self.is_valid_placement(board, row, col, num) {
                board[row][col] = num;
                if self.fill_board(board, next_row, next_col) {
                    return true;
                }
                board[row][col] = 0;
            }
        }
        false
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

    fn remove_numbers(&mut self, board: &mut Vec<Vec<i32>>, count: u32) {
        let mut positions: Vec<(usize, usize)> = (0..9)
            .flat_map(|i| (0..9).map(move |j| (i, j)))
            .collect();
        positions.shuffle(&mut self.rng);

        let mut removed = 0;
        let mut unique_solutions = HashSet::new();

        for (row, col) in positions {
            if removed >= count {
                break;
            }

            let temp = board[row][col];
            board[row][col] = 0;

            // Verify uniqueness (simplified check)
            if self.count_solutions(board, &mut unique_solutions, 2) > 1 {
                board[row][col] = temp;
                continue;
            }

            removed += 1;
        }
    }

    fn count_solutions(&self, board: &Vec<Vec<i32>>, solutions: &mut HashSet<String>, limit: usize) -> usize {
        if solutions.len() >= limit {
            return solutions.len();
        }

        if let Some(pos) = self.find_empty(board) {
            let (row, col) = pos;
            for num in 1..=9 {
                if self.is_valid_placement(board, row, col, num) {
                    let mut new_board = board.clone();
                    new_board[row][col] = num;
                    self.count_solutions(&new_board, solutions, limit);
                }
            }
        } else {
            solutions.insert(board.iter().flatten().map(|&x| x.to_string()).collect());
        }

        solutions.len()
    }

    fn find_empty(&self, board: &Vec<Vec<i32>>) -> Option<(usize, usize)> {
        for i in 0..9 {
            for j in 0..9 {
                if board[i][j] == 0 {
                    return Some((i, j));
                }
            }
        }
        None
    }

    fn get_weighted_difficulty(&mut self) -> &'static str {
        let total: u32 = self.difficulty_weights.iter().map(|&(w, _)| w).sum();
        let mut rand_val = self.rng.gen_range(0..total);
        
        for &(weight, difficulty) in &self.difficulty_weights {
            if rand_val < weight {
                return difficulty;
            }
            rand_val -= weight;
        }
        
        self.difficulty_weights[1].1 // Default to Medium
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_generation() {
        let mut generator = BoardGenerator::new();
        let grid = generator.generate().unwrap();
        
        // Verify board dimensions
        assert_eq!(grid.value.len(), 9);
        assert_eq!(grid.solution.len(), 9);
        for i in 0..9 {
            assert_eq!(grid.value[i].len(), 9);
            assert_eq!(grid.solution[i].len(), 9);
        }

        // Verify solution is valid
        for row in 0..9 {
            for col in 0..9 {
                if grid.value[row][col] != 0 {
                    assert_eq!(grid.value[row][col], grid.solution[row][col]);
                }
            }
        }
    }

    #[test]
    fn test_difficulty_distribution() {
        let mut generator = BoardGenerator::new();
        let mut difficulties = std::collections::HashMap::new();
        
        for _ in 0..100 {
            let grid = generator.generate().unwrap();
            *difficulties.entry(grid.difficulty).or_insert(0) += 1;
        }

        // Verify we have a mix of difficulties
        assert!(difficulties.len() >= 2, "Should generate multiple difficulty levels");
    }
} 