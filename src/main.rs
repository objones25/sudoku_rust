//! A Sudoku solver that uses recursive backtracking and integrates with the Dosuku API.
//! 
//! This program:
//! 1. Fetches a new Sudoku puzzle from the Dosuku API
//! 2. Solves it using recursive backtracking with parallel processing
//! 3. Verifies the solution against the API's solution
//! 4. Checks for solution uniqueness
//! 5. Displays both solutions if they differ

use sudoku::{api, solver::Solver, benchmark};
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use std::env;

#[tokio::main]
async fn main() {
    // Initialize logging with debug level
    FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_thread_names(true)
        .with_ansi(true)
        .pretty()
        .init();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("benchmark") => {
            let count = args.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(100);
            
            info!("Running benchmark with {} boards...", count);
            match benchmark::run_benchmark(count, true).await {
                Ok(results) => results.print_results(),
                Err(e) => error!("Benchmark failed: {}", e),
            }
        }
        _ => {
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

                            if solver.has_unique_solution() {
                                info!("✅ This puzzle has a unique solution!");
                            } else {
                                info!("⚠️  This puzzle has multiple valid solutions!");
                            }
                        }
                        Err(e) => error!("Failed to solve board: {}", e),
                    }
                }
                Err(e) => error!("Failed to fetch board: {}", e),
            }
        }
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


