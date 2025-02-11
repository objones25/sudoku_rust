//! A Sudoku solver that uses recursive backtracking and integrates with the Dosuku API.
//! 
//! This program:
//! 1. Fetches a new Sudoku puzzle from the Dosuku API
//! 2. Solves it using recursive backtracking
//! 3. Verifies the solution against the API's solution
//! 4. Displays both solutions if they differ

use sudoku::{api, solver::Solver};
use tracing::{info, error};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Fetching new Sudoku board from API...");
    
    match api::fetch_new_board().await {
        Ok(grid) => {
            info!("Original board (Difficulty: {}):", grid.difficulty);
            print_board(&grid.value);

            let mut solver = Solver::new(grid.clone());
            match solver.solve() {
                Ok(solution) => {
                    info!("Our solution:");
                    print_board(&solution);
                    
                    if solver.verify_solution() {
                        info!("✅ Solution verified against API's solution!");
                    } else {
                        error!("❌ Our solution differs from API's solution!");
                        info!("API's solution:");
                        print_board(&solver.get_original_solution());
                    }
                }
                Err(e) => error!("Failed to solve board: {}", e),
            }
        }
        Err(e) => error!("Failed to fetch board: {}", e),
    }
}

/// Prints a Sudoku board in a pretty format with grid lines.
/// 
/// # Arguments
/// 
/// * `board` - A 9x9 grid represented as a slice of vectors containing integers.
///            Empty cells are represented by 0.
fn print_board(board: &[Vec<i32>]) {
    println!("┌───────┬───────┬───────┐");
    for (i, row) in board.iter().enumerate() {
        print!("│ ");
        for (j, &cell) in row.iter().enumerate() {
            if cell == 0 {
                print!("· ");
            } else {
                print!("{} ", cell);
            }
            if (j + 1) % 3 == 0 && j < 8 {
                print!("│ ");
            }
        }
        println!("│");
        if (i + 1) % 3 == 0 && i < 8 {
            println!("├───────┼───────┼───────┤");
        }
    }
    println!("└───────┴───────┴───────┘");
}


