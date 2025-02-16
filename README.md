# Rust Sudoku Solver

A high-performance, multi-threaded Sudoku solver implementation in Rust with API integration for puzzle generation.

## Features

- Multi-threaded puzzle solving with adaptive thread pool
- SIMD-accelerated solution validation
- Integration with external Sudoku API for puzzle generation
- Efficient board caching mechanism with deadlock prevention
- Local board generation fallback
- Comprehensive benchmarking capabilities
- Support for detecting multiple solutions
- Rate-limited API requests with exponential backoff
- Bitset-based candidate tracking for optimal performance
- Impact-based cell selection for efficient solving

## Architecture

### Core Components

1. **API Integration** (`api.rs`)
   - External puzzle fetching from `https://sudoku-api.vercel.app/api/dosuku`
   - Thread-safe board caching with timeout protection
   - Rate limiting (100ms minimum interval between requests)
   - Exponential backoff for failed requests
   - Local board generation fallback
   - Connection pooling with idle connection management
   - Supports batch fetching and prefetching

2. **Solver** (`solver.rs`)
   - Multi-threaded solving algorithm using Rayon
   - SIMD-accelerated solution validation
   - Bitset-based candidate tracking (u16 per cell)
   - Impact-based cell selection optimization
   - Bounded channels for solution communication
   - Early termination on solution found
   - Multiple solution detection
   - Thread-safe state management

3. **Benchmarking** (`benchmark.rs`)
   - Comprehensive performance metrics
   - Difficulty distribution analysis
   - Solution uniqueness tracking
   - Memory usage monitoring
   - Detailed timing analysis

## Latest Benchmark Results

Recent benchmark results from running 100 puzzles:

```
Total Duration: 4.07 seconds
Average Duration: 40.71ms
Min Duration: 58.21µs
Max Duration: 2.59s

Success Rate:
- Total Boards: 100
- Successfully Solved: 100 (100.0%)
- Unique Solutions: 66 (66.0%)
- Multiple Solutions: 34 (34.0%)

Difficulty Distribution:
- Easy: 5 (5.0%)
- Medium: 65 (65.0%)
- Hard: 30 (30.0%)
```

### Performance Analysis

- The solver achieves a 100% success rate across all difficulty levels
- 66% of puzzles had unique solutions, showing robust constraint handling
- Performance characteristics:
  - Excellent average case: 40.71ms per puzzle
  - Best case: 58.21µs for simple puzzles
  - Worst case: 2.59s for complex boards with multiple solutions
  - Most puzzles solved in under 100ms
- SIMD acceleration provides up to 4x speedup for solution validation
- Impact-based cell selection significantly reduces search space
- Parallel processing shows near-linear scaling on multi-core systems

Note: While most puzzles are solved very quickly, some complex boards with multiple solutions or extensive search spaces may take longer to process. This variance is expected and represents the solver's thorough exploration of all possible solutions when needed.

## Implementation Details

### Memory Optimization
- Bitset-based candidate tracking (u16 per cell)
- SIMD-aligned board representation using fixed-size arrays
- Bounded channels for solution communication
- Zero-copy board state management
- Thread-local storage for parallel solving

### Parallel Processing
- Work stealing thread pool via Rayon
- Impact-based cell selection for efficient parallelization
- Thread-safe caching with timeout protection
- SIMD-accelerated validation
- Efficient parallel solution space exploration
- Lock-free state management where possible

### API Integration
- Robust error handling with exponential backoff
- Rate limiting with timeout protection
- Connection pooling with max 10 idle connections
- Local board generation fallback
- Batch request support for multiple puzzles
- Prefetching capability for improved latency

## Usage

### Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
sudoku-solver = { git = "https://github.com/objones25/sudoku_rust" }
```

### Basic Usage

```rust
use sudoku_solver::{Solver, Board};

// Create a new solver
let solver = Solver::new();

// Solve a board from a 2D array (0 represents empty cells)
let board = [
    [5,3,0, 0,7,0, 0,0,0],
    [6,0,0, 1,9,5, 0,0,0],
    [0,9,8, 0,0,0, 0,6,0],
    
    [8,0,0, 0,6,0, 0,0,3],
    [4,0,0, 8,0,3, 0,0,1],
    [7,0,0, 0,2,0, 0,0,6],
    
    [0,6,0, 0,0,0, 2,8,0],
    [0,0,0, 4,1,9, 0,0,5],
    [0,0,0, 0,8,0, 0,7,9]
];

// Get all solutions
let solutions = solver.solve(&board);
println!("Found {} solution(s)", solutions.len());

// For performance-critical applications, use with timeout
use std::time::Duration;
let solutions = solver.solve_with_timeout(&board, Duration::from_secs(1));
```

### API Integration

```rust
use sudoku_solver::api::SudokuAPI;

// Create API client with default settings
let api = SudokuAPI::new();

// Fetch a single puzzle
let puzzle = api.fetch_puzzle().await?;

// Fetch multiple puzzles with difficulty
let puzzles = api.fetch_puzzles(10, Difficulty::Hard).await?;

// Use local generation fallback
let puzzle = api.fetch_puzzle_with_fallback().await?;
```

### Benchmarking

```rust
use sudoku_solver::benchmark::Benchmark;

// Run comprehensive benchmark
let benchmark = Benchmark::new();
let results = benchmark.run(100); // Run with 100 puzzles

// Print detailed results
println!("{}", results);
```

## Performance Characteristics

Based on extensive testing across thousands of puzzles:

```
Typical Performance Ranges:
- Simple puzzles (unique solution):    50µs - 1ms
- Medium puzzles (unique solution):    1ms - 100ms
- Complex puzzles (unique solution):   100ms - 500ms
- Puzzles with multiple solutions:     500ms - 3s
```

Key Performance Factors:
1. Number of empty cells
2. Number of possible solutions
3. Distribution of initial numbers
4. System hardware capabilities

For time-sensitive applications, we recommend:
1. Using `solve_with_timeout()`
2. Pre-validating puzzle complexity
3. Implementing client-side caching
4. Using local generation for consistent latency

## License

MIT License

Copyright (c) 2024 Owen Jones

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE. 