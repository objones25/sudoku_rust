use crate::{ApiResponse, Grid, Result};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, warn};
use once_cell::sync::Lazy;

const API_URL: &str = "https://sudoku-api.vercel.app/api/dosuku";
const CACHE_SIZE: usize = 50;
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(100);

static BOARD_CACHE: Lazy<Mutex<VecDeque<Grid>>> = Lazy::new(|| Mutex::new(VecDeque::with_capacity(CACHE_SIZE)));
static LAST_REQUEST: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));

/// Fetches a new Sudoku board from the API or cache.
/// Uses a local cache to store recently fetched boards and implements rate limiting.
pub async fn fetch_new_board() -> Result<Grid> {
    // Try to get a board from cache first
    if let Some(board) = get_from_cache() {
        debug!("Retrieved board from cache");
        return Ok(board);
    }

    // Rate limiting
    let now = Instant::now();
    let mut last_request = LAST_REQUEST.lock().unwrap();
    let elapsed = now.duration_since(*last_request);
    if elapsed < MIN_REQUEST_INTERVAL {
        sleep(MIN_REQUEST_INTERVAL - elapsed).await;
    }
    *last_request = Instant::now();

    // Fetch from API
    debug!("Fetching new board from API");
    match reqwest::get(API_URL).await?.json::<ApiResponse>().await {
        Ok(response) => {
            if let Some(board) = response.newboard.grids.into_iter().next() {
                add_to_cache(board.clone());
                Ok(board)
            } else {
                warn!("API returned empty grid list");
                Ok(Grid {
                    value: vec![vec![0; 9]; 9],
                    solution: vec![vec![0; 9]; 9],
                    difficulty: "Unknown".to_string(),
                })
            }
        }
        Err(e) => {
            warn!("Failed to parse API response: {}", e);
            Err(e.into())
        }
    }
}

/// Prefetches multiple boards in the background to fill the cache
pub async fn prefetch_boards(count: usize) -> Result<()> {
    debug!("Prefetching {} boards", count);
    for _ in 0..count {
        if let Ok(board) = fetch_new_board().await {
            add_to_cache(board);
        }
        sleep(MIN_REQUEST_INTERVAL).await;
    }
    Ok(())
}

/// Fetches multiple boards in parallel, respecting rate limits
pub async fn fetch_multiple_boards(count: usize) -> Result<Vec<Grid>> {
    let mut boards = Vec::with_capacity(count);
    
    // First, try to get as many boards from cache as possible
    while let Some(board) = get_from_cache() {
        boards.push(board);
        if boards.len() >= count {
            return Ok(boards);
        }
    }

    // Fetch remaining boards from API with rate limiting
    let remaining = count - boards.len();
    for _ in 0..remaining {
        if let Ok(board) = fetch_new_board().await {
            boards.push(board);
        }
        sleep(MIN_REQUEST_INTERVAL).await;
    }

    Ok(boards)
}

fn get_from_cache() -> Option<Grid> {
    BOARD_CACHE.lock().unwrap().pop_front()
}

fn add_to_cache(board: Grid) {
    let mut cache = BOARD_CACHE.lock().unwrap();
    if cache.len() >= CACHE_SIZE {
        cache.pop_back();
    }
    cache.push_front(board);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_fetch_new_board() {
        let board = fetch_new_board().await.unwrap();
        assert_eq!(board.value.len(), 9);
        for row in board.value.iter() {
            assert_eq!(row.len(), 9);
        }
        assert_eq!(board.solution.len(), 9);
        for row in board.solution.iter() {
            assert_eq!(row.len(), 9);
        }
        assert!(!board.difficulty.is_empty());
    }

    #[tokio::test]
    async fn test_cache() {
        // Fill cache
        let board = fetch_new_board().await.unwrap();
        add_to_cache(board.clone());
        
        // Verify cache retrieval
        let cached_board = get_from_cache().unwrap();
        assert_eq!(cached_board.value, board.value);
        assert_eq!(cached_board.solution, board.solution);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let start = Instant::now();
        let mut boards = Vec::new();
        
        // Try to fetch 5 boards quickly
        for _ in 0..5 {
            boards.push(fetch_new_board().await.unwrap());
        }
        
        let elapsed = start.elapsed();
        assert!(elapsed >= MIN_REQUEST_INTERVAL * 4, "Rate limiting should prevent rapid requests");
    }

    #[tokio::test]
    async fn test_prefetch() {
        // Clear cache
        while get_from_cache().is_some() {}
        
        // Prefetch 3 boards
        prefetch_boards(3).await.unwrap();
        
        // Verify cache size
        let mut count = 0;
        while get_from_cache().is_some() {
            count += 1;
        }
        assert_eq!(count, 3, "Cache should contain prefetched boards");
    }

    #[tokio::test]
    async fn test_fetch_multiple() {
        let count = 5;
        let timeout_duration = Duration::from_secs(10);
        
        match timeout(timeout_duration, fetch_multiple_boards(count)).await {
            Ok(Ok(boards)) => {
                assert_eq!(boards.len(), count, "Should fetch requested number of boards");
                for board in boards {
                    assert_eq!(board.value.len(), 9);
                    assert_eq!(board.solution.len(), 9);
                }
            },
            Ok(Err(e)) => panic!("Failed to fetch multiple boards: {}", e),
            Err(_) => panic!("Fetch multiple boards timed out"),
        }
    }
} 