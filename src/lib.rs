use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod solver;
pub mod api;
pub mod benchmark;

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