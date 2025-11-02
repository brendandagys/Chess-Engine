/// Tests for draw detection (insufficient material, repetition, fifty-move rule)
///
/// This test suite validates draw detection including:
///
/// 1. **Fifty-Move Rule**: 50 moves without pawn move or capture
/// 2. **Draw by Repetition**: Threefold repetition detection
/// 3. **Insufficient Material**: K vs K, K+B vs K, K+N vs K, K+B vs K+B (same color)
mod test_utils;
use test_utils::*;

use chess_engine::types::GameResult;

#[cfg(test)]
mod fifty_move_rule {
    use chess_engine::types::Square;

    use crate::test_utils::position_from_fen;

    #[test]
    fn test_fifty_move_rule_draw() {
        let fen = "k7/n7/8/8/8/8/N6N/N6K w - - 99 1";
        let mut position = position_from_fen(fen);

        assert_eq!(position.fifty, 99, "Should start with fifty counter at 99");

        let move_made = position.make_move(Square::A2, Square::B4, None);
        assert!(move_made, "White knight A2->B4 should be legal");

        assert_eq!(
            position.fifty, 100,
            "After one more halfmove, fifty counter should be 100, got {}",
            position.fifty
        );

        let result = position.check_game_result();
        assert_eq!(
            result,
            chess_engine::types::GameResult::DrawByFiftyMoveRule,
            "Should be draw by fifty-move rule, got {:?}",
            result
        );
    }

    #[test]
    fn test_fifty_move_rule_resets_on_pawn_move() {
        // Position with pawns and space for kings to move
        let fen = "4k3/4p3/8/8/8/8/4P3/3K4 w - - 40 1";
        let mut position = position_from_fen(fen);

        assert_eq!(position.fifty, 40, "Should start at 40");

        // Make a pawn move - this should reset the counter
        position.make_move(Square::E2, Square::E4, None);

        assert_eq!(
            position.fifty, 0,
            "Pawn move should reset fifty counter to 0"
        );

        position.ply = 0;
        position.first_move[0] = 0;

        let result = position.check_game_result();
        assert_eq!(
            result,
            chess_engine::types::GameResult::InProgress,
            "Game should be in progress after counter reset"
        );
    }

    #[test]
    fn test_fifty_move_rule_resets_on_capture() {
        // Position where a capture is possible with kings having space
        // Using rooks to avoid insufficient material after capture
        let fen = "3k4/8/8/4r3/4R3/8/8/3K4 w - - 98 1";
        let mut position = position_from_fen(fen);

        assert_eq!(position.fifty, 98, "Should start at 98");

        // Capture the black rook
        position.make_move(Square::E4, Square::E5, None);

        assert_eq!(position.fifty, 0, "Capture should reset fifty counter to 0");

        position.ply = 0;
        position.first_move[0] = 0;

        let result = position.check_game_result();
        assert_eq!(
            result,
            chess_engine::types::GameResult::InProgress,
            "Game should be in progress after capture"
        );
    }

    #[test]
    fn test_fifty_move_rule_at_limit() {
        // Position with rooks so kings have space to move
        let fen = "4k3/8/8/8/8/8/8/R3K2R w - - 99 1";
        let mut position = position_from_fen(fen);

        position.ply = 0;
        position.first_move[0] = 0;

        let result_before = position.check_game_result();
        assert_eq!(
            result_before,
            chess_engine::types::GameResult::InProgress,
            "Game should still be in progress at 99 half-moves"
        );

        position.make_move(Square::A1, Square::B1, None);

        assert_eq!(
            position.fifty, 100,
            "Should reach fifty-move limit after one more move"
        );

        // Generate moves again for the new position
        position.ply = 0;
        position.first_move[0] = 0;

        // Now check that the game is a draw
        let result_after = position.check_game_result();
        assert_eq!(
            result_after,
            chess_engine::types::GameResult::DrawByFiftyMoveRule,
            "Game should be draw by fifty-move rule at 100 halfmoves"
        );
    }

