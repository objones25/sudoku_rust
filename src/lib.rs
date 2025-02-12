use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod solver;
pub mod api;
pub mod benchmark;

/// A bitset representation of candidate numbers for a Sudoku cell
#[derive(Debug, Clone, Copy, Default)]
pub struct CandidateSet(u16);

impl CandidateSet {
    /// Creates a new CandidateSet with all numbers 1-9 as candidates
    #[inline]
    pub fn all() -> Self {
        Self(0x1FF) // Binary: 0b111111111 (9 ones)
    }

    /// Creates an empty CandidateSet
    #[inline]
    pub fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub fn add_candidate(&mut self, n: u8) {
        debug_assert!(n >= 1 && n <= 9, "Invalid candidate number");
        self.0 |= 1 << (n - 1);
    }

    #[inline]
    pub fn remove_candidate(&mut self, n: u8) {
        debug_assert!(n >= 1 && n <= 9, "Invalid candidate number");
        self.0 &= !(1 << (n - 1));
    }

    #[inline]
    pub fn has_candidate(&self, n: u8) -> bool {
        debug_assert!(n >= 1 && n <= 9, "Invalid candidate number");
        (self.0 & (1 << (n - 1))) != 0
    }

    #[inline]
    pub fn count_candidates(&self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn iter_candidates(&self) -> impl Iterator<Item = u8> + '_ {
        (1..=9u8).filter(|&n| self.has_candidate(n))
    }
}

/// A flat array representation of a Sudoku board
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    cells: [u8; 81],
}

impl Board {
    /// Creates a new board from a 2D grid
    pub fn new(grid: &[Vec<i32>]) -> Self {
        let mut cells = [0; 81];
        for (i, row) in grid.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                debug_assert!(val >= 0 && val <= 9, "Invalid cell value");
                cells[i * 9 + j] = val as u8;
            }
        }
        Self { cells }
    }

    /// Creates an empty board
    #[inline]
    pub fn empty() -> Self {
        Self { cells: [0; 81] }
    }

    /// Gets the value at the specified position
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> u8 {
        debug_assert!(row < 9 && col < 9, "Invalid board position");
        self.cells[row * 9 + col]
    }

    /// Sets the value at the specified position
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: u8) {
        debug_assert!(row < 9 && col < 9, "Invalid board position");
        debug_assert!(value <= 9, "Invalid cell value");
        self.cells[row * 9 + col] = value;
    }

    /// Converts the board to a 2D vector representation
    pub fn to_vec(&self) -> Vec<Vec<i32>> {
        let mut result = vec![vec![0; 9]; 9];
        for i in 0..9 {
            for j in 0..9 {
                result[i][j] = self.get(i, j) as i32;
            }
        }
        result
    }

    /// Returns true if the cell at the specified position is empty (0)
    #[inline]
    pub fn is_empty_cell(&self, row: usize, col: usize) -> bool {
        self.get(row, col) == 0
    }

    /// Returns the box index (0-8) for a given row and column
    #[inline]
    pub fn get_box_index(row: usize, col: usize) -> usize {
        (row / 3) * 3 + col / 3
    }
}

#[derive(Debug, Error)]
pub enum SudokuError {
    #[error("Invalid board state")]
    InvalidBoard,
    #[error("API error: {0}")]
    ApiError(#[from] reqwest::Error),
    #[error("Invalid value at position ({row}, {col}): {value}")]
    InvalidValue {
        row: usize,
        col: usize,
        value: i32,
    },
    #[error("Benchmark error: {0}")]
    BenchmarkError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub value: Vec<Vec<i32>>,
    pub solution: Vec<Vec<i32>>,
    pub difficulty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardWrapper {
    pub grids: Vec<Grid>,
    pub results: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub newboard: BoardWrapper,
}

pub type Result<T> = std::result::Result<T, SudokuError>; 