use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use sudoku::{api, solver::Solver};
use tokio::runtime::Runtime;
use std::collections::HashMap;

fn solve_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Create a benchmark group
    let mut group = c.benchmark_group("sudoku_solver");
    group.sample_size(10); // Reduce sample size due to API rate limiting
    
    // Fetch some boards for benchmarking
    let boards = rt.block_on(async {
        api::prefetch_boards(10).await.unwrap();
        api::fetch_multiple_boards(10).await.unwrap()
    });

    // Group boards by difficulty to avoid duplicate IDs
    let mut difficulty_groups: HashMap<String, Vec<_>> = HashMap::new();
    for board in boards {
        difficulty_groups
            .entry(board.difficulty.clone())
            .or_default()
            .push(board);
    }

    // Benchmark each difficulty level with unique IDs
    for (difficulty, boards) in difficulty_groups {
        for (idx, board) in boards.iter().enumerate() {
            let id = format!("{}_board_{}", difficulty, idx + 1);
            group.bench_with_input(
                BenchmarkId::new("solve", id),
                board,
                |b, board| {
                    b.iter(|| {
                        let mut solver = Solver::new(board.clone());
                        solver.solve().unwrap()
                    })
                },
            );
        }
    }
    group.finish();
}

criterion_group!(benches, solve_benchmark);
criterion_main!(benches); 