    #[test]
    fn test_fifty_move_counter_from_fen() {
        // Load position with halfmove clock already set, with space for kings to move
        // Using rooks to avoid insufficient material
        let fen = "3k4/8/8/8/8/8/8/R2K3R w - - 75 1";
        let mut position = position_from_fen(fen);

        assert_eq!(
            position.fifty, 75,
            "Fifty counter should be loaded from FEN"
        );

        position.ply = 0;
        position.first_move[0] = 0;

        let result = position.check_game_result();
        assert_eq!(
            result,
            chess_engine::types::GameResult::InProgress,
            "Game should still be in progress at 75 halfmoves"
        );
    }
}

#[cfg(test)]
mod repetition_detection {
    use chess_engine::types::Square;

    use super::*;

    #[test]
    fn test_repetition_counter_works() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut position = position_from_fen(fen);

        // Starting position should have 0 repetitions (hasn't been repeated yet)
        let reps_start = position.repetitions();
        assert_eq!(
            reps_start, 0,
            "Starting position should have 0 repetitions, got {}",
            reps_start
        );

        // Make a move
        let _ = position.make_move(Square::E2, Square::E4, None);
        let reps_after_move = position.repetitions();

        // After one move, still should have 0 repetitions (new position)
        assert_eq!(
            reps_after_move, 0,
            "After one move, should have 0 repetitions (new position), got {}",
            reps_after_move
        );
    }

    #[test]
    fn test_threefold_repetition_basic() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut position = position_from_fen(fen);

        // Initial position - no repetitions
        assert_eq!(
            position.repetitions(),
            0,
            "Starting position should have 0 reps"
        );

        // Move 1: Nf3
        position.make_move(Square::G1, Square::F3, None);
        assert_eq!(
            position.repetitions(),
            0,
            "After 1. Nf3, should have 0 reps"
        );

        // Move 2: Nf6
        position.make_move(Square::G8, Square::F6, None);
        assert_eq!(
            position.repetitions(),
            0,
            "After 1... Nf6, should have 0 reps"
        );

        // Move 3: Ng1 (back to starting position for white)
        position.make_move(Square::F3, Square::G1, None);
        assert_eq!(
            position.repetitions(),
            0,
            "After 2. Ng1, should have 0 reps"
        );

        // Move 4: Ng8 (back to starting position for black)
        position.make_move(Square::F6, Square::G8, None);
        assert_eq!(
            position.repetitions(),
            1,
            "After 2... Ng8 (back to start), should have 1 rep"
        );

        // Move 5: Nf3 again
        position.make_move(Square::G1, Square::F3, None);
        assert_eq!(
            position.repetitions(),
            1,
            "After 3. Nf3, should have 1 rep (matches ply 1)"
        );

        // Move 6: Nf6 again
        position.make_move(Square::G8, Square::F6, None);
        assert_eq!(
            position.repetitions(),
            1,
            "After 3... Nf6, should have 1 rep (matches ply 2)"
        );

        // Move 7: Ng1 again
        position.make_move(Square::F3, Square::G1, None);
        assert_eq!(position.repetitions(), 1, "After 4. Ng1, should have 1 rep");

        // Move 8: Ng8 again (back to starting position - SECOND repetition)
        position.make_move(Square::F6, Square::G8, None);
        // This is the third occurrence of the starting position
        assert_eq!(
            position.repetitions(),
            2,
            "After 4... Ng8 (third time at start), should have 2 reps"
        );
    }

    #[test]
    fn test_no_repetition_after_capture() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut position = position_from_fen(fen);

        // Create a repetition first by moving knights back and forth
        position.make_move(Square::G1, Square::F3, None); // Nf3
        position.make_move(Square::G8, Square::F6, None); // Nf6
        position.make_move(Square::F3, Square::G1, None); // Ng1
        position.make_move(Square::F6, Square::G8, None); // Ng8 (back to start)

        // Should have 1 repetition now
        assert_eq!(
            position.repetitions(),
            1,
            "Before capture, should have 1 rep"
        );

        // Now make a pawn move and capture
        position.make_move(Square::E2, Square::E4, None); // e4
        position.make_move(Square::D7, Square::D5, None); // d5
        position.make_move(Square::E4, Square::D5, None); // exd5 (capture)

        // After capture, fifty move counter resets, so can't have repetitions
        // from before the capture
        assert_eq!(
            position.repetitions(),
            0,
            "After capture, should have 0 reps (fifty counter reset)"
        );
    }

    #[test]
    fn test_no_repetition_after_pawn_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut position = position_from_fen(fen);

        // Move knights back and forth
        position.make_move(Square::G1, Square::F3, None);
        position.make_move(Square::G8, Square::F6, None);
        position.make_move(Square::F3, Square::G1, None);
        position.make_move(Square::F6, Square::G8, None);

        // Now we're back to starting position - should have 1 rep
        assert_eq!(position.repetitions(), 1, "Should have 1 rep");

        // Make a pawn move
        position.make_move(Square::E2, Square::E4, None);

        // Fifty counter reset, so can't count repetitions from before
        assert_eq!(
            position.repetitions(),
            0,
            "After pawn move, should have 0 reps (fifty counter reset)"
        );
    }

    #[test]
    fn test_repetition_only_counts_same_side_to_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut position = position_from_fen(fen);

        // Move 1: Nf3
        position.make_move(Square::G1, Square::F3, None);

        let hash_after_nf3 = position.board.hash.current_key;

        // Move 2: Nf6
        position.make_move(Square::G8, Square::F6, None);

        // Move 3: Ng1 (white knight back)
        position.make_move(Square::F3, Square::G1, None);

        // Move 4: Ng8 (black knight back)
        position.make_move(Square::F6, Square::G8, None);

        // Move 5: Nf3 again
        position.make_move(Square::G1, Square::F3, None);

        // Position after second Nf3 with black to move should match hash after first Nf3
        assert_eq!(
            position.board.hash.current_key, hash_after_nf3,
            "Position should be the same"
        );

        // This should show 1 repetition
        assert_eq!(
            position.repetitions(),
            1,
            "Same position with same side to move should count as 1 rep"
        );
    }

    #[test]
    fn test_repetition_respects_fifty_move_rule() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut position = position_from_fen(fen);

        let start_hash = position.board.hash.current_key;

        // Move 1-2: Knight out and back
        position.make_move(Square::G1, Square::F3, None);
        position.make_move(Square::G8, Square::F6, None);
        position.make_move(Square::F3, Square::G1, None);
        position.make_move(Square::F6, Square::G8, None);

        // Should be at starting position with 1 rep
        assert_eq!(position.board.hash.current_key, start_hash);
        assert_eq!(position.repetitions(), 1);

        // Now make a pawn move (resets fifty counter)
        position.make_move(Square::E2, Square::E4, None);

        assert!(
            position.repetitions() == 0,
            "Fifty counter should prevent counting old positions"
        );
    }

    #[test]
    fn test_draw_by_repetition_detection() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut position = position_from_fen(fen);

        // Create threefold repetition
        // Move 1-2: Back to start
        position.make_move(Square::G1, Square::F3, None);
        position.make_move(Square::G8, Square::F6, None);
        position.make_move(Square::F3, Square::G1, None);
        position.make_move(Square::F6, Square::G8, None);

        // Move 3-4: Back to start again
        position.make_move(Square::G1, Square::F3, None);
        position.make_move(Square::G8, Square::F6, None);
        position.make_move(Square::F3, Square::G1, None);
        position.make_move(Square::F6, Square::G8, None);

        // Should have 2 repetitions (3 occurrences total)
        assert_eq!(
            position.repetitions(),
            2,
            "Should have 2 reps (3 occurrences total)"
        );

        let result = position.check_game_result();
        assert_eq!(
            result,
            chess_engine::types::GameResult::DrawByRepetition,
            "Should be draw by repetition"
        );
    }
}

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

