use crate::{api, solver::Solver, Result, SudokuError};
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Results from a benchmark run
#[derive(Debug)]
pub struct BenchmarkResults {
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub total_boards: usize,
    pub solved_boards: usize,
    pub unique_solutions: usize,
    pub multiple_solutions: usize,
    pub difficulty_stats: DifficultyStats,
}

/// Statistics about puzzle difficulties
#[derive(Debug, Default)]
pub struct DifficultyStats {
    pub easy: usize,
    pub medium: usize,
    pub hard: usize,
    pub unknown: usize,
}

impl BenchmarkResults {
    /// Returns the success rate as a percentage (including both unique and multiple solutions)
    pub fn success_rate(&self) -> f64 {
        (self.solved_boards as f64 / self.total_boards as f64) * 100.0
    }

    /// Returns the unique solution rate as a percentage of solved boards
    pub fn unique_solution_rate(&self) -> f64 {
        (self.unique_solutions as f64 / self.solved_boards as f64) * 100.0
    }

    /// Pretty prints the benchmark results
    pub fn print_results(&self) {
        println!("\n=== Benchmark Results ===");
        println!("Total Duration: {:?}", self.total_duration);
        println!("Average Duration: {:?}", self.average_duration);
        println!("Min Duration: {:?}", self.min_duration);
        println!("Max Duration: {:?}", self.max_duration);
        println!("Total Boards: {}", self.total_boards);
        println!("Successfully Solved: {} ({:.1}%)", self.solved_boards, self.success_rate());
        println!("Unique Solutions: {} ({:.1}%)", self.unique_solutions, self.unique_solution_rate());
        println!("Multiple Solutions: {} ({:.1}%)", 
            self.multiple_solutions,
            (self.multiple_solutions as f64 / self.solved_boards as f64) * 100.0
        );
        
        println!("\nDifficulty Distribution:");
        println!("  Easy: {} ({:.1}%)", 
            self.difficulty_stats.easy,
            (self.difficulty_stats.easy as f64 / self.total_boards as f64) * 100.0
        );
        println!("  Medium: {} ({:.1}%)",
            self.difficulty_stats.medium,
            (self.difficulty_stats.medium as f64 / self.total_boards as f64) * 100.0
        );
        println!("  Hard: {} ({:.1}%)",
            self.difficulty_stats.hard,
            (self.difficulty_stats.hard as f64 / self.total_boards as f64) * 100.0
        );
        if self.difficulty_stats.unknown > 0 {
            println!("  Unknown: {} ({:.1}%)",
                self.difficulty_stats.unknown,
                (self.difficulty_stats.unknown as f64 / self.total_boards as f64) * 100.0
            );
        }
    }
}

/// Runs a benchmark solving the specified number of boards
pub async fn run_benchmark(board_count: usize, prefetch: bool) -> Result<BenchmarkResults> {
    if board_count == 0 {
        return Err(SudokuError::BenchmarkError("Board count must be greater than 0".to_string()));
    }

    // Prefetch boards if requested
    if prefetch {
        info!("Prefetching {} boards...", board_count);
        api::prefetch_boards(board_count).await?;
    }

    info!("Starting benchmark with {} boards...", board_count);
    let start = Instant::now();
    let mut min_duration = Duration::from_secs(u64::MAX);
    let mut max_duration = Duration::from_secs(0);
    let mut total_duration = Duration::from_secs(0);
    let mut solved_boards = 0;
    let mut unique_solutions = 0;
    let mut multiple_solutions = 0;
    let mut difficulty_stats = DifficultyStats::default();

    // Fetch all boards
    let boards = api::fetch_multiple_boards(board_count).await?;
    
    // Process each board
    for (i, board) in boards.iter().cloned().enumerate() {
        debug!("Solving board {}/{}", i + 1, board_count);
        
        // Update difficulty stats
        match board.difficulty.to_lowercase().as_str() {
            "easy" => difficulty_stats.easy += 1,
            "medium" => difficulty_stats.medium += 1,
            "hard" => difficulty_stats.hard += 1,
            _ => difficulty_stats.unknown += 1,
        }

        // Solve the board and measure time
        let solve_start = Instant::now();
        let mut solver = Solver::new(board);
        match solver.solve() {
            Ok(_) => {
                solved_boards += 1;
                if solver.has_unique_solution() {
                    unique_solutions += 1;
                } else {
                    multiple_solutions += 1;
                }
                let duration = solve_start.elapsed();
                min_duration = min_duration.min(duration);
                max_duration = max_duration.max(duration);
                total_duration += duration;
            }
            Err(e) => {
                debug!("Failed to solve board {}: {}", i + 1, e);
            }
        }
    }

    let results = BenchmarkResults {
        total_duration: start.elapsed(),
        average_duration: total_duration / board_count as u32,
        min_duration,
        max_duration,
        total_boards: board_count,
        solved_boards,
        unique_solutions,
        multiple_solutions,
        difficulty_stats,
    };

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_benchmark_small() {
        let timeout_duration = Duration::from_secs(30);
        match timeout(timeout_duration, run_benchmark(5, true)).await {
            Ok(Ok(results)) => {
                assert_eq!(results.total_boards, 5);
                assert!(results.total_duration > Duration::from_millis(0));
                assert!(results.success_rate() > 0.0);
            },
            Ok(Err(e)) => panic!("Benchmark failed: {}", e),
            Err(_) => panic!("Benchmark timed out"),
        }
    }

    #[tokio::test]
    async fn test_benchmark_invalid_count() {
        match run_benchmark(0, false).await {
            Ok(_) => panic!("Should fail with zero boards"),
            Err(SudokuError::BenchmarkError(_)) => (),
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_benchmark_with_prefetch() {
        let timeout_duration = Duration::from_secs(30);
        match timeout(timeout_duration, run_benchmark(3, true)).await {
            Ok(Ok(results)) => {
                assert_eq!(results.total_boards, 3);
                assert!(results.solved_boards > 0);
            },
            Ok(Err(e)) => panic!("Benchmark failed: {}", e),
            Err(_) => panic!("Benchmark timed out"),
        }
    }
} 