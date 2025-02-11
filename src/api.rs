use crate::{ApiResponse, Grid, Result};

const API_URL: &str = "https://sudoku-api.vercel.app/api/dosuku";

pub async fn fetch_new_board() -> Result<Grid> {
    let response = reqwest::get(API_URL).await?.json::<ApiResponse>().await?;
    Ok(response.newboard.grids.into_iter().next().unwrap_or_else(|| Grid {
        value: vec![vec![0; 9]; 9],
        solution: vec![vec![0; 9]; 9],
        difficulty: "Unknown".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

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
} 