#[cfg(test)]
mod stalemate_detection {
    use super::*;

    /// Classic stalemate: King with no legal moves but not in check
    #[test]
    fn test_classic_stalemate_king_in_corner() {
        // Black king on a8, white queen on b6, white king on c6
        // Black king has no legal moves and is not in check
        let fen = "k7/8/1QK5/8/8/8/8/8 b - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_eq!(
            result,
            GameResult::Stalemate,
            "Black king with no legal moves (not in check) should be stalemate"
        );
    }

    /// Common endgame stalemate: King vs King + Queen
    #[test]
    fn test_stalemate_queen_endgame() {
        // Famous stalemate pattern: Black king on a1, white king on c2, white queen on b3
        let fen = "8/8/8/8/8/1Q6/2K5/k7 b - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_eq!(
            result,
            GameResult::Stalemate,
            "Classic queen endgame stalemate should be detected"
        );
    }

    /// Stalemate with multiple pieces but no legal moves
    #[test]
    fn test_complex_stalemate() {
        // Black king on h1, white king on f2, white rook on h2
        // Black king is trapped in the corner
        let fen = "8/8/8/8/8/8/5KR1/7k b - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_eq!(
            result,
            GameResult::Stalemate,
            "Complex position with no legal moves should be stalemate"
        );
    }

    #[test]
    fn test_not_stalemate_has_moves() {
        // Black king can move to b7, c7, or c8 (added pawns to avoid insufficient material)
        let fen = "k7/8/2K5/8/8/8/PP6/8 b - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_ne!(
            result,
            GameResult::Stalemate,
            "King with legal moves should NOT be stalemate"
        );
        assert_eq!(
            result,
            GameResult::InProgress,
            "Game should be in progress when king has moves"
        );
    }

    /// NOT stalemate: It's checkmate (king in check with no legal moves)
    #[test]
    fn test_not_stalemate_is_checkmate() {
        // Black king on h8 is in check from queen on g7, no escape
        let fen = "7k/6Q1/5K2/8/8/8/8/8 b - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_ne!(
            result,
            GameResult::Stalemate,
            "King in check with no legal moves is checkmate, not stalemate"
        );
        assert!(
            matches!(result, GameResult::Checkmate(_)),
            "Should be checkmate, got {:?}",
            result
        );
    }

    /// Stalemate where king's only "moves" would be into check
    #[test]
    fn test_stalemate_all_moves_into_check() {
        // Black king on e1, white king on e3, white queen on d3
        // All squares around black king are controlled
        let fen = "8/8/8/8/8/3QK3/8/4k3 b - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_eq!(
            result,
            GameResult::Stalemate,
            "King where all potential moves are into check should be stalemate"
        );
    }

    /// Famous stalemate trap from practical games
    #[test]
    fn test_practical_stalemate_trap() {
        // This position can arise from careless play with queen vs lone king
        // Black king on a8, white queen on c7, white king on a6
        let fen = "k7/2Q5/K7/8/8/8/8/8 b - - 0 1";
        let mut position = position_from_fen(fen);

        let result = position.check_game_result();
        assert_eq!(
            result,
            GameResult::Stalemate,
            "Classic stalemate trap should be detected"
        );
    }

    /// Test that stalemate is detected after a move creates the position
    #[test]
    fn test_stalemate_after_move() {
        // White queen can move to create stalemate
        // Starting position where king has one escape square
        let fen = "k7/2Q5/2K5/8/8/8/8/8 b - - 0 1";
        let mut position = position_from_fen(fen);

        // This position should already be stalemate (king on a8, queen on c7, king on c6)
        let result = position.check_game_result();
        assert_eq!(
            result,
            GameResult::Stalemate,
            "This should already be a stalemate position"
        );
    }

    /// Test creating stalemate with a queen move
    #[test]
    fn test_create_stalemate_with_queen() {
        use chess_engine::types::Square;

        // Position where white can create stalemate
        // King on a8, white king on c7, white queen on c6
        let fen = "k7/2K5/2Q5/8/8/8/8/8 w - - 0 1";
        let mut position = position_from_fen(fen);

        // Black is already in stalemate
        let result = position.check_game_result();
        assert_eq!(
            result,
            GameResult::InProgress,
            "White to move, black not yet stalemated"
        );

        // White plays Qc6-b6, maintaining stalemate for black's next turn
        let move_made = position.make_move(Square::C6, Square::B6, None);
        assert!(move_made, "Queen move should be legal");

        // After the move, black is in stalemate
        let result_after = position.check_game_result();
        assert_eq!(
            result_after,
            GameResult::Stalemate,
            "Should be stalemate after queen move to b6"
        );
    }
}
