# Rust Sudoku Solver

A high-performance, multi-threaded Sudoku solver implementation in Rust with API integration for puzzle generation.

## Features

- Multi-threaded puzzle solving
- Integration with external Sudoku API for puzzle generation
- Board caching mechanism
- Comprehensive benchmarking capabilities
- Support for detecting multiple solutions
- Rate-limited API requests

## Architecture

### Core Components

1. **API Integration** (`api.rs`)
   - External puzzle fetching from `https://sudoku-api.vercel.app/api/dosuku`
   - Implements board caching (cache size: 50 boards)
   - Rate limiting (100ms minimum interval between requests)
   - Supports batch fetching and prefetching

2. **Solver** (`solver.rs`)
   - Multi-threaded solving algorithm
   - Supports detection of multiple solutions
   - Cell-by-cell solving with candidate tracking
   - Empty cell optimization

## Benchmark Results

Recent benchmark results from running 100 puzzles:

```
Total Duration: 96.98 seconds
Average Duration: 633.75 ms
Min Duration: 203.04 µs
Max Duration: 13.18 seconds

Success Rate:
- Total Boards: 100
- Successfully Solved: 100 (100.0%)
- Unique Solutions: 22 (22.0%)
- Multiple Solutions: 78 (78.0%)

Difficulty Distribution:
- Easy: 8 (8.0%)
- Medium: 52 (52.0%)
- Hard: 40 (40.0%)
```

### Performance Analysis

- The solver achieves a 100% success rate across all difficulty levels
- 78% of puzzles had multiple valid solutions, indicating they might not be well-constrained
- Significant variance in solving time (203µs to 13.18s) suggests complexity-dependent performance
- Average solving time of 633.75ms is reasonable for complex puzzles

## Implementation Details

### API Rate Limiting
- Minimum 100ms interval between requests
- Implements board caching to reduce API load
- Supports prefetching to optimize performance

### Solver Characteristics
- Multi-threaded implementation for parallel solution exploration
- Early detection of multiple solutions
- Optimization for empty cell count (ranges from 41 to 64 cells observed)
- Candidate tracking for efficient solution space exploration

### Logging and Debugging
- Comprehensive debug logging
- Tracks solving progress and solution discovery
- Thread-aware logging for parallel execution analysis

## Usage

[TODO: Add usage examples and API documentation]

## Performance Optimization Tips

1. **Board Caching**
   - Utilize the built-in board cache for repeated operations
   - Consider prefetching boards for batch operations

2. **Parallel Processing**
   - The solver automatically utilizes multiple threads
   - Best performance on multi-core systems

3. **API Usage**
   - Respect rate limiting (100ms between requests)
   - Use batch fetching for multiple boards
   - Implement local caching for frequently used boards

## Future Improvements

1. **Solution Quality**
   - Implement stricter puzzle validation
   - Reduce the percentage of multiple-solution puzzles
   - Add difficulty validation

2. **Performance**
   - Optimize worst-case solving time
   - Implement smarter candidate selection
   - Add solving strategy heuristics

3. **API Integration**
   - Add support for multiple puzzle sources
   - Implement failover mechanisms
   - Enhance caching strategies

## License

[TODO: Add license information] 