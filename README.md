# Rust Sudoku Solver

A high-performance, multi-threaded Sudoku solver implementation in Rust with API integration for puzzle generation.

## Features

- Multi-threaded puzzle solving with adaptive thread pool
- Integration with external Sudoku API for puzzle generation
- Efficient board caching mechanism with LRU implementation
- Comprehensive benchmarking capabilities
- Support for detecting multiple solutions
- Rate-limited API requests with exponential backoff
- Bitset-based candidate tracking for optimal performance

## Architecture

### Core Components

1. **API Integration** (`api.rs`)
   - External puzzle fetching from `https://sudoku-api.vercel.app/api/dosuku`
   - LRU board caching (cache size: 50 boards)
   - Rate limiting (100ms minimum interval between requests)
   - Exponential backoff for failed requests
   - Supports batch fetching and prefetching

2. **Solver** (`solver.rs`)
   - Multi-threaded solving algorithm using Rayon
   - Bitset-based candidate tracking
   - Cell selection optimization based on candidate count
   - Parallel solution space exploration
   - Multiple solution detection

3. **Benchmarking** (`benchmark.rs`)
   - Comprehensive performance metrics
   - Difficulty distribution analysis
   - Solution uniqueness tracking
   - Memory usage monitoring

## Latest Benchmark Results

Recent benchmark results from running 100 puzzles:

```
Total Duration: 70.11 seconds
Average Duration: 94.57 ms
Min Duration: 150.17 µs
Max Duration: 3.08 seconds

Success Rate:
- Total Boards: 100
- Successfully Solved: 100 (100.0%)
- Unique Solutions: 40 (40.0%)
- Multiple Solutions: 60 (60.0%)

Difficulty Distribution:
- Easy: 2 (2.0%)
- Medium: 70 (70.0%)
- Hard: 28 (28.0%)
```

### Performance Analysis

- The solver achieves a 100% success rate across all difficulty levels
- 60% of puzzles had multiple valid solutions, indicating potential for constraint improvement
- Significant variance in solving time (150µs to 3.08s) shows complexity-dependent performance
- Average solving time of 94.57ms demonstrates excellent performance for complex puzzles

## Implementation Details

### Memory Optimization
- Bitset-based candidate tracking (u16 per cell)
- Efficient board representation using fixed-size arrays
- Memory-efficient caching with LRU policy
- Zero-copy board state management

### Parallel Processing
- Work stealing thread pool via Rayon
- Adaptive parallelization based on puzzle complexity
- Thread-safe caching mechanism
- Efficient parallel solution space exploration

### API Integration
- Robust error handling with exponential backoff
- Rate limiting to prevent API abuse
- Efficient board caching to reduce API calls
- Batch request support for multiple puzzles

## Usage

[TODO: Add usage examples and API documentation]

## Performance Optimization Tips

1. **Board Caching**
   - Utilize the LRU cache for repeated operations
   - Consider prefetching boards for batch operations
   - Monitor cache hit rates for optimal sizing

2. **Parallel Processing**
   - The solver automatically utilizes available CPU cores
   - Best performance on multi-core systems
   - Consider CPU affinity settings for optimal performance

3. **API Usage**
   - Respect rate limiting (100ms between requests)
   - Use batch fetching for multiple boards
   - Implement local caching for frequently used boards

## Future Improvements

1. **Solution Quality**
   - Implement advanced solving techniques (Hidden Singles, Naked Pairs)
   - Add constraint validation for puzzle generation
   - Improve multiple solution detection efficiency

2. **Performance**
   - Implement SIMD operations for candidate management
   - Add GPU acceleration for batch solving
   - Optimize memory allocation patterns

3. **Features**
   - Add puzzle generation capabilities
   - Implement difficulty rating system
   - Add solution visualization
   - Support for additional puzzle formats

## License

[TODO: Add license information] 