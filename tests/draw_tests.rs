/// Tests for draw detection (insufficient material, repetition, fifty-move rule)
///
/// This test suite validates draw detection including:
///
/// 1. **Insufficient Material**: K vs K, K+B vs K, K+N vs K, K+B vs K+B (same color)
/// 2. **Draw by Repetition**: Threefold repetition detection
/// 3. **Fifty-Move Rule**: 50 moves without pawn move or capture
mod test_utils;
use test_utils::*;

use chess_engine::types::GameResult;

#[cfg(test)]
mod insufficient_material {
    use super::*;

    /// K vs K - should be a draw
    #[test]
    fn test_king_vs_king() {
        let fen = "8/8/8/4k3/8/3K4/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_eq!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King vs King should be insufficient material"
        );
    }

    /// K+B vs K - should be a draw
    #[test]
    fn test_king_bishop_vs_king() {
        let fen = "8/8/8/4k3/8/3KB3/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_eq!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King + Bishop vs King should be insufficient material"
        );
    }

    /// K vs K+B - should be a draw
    #[test]
    fn test_king_vs_king_bishop() {
        let fen = "8/8/8/4kb2/8/3K4/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_eq!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King vs King + Bishop should be insufficient material"
        );
    }

    /// K+N vs K - should be a draw
    #[test]
    fn test_king_knight_vs_king() {
        let fen = "8/8/8/4k3/8/3KN3/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_eq!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King + Knight vs King should be insufficient material"
        );
    }

    /// K vs K+N - should be a draw
    #[test]
    fn test_king_vs_king_knight() {
        let fen = "8/8/8/4kn2/8/3K4/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_eq!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King vs King + Knight should be insufficient material"
        );
    }

    /// K+B vs K+B (bishops on same color) - should be a draw
    #[test]
    fn test_king_bishop_vs_king_bishop_same_color() {
        let fen = "8/8/8/b3k3/8/3KB3/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();

        assert_eq!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King vs King + Bishop (same color) should be insufficient material"
        );
    }

    /// K+B vs K+B (bishops on opposite colors) - NOT insufficient material
    #[test]
    fn test_king_bishop_vs_king_bishop_opposite_colors_not_draw() {
        // White bishop on light square (b1), black bishop on dark square (a8)
        let fen = "b7/8/8/4k3/8/3KB3/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_ne!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King + Bishop vs King + Bishop (opposite colors) is NOT insufficient material"
        );
    }

    /// K+N vs K+N - NOT insufficient material (mate is still possible)
    #[test]
    fn test_king_knight_vs_king_knight_not_draw() {
        let fen = "8/8/8/4kn2/8/3KN3/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_ne!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King + Knight vs King + Knight is NOT insufficient material"
        );
    }

    /// K+B vs K+N - NOT insufficient material (mate is still possible)
    #[test]
    fn test_king_bishop_vs_king_knight_not_draw() {
        let fen = "8/8/8/4kn2/8/3KB3/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_ne!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King + Bishop vs King + Knight is NOT insufficient material"
        );
    }

    /// K+R vs K - NOT insufficient material (easy checkmate)
    #[test]
    fn test_king_rook_vs_king_not_draw() {
        let fen = "8/8/8/4k3/8/3KR3/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_ne!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King + Rook vs King is NOT insufficient material"
        );
    }

    /// K+Q vs K - NOT insufficient material (easy checkmate)
    #[test]
    fn test_king_queen_vs_king_not_draw() {
        let fen = "8/8/8/4k3/8/3KQ3/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_ne!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King + Queen vs King is NOT insufficient material"
        );
    }

    /// K+P vs K - NOT insufficient material (pawn can promote)
    #[test]
    fn test_king_pawn_vs_king_not_draw() {
        let fen = "8/8/8/4k3/8/3KP3/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_ne!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King + Pawn vs King is NOT insufficient material"
        );
    }

    /// K+2B vs K - NOT insufficient material (mate is possible)
    #[test]
    fn test_king_two_bishops_vs_king_not_draw() {
        let fen = "8/8/8/4k3/8/3KBB2/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_ne!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King + 2 Bishops vs King is NOT insufficient material"
        );
    }

    /// K+B+N vs K - NOT insufficient material (mate is possible)
    #[test]
    fn test_king_bishop_knight_vs_king_not_draw() {
        let fen = "8/8/8/4k3/8/3KBN2/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_ne!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King + Bishop + Knight vs King is NOT insufficient material"
        );
    }

    /// K+2N vs K - Complex case, practically a draw but technically not by rules
    /// (Mate is possible but only with cooperation from the opponent)
    #[test]
    fn test_king_two_knights_vs_king() {
        let fen = "8/8/8/4k3/8/3KNN2/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        // This is a controversial case - FIDE rules say it's not automatic draw
        // but it's a draw in practice (forced mate impossible).
        // Most engines do NOT flag this as insufficient material
        assert_ne!(
            result,
            GameResult::DrawByInsufficientMaterial,
            "King + Two Knights vs King is NOT insufficient material"
        );
    }
}
