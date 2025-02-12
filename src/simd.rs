#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::arch::x86_64::*;

use crate::{Board, CandidateSet};

/// Feature detection for SIMD support
#[inline]
pub fn has_simd_support() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        is_x86_feature_detected!("sse2")
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

/// SIMD-optimized candidate set using 128-bit operations
#[derive(Debug, Clone, Copy)]
pub struct SimdCandidateSet {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    candidates: __m128i,
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    candidates: u16,
}

impl SimdCandidateSet {
    /// Creates a new SIMD candidate set with all candidates enabled
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    #[inline]
    pub unsafe fn new() -> Self {
        Self {
            candidates: _mm_set1_epi16(0x1FF) // All candidates available (9 bits set)
        }
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline]
    pub fn new() -> Self {
        Self {
            candidates: 0x1FF
        }
    }

    /// Removes multiple candidates at once using SIMD operations
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    #[inline]
    pub unsafe fn remove_candidates(&mut self, values: __m128i) {
        self.candidates = _mm_andnot_si128(values, self.candidates);
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline]
    pub fn remove_candidates(&mut self, values: u16) {
        self.candidates &= !values;
    }

    /// Checks for the presence of multiple candidates simultaneously
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    #[inline]
    pub unsafe fn has_candidates(&self, values: __m128i) -> bool {
        let result = _mm_and_si128(self.candidates, values);
        _mm_movemask_epi8(result) != 0
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline]
    pub fn has_candidates(&self, values: u16) -> bool {
        self.candidates & values != 0
    }

    /// Converts a regular CandidateSet to SIMD format
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    #[inline]
    pub unsafe fn from_candidate_set(set: CandidateSet) -> Self {
        Self {
            candidates: _mm_set1_epi16(set.0 as i16)
        }
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline]
    pub fn from_candidate_set(set: CandidateSet) -> Self {
        Self {
            candidates: set.0
        }
    }

    /// Converts back to a regular CandidateSet
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    #[inline]
    pub unsafe fn to_candidate_set(&self) -> CandidateSet {
        let value = _mm_extract_epi16(self.candidates, 0) as u16;
        CandidateSet(value)
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline]
    pub fn to_candidate_set(&self) -> CandidateSet {
        CandidateSet(self.candidates)
    }
}

/// SIMD-optimized board representation for efficient validation
#[derive(Debug)]
pub struct SimdBoard {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    rows: [__m128i; 9],
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    rows: [[u8; 9]; 9],
}

impl SimdBoard {
    /// Creates a new SIMD board from a regular board
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    pub unsafe fn from_board(board: &Board) -> Self {
        let mut simd_rows = [_mm_setzero_si128(); 9];
        
        for row in 0..9 {
            let row_data: [i16; 8] = board.cells[row * 9..row * 9 + 8]
                .iter()
                .map(|&x| x as i16)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            
            simd_rows[row] = _mm_loadu_si128(row_data.as_ptr() as *const __m128i);
        }
        
        Self { rows: simd_rows }
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn from_board(board: &Board) -> Self {
        let mut rows = [[0; 9]; 9];
        for row in 0..9 {
            for col in 0..9 {
                rows[row][col] = board.cells[row * 9 + col];
            }
        }
        Self { rows }
    }

    /// Validates a row using SIMD operations
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    #[inline]
    pub unsafe fn is_valid_row(&self, row: usize) -> bool {
        let row_data = self.rows[row];
        let shuffled = _mm_shuffle_epi32(row_data, 0x4E);
        let duplicates = _mm_cmpeq_epi16(row_data, shuffled);
        _mm_movemask_epi8(duplicates) == 0
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline]
    pub fn is_valid_row(&self, row: usize) -> bool {
        let mut seen = [false; 10];
        for &value in &self.rows[row] {
            if value == 0 || seen[value as usize] {
                return false;
            }
            seen[value as usize] = true;
        }
        true
    }

    /// Validates multiple rows simultaneously
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    pub unsafe fn validate_multiple_rows(&self, start_row: usize, count: usize) -> bool {
        (start_row..start_row + count)
            .all(|row| self.is_valid_row(row))
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn validate_multiple_rows(&self, start_row: usize, count: usize) -> bool {
        (start_row..start_row + count)
            .all(|row| self.is_valid_row(row))
    }
}

/// Provides optimized SIMD operations for board validation
pub struct SimdValidator;

impl SimdValidator {
    /// Validates a solution using SIMD operations where available
    pub fn validate_solution(board: &Board) -> bool {
        if has_simd_support() {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe {
                let simd_board = SimdBoard::from_board(board);
                // Validate rows in chunks of 2 for better throughput
                for chunk in (0..9).step_by(2) {
                    if !simd_board.validate_multiple_rows(chunk, chunk + 2.min(9 - chunk)) {
                        return false;
                    }
                }
                true
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                Self::validate_solution_fallback(board)
            }
        } else {
            Self::validate_solution_fallback(board)
        }
    }

    /// Non-SIMD fallback implementation for validation
    fn validate_solution_fallback(board: &Board) -> bool {
        // Check rows
        for row in 0..9 {
            let mut seen = [false; 10];
            for col in 0..9 {
                let num = board.get(row, col);
                if num == 0 || seen[num as usize] {
                    return false;
                }
                seen[num as usize] = true;
            }
        }

        // Check columns
        for col in 0..9 {
            let mut seen = [false; 10];
            for row in 0..9 {
                let num = board.get(row, col);
                if num == 0 || seen[num as usize] {
                    return false;
                }
                seen[num as usize] = true;
            }
        }

        // Check boxes
        for box_row in 0..3 {
            for box_col in 0..3 {
                let mut seen = [false; 10];
                for i in 0..3 {
                    for j in 0..3 {
                        let num = board.get(box_row * 3 + i, box_col * 3 + j);
                        if num == 0 || seen[num as usize] {
                            return false;
                        }
                        seen[num as usize] = true;
                    }
                }
            }
        }

        true
    }
}

/// SIMD-optimized board validation and candidate checking
#[derive(Debug)]
pub struct SimdSolver {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    row_masks: [__m128i; 9],
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    col_masks: [__m128i; 9],
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    box_masks: [__m128i; 9],
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    row_masks: [[u8; 9]; 9],
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    col_masks: [[u8; 9]; 9],
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    box_masks: [[u8; 9]; 9],
}

impl SimdSolver {
    /// Creates a new SIMD solver with precomputed masks
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    pub unsafe fn new(board: &Board) -> Self {
        let mut row_masks = [_mm_setzero_si128(); 9];
        let mut col_masks = [_mm_setzero_si128(); 9];
        let mut box_masks = [_mm_setzero_si128(); 9];

        // Precompute masks for each row, column, and box
        for row in 0..9 {
            let mut row_data = [0i16; 8];
            for col in 0..9 {
                let value = board.get(row, col);
                if value != 0 {
                    row_data[col.min(7)] |= 1 << (value - 1);
                }
            }
            row_masks[row] = _mm_loadu_si128(row_data.as_ptr() as *const __m128i);
        }

        // Similar for columns
        for col in 0..9 {
            let mut col_data = [0i16; 8];
            for row in 0..9 {
                let value = board.get(row, col);
                if value != 0 {
                    col_data[row.min(7)] |= 1 << (value - 1);
                }
            }
            col_masks[col] = _mm_loadu_si128(col_data.as_ptr() as *const __m128i);
        }

        // And boxes
        for box_idx in 0..9 {
            let box_row = (box_idx / 3) * 3;
            let box_col = (box_idx % 3) * 3;
            let mut box_data = [0i16; 8];
            
            for i in 0..3 {
                for j in 0..3 {
                    let value = board.get(box_row + i, box_col + j);
                    if value != 0 {
                        box_data[(i * 3 + j).min(7)] |= 1 << (value - 1);
                    }
                }
            }
            box_masks[box_idx] = _mm_loadu_si128(box_data.as_ptr() as *const __m128i);
        }

        Self {
            row_masks,
            col_masks,
            box_masks,
        }
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn new(board: &Board) -> Self {
        let mut row_masks = [[0; 9]; 9];
        let mut col_masks = [[0; 9]; 9];
        let mut box_masks = [[0; 9]; 9];

        // Initialize masks without SIMD
        for row in 0..9 {
            for col in 0..9 {
                let value = board.get(row, col);
                if value != 0 {
                    row_masks[row][col] = value;
                    col_masks[col][row] = value;
                    let box_idx = (row / 3) * 3 + col / 3;
                    let box_pos = (row % 3) * 3 + col % 3;
                    box_masks[box_idx][box_pos] = value;
                }
            }
        }

        Self {
            row_masks,
            col_masks,
            box_masks,
        }
    }

    /// Checks if a value can be placed at the given position using SIMD
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    pub unsafe fn is_valid_candidate(&self, row: usize, col: usize, value: u8) -> bool {
        let value_mask = _mm_set1_epi16(1 << (value - 1));
        
        // Check row
        let row_check = _mm_and_si128(self.row_masks[row], value_mask);
        if _mm_movemask_epi8(row_check) != 0 {
            return false;
        }

        // Check column
        let col_check = _mm_and_si128(self.col_masks[col], value_mask);
        if _mm_movemask_epi8(col_check) != 0 {
            return false;
        }

        // Check box
        let box_idx = (row / 3) * 3 + col / 3;
        let box_check = _mm_and_si128(self.box_masks[box_idx], value_mask);
        if _mm_movemask_epi8(box_check) != 0 {
            return false;
        }

        true
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn is_valid_candidate(&self, row: usize, col: usize, value: u8) -> bool {
        // Check row
        if self.row_masks[row].contains(&value) {
            return false;
        }

        // Check column
        if self.col_masks[col].contains(&value) {
            return false;
        }

        // Check box
        let box_idx = (row / 3) * 3 + col / 3;
        if self.box_masks[box_idx].contains(&value) {
            return false;
        }

        true
    }

    /// Updates the masks when a value is placed
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    pub unsafe fn update_masks(&mut self, row: usize, col: usize, value: u8) {
        let value_mask = _mm_set1_epi16(1 << (value - 1));
        
        // Update row mask
        self.row_masks[row] = _mm_or_si128(self.row_masks[row], value_mask);
        
        // Update column mask
        self.col_masks[col] = _mm_or_si128(self.col_masks[col], value_mask);
        
        // Update box mask
        let box_idx = (row / 3) * 3 + col / 3;
        self.box_masks[box_idx] = _mm_or_si128(self.box_masks[box_idx], value_mask);
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn update_masks(&mut self, row: usize, col: usize, value: u8) {
        self.row_masks[row][col] = value;
        self.col_masks[col][row] = value;
        let box_idx = (row / 3) * 3 + col / 3;
        let box_pos = (row % 3) * 3 + col % 3;
        self.box_masks[box_idx][box_pos] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_support_detection() {
        let _ = has_simd_support();
    }

    #[test]
    fn test_simd_candidate_set() {
        if !has_simd_support() {
            return;
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            let mut simd_set = SimdCandidateSet::new();
            let values = _mm_set1_epi16(0x1); // Remove candidate 1
            simd_set.remove_candidates(values);
            assert!(!simd_set.has_candidates(values));
        }
    }

    #[test]
    fn test_simd_board_validation() {
        let mut board = Board::empty();
        // Set up a valid row
        for i in 0..9 {
            board.set(0, i, (i + 1) as u8);
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            if has_simd_support() {
                let simd_board = SimdBoard::from_board(&board);
                assert!(simd_board.is_valid_row(0));
            }
        }
    }
} 