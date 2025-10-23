/// Comprehensive evaluation tests for the chess engine
///
/// This test suite validates the evaluation function which assigns a numerical
/// score to chess positions. The evaluation considers:
///
/// 1. **Material Balance**: Piece values (Queen > Rook > Bishop ≈ Knight > Pawn)
/// 2. **Piece Positioning**: Central control, piece activity, piece-square tables
/// 3. **Pawn Structure**: Passed pawns, isolated pawns, pawn chains, doubled pawns
/// 4. **Rook Placement**: Open files, semi-open files, file control
/// 5. **King Safety**: Pawn shields when queens are on board, centralization in endgames
///
/// The evaluation function returns a score from white's perspective:
/// - Positive scores favor white
/// - Negative scores favor black
/// - Zero indicates material/positional equality
mod test_utils;

use chess_engine::{
    position::Position,
    time::TimeManager,
    types::{Board, Piece, Side, Square},
};
use test_utils::*;

/// Creates a position with only kings (required for valid chess positions)
fn position_with_kings(side_to_move: Side) -> Position {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    position.board = Board::empty();
    position
        .board
        .add_piece(Side::White, Piece::King, Square::E1);
    position
        .board
        .add_piece(Side::Black, Piece::King, Square::E8);

    position.castle = 0;
    position.side = side_to_move;
    position.other_side = side_to_move.opponent();
    position.ply_from_start_of_game = 0;
    position.fifty = 0;

    position
}

/// Helper to get evaluation from white's perspective
fn evaluate(position: &Position) -> i32 {
    position.evaluate_position()
}

#[cfg(test)]
mod material_evaluation {
    use super::*;

    #[test]
    fn test_equal_material_is_roughly_equal() {
        let position = Position::new(TimeManager::default());
        let score = evaluate(&position);

        // Starting position should be roughly equal (within positional differences)
        assert!(
            score.abs() < 100,
            "Starting position score should be close to 0, got {}",
            score
        );
    }

    #[test]
    fn test_extra_pawn_advantage() {
        let mut position = position_with_kings(Side::White);

        // Add equal material except one extra white pawn
        position
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E4);

        let score = evaluate(&position);

        // White should have a positive score (advantage)
        assert!(
            score > 0,
            "Extra white pawn should give positive score, got {}",
            score
        );
    }

    #[test]
    fn test_piece_value_hierarchy() {
        // Queen > Rook > Bishop ≈ Knight > Pawn
        let mut white_queen = position_with_kings(Side::White);
        white_queen
            .board
            .add_piece(Side::White, Piece::Queen, Square::D4);
        let queen_score = evaluate(&white_queen);

        let mut white_rook = position_with_kings(Side::White);
        white_rook
            .board
            .add_piece(Side::White, Piece::Rook, Square::D4);
        let rook_score = evaluate(&white_rook);

        let mut white_bishop = position_with_kings(Side::White);
        white_bishop
            .board
            .add_piece(Side::White, Piece::Bishop, Square::D4);
        let bishop_score = evaluate(&white_bishop);

        let mut white_knight = position_with_kings(Side::White);
        white_knight
            .board
            .add_piece(Side::White, Piece::Knight, Square::D4);
        let knight_score = evaluate(&white_knight);

        let mut white_pawn = position_with_kings(Side::White);
        white_pawn
            .board
            .add_piece(Side::White, Piece::Pawn, Square::D4);
        let pawn_score = evaluate(&white_pawn);

        assert!(
            queen_score > rook_score,
            "Queen should be worth more than rook"
        );
        assert!(
            rook_score > bishop_score,
            "Rook should be worth more than bishop"
        );
        assert!(
            bishop_score > pawn_score,
            "Bishop should be worth more than pawn"
        );
        assert!(
            knight_score > pawn_score,
            "Knight should be worth more than pawn"
        );
    }

    #[test]
    fn test_black_material_advantage() {
        let mut position = position_with_kings(Side::White);

        // Give black extra material
        position
            .board
            .add_piece(Side::Black, Piece::Rook, Square::A8);

        let score = evaluate(&position);

        // Black advantage should give negative score (from white's perspective)
        assert!(
            score < 0,
            "Black material advantage should give negative score, got {}",
            score
        );
    }

    #[test]
    fn test_symmetric_material_evaluation() {
        let mut white_advantage = position_with_kings(Side::White);
        white_advantage
            .board
            .add_piece(Side::White, Piece::Knight, Square::D4);
        let white_score = evaluate(&white_advantage);

        let mut black_advantage = position_with_kings(Side::White);
        black_advantage
            .board
            .add_piece(Side::Black, Piece::Knight, Square::D5);
        let black_score = evaluate(&black_advantage);

        // White advantage and black advantage should be roughly opposite
        // (may differ slightly due to positional bonuses)
        assert!(
            (white_score + black_score).abs() < 50,
            "Symmetric positions should have roughly opposite scores: {} vs {}",
            white_score,
            black_score
        );

        assert!(
            white_score == -black_score,
            "White {white_score} and black {black_score} score should equal"
        );
    }
}

#[cfg(test)]
mod positional_evaluation {
    use super::*;

    #[test]
    fn test_central_pawns_better_than_edge_pawns() {
        let mut center_pawn = position_with_kings(Side::White);
        center_pawn
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E4);
        let center_score = evaluate(&center_pawn);

        let mut edge_pawn = position_with_kings(Side::White);
        edge_pawn
            .board
            .add_piece(Side::White, Piece::Pawn, Square::A4);
        let edge_score = evaluate(&edge_pawn);

        assert!(
            center_score > edge_score,
            "Central pawn should score higher than edge pawn: {} vs {}",
            center_score,
            edge_score
        );
    }

    #[test]
    fn test_knight_centralization() {
        let mut center_knight = position_with_kings(Side::White);
        center_knight
            .board
            .add_piece(Side::White, Piece::Knight, Square::E4);
        let center_score = evaluate(&center_knight);

        let mut corner_knight = position_with_kings(Side::White);
        corner_knight
            .board
            .add_piece(Side::White, Piece::Knight, Square::A1);
        let corner_score = evaluate(&corner_knight);

        assert!(
            center_score > corner_score,
            "Centralized knight should score higher than corner knight: {} vs {}",
            center_score,
            corner_score
        );
    }

    #[test]
    fn test_advanced_pawns_are_valuable() {
        let mut advanced_pawn = position_with_kings(Side::White);
        advanced_pawn
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E6);
        let advanced_score = evaluate(&advanced_pawn);

        let mut starting_pawn = position_with_kings(Side::White);
        starting_pawn
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E2);
        let starting_score = evaluate(&starting_pawn);

        assert!(
            advanced_score > starting_score,
            "Advanced pawn should score higher: {} vs {}",
            advanced_score,
            starting_score
        );
    }

    #[test]
    fn test_bishop_pair_positioning() {
        let mut center_bishop = position_with_kings(Side::White);
        center_bishop
            .board
            .add_piece(Side::White, Piece::Bishop, Square::D4);
        let center_score = evaluate(&center_bishop);

        let mut corner_bishop = position_with_kings(Side::White);
        corner_bishop
            .board
            .add_piece(Side::White, Piece::Bishop, Square::A1);
        let corner_score = evaluate(&corner_bishop);

        assert!(
            center_score > corner_score,
            "Centralized bishop should score higher: {} vs {}",
            center_score,
            corner_score
        );
    }

    #[test]
    fn test_bishop_pair_positioning_closer() {
        let mut center_bishop = position_with_kings(Side::White);
        center_bishop
            .board
            .add_piece(Side::White, Piece::Bishop, Square::D4);
        let center_score = evaluate(&center_bishop);

        let mut corner_bishop = position_with_kings(Side::White);
        corner_bishop
            .board
            .add_piece(Side::White, Piece::Bishop, Square::F3);
        let corner_score = evaluate(&corner_bishop);

        assert!(
            center_score > corner_score,
            "Centralized bishop should score higher: {} vs {}",
            center_score,
            corner_score
        );
    }
}

#[cfg(test)]
mod pawn_structure {
    use super::*;

    #[test]
    fn test_isolated_pawn_penalty() {
        // Isolated pawn (no friendly pawns on adjacent files)
        let mut isolated = position_with_kings(Side::White);
        isolated
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E4);
        let isolated_score = evaluate(&isolated);

        // Connected pawn (has support)
        let mut connected = position_with_kings(Side::White);
        connected
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E4);
        connected
            .board
            .add_piece(Side::White, Piece::Pawn, Square::D4);
        let connected_score = evaluate(&connected);

        assert!(
            connected_score > isolated_score,
            "Connected pawns should score higher than isolated pawn: {} vs {}",
            connected_score,
            isolated_score
        );
    }

    #[test]
    fn test_passed_pawn_bonus() {
        // Passed pawn (no enemy pawns can stop it)
        let mut passed = position_with_kings(Side::White);
        passed.board.add_piece(Side::White, Piece::Pawn, Square::E6);
        let passed_score = evaluate(&passed);

        // Blocked pawn
        let mut blocked = position_with_kings(Side::White);
        blocked
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E6);
        blocked
            .board
            .add_piece(Side::Black, Piece::Pawn, Square::E7);
        let blocked_score = evaluate(&blocked);

        assert!(
            passed_score > blocked_score,
            "Passed pawn should score higher than blocked pawn: {} vs {}",
            passed_score,
            blocked_score
        );
    }

    #[test]
    fn test_advanced_passed_pawn_extra_value() {
        let mut far_passed = position_with_kings(Side::White);
        far_passed
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E6);
        let far_score = evaluate(&far_passed);

        let mut close_passed = position_with_kings(Side::White);
        close_passed
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E3);
        let close_score = evaluate(&close_passed);

        assert!(
            far_score > close_score,
            "Advanced passed pawn should score higher: {} vs {}",
            far_score,
            close_score
        );
    }

    #[test]
    fn test_doubled_pawns() {
        let mut doubled = position_with_kings(Side::White);
        doubled
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E3);
        doubled
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E4);
        let doubled_score = evaluate(&doubled);

        let mut spread = position_with_kings(Side::White);
        spread.board.add_piece(Side::White, Piece::Pawn, Square::E3);
        spread.board.add_piece(Side::White, Piece::Pawn, Square::D3);
        let spread_score = evaluate(&spread);

        // Spread pawns should generally be better than doubled
        // (though evaluation may not heavily penalize this directly)
        assert!(
            spread_score >= doubled_score,
            "Spread pawns should score at least as well as doubled: {} vs {}",
            spread_score,
            doubled_score
        );
    }

    #[test]
    fn test_pawn_chain() {
        let mut chain = position_with_kings(Side::White);
        chain.board.add_piece(Side::White, Piece::Pawn, Square::D4);
        chain.board.add_piece(Side::White, Piece::Pawn, Square::E3);
        chain.board.add_piece(Side::White, Piece::Pawn, Square::F2);
        let chain_score = evaluate(&chain);

        let mut isolated_pawns = position_with_kings(Side::White);
        isolated_pawns
            .board
            .add_piece(Side::White, Piece::Pawn, Square::A4);
        isolated_pawns
            .board
            .add_piece(Side::White, Piece::Pawn, Square::D4);
        isolated_pawns
            .board
            .add_piece(Side::White, Piece::Pawn, Square::H4);
        let isolated_score = evaluate(&isolated_pawns);

        assert!(
            chain_score > isolated_score,
            "Pawn chain should score higher than isolated pawns: {} vs {}",
            chain_score,
            isolated_score
        );
    }
}

#[cfg(test)]
mod rook_evaluation {
    use super::*;

    #[test]
    fn test_rook_on_open_file_middle() {
        // Rook on completely open file (no pawns)
        let mut open_file_middle = position_with_kings(Side::White);
        open_file_middle
            .board
            .add_piece(Side::White, Piece::Rook, Square::E1);
        open_file_middle
            .board
            .add_piece(Side::White, Piece::Pawn, Square::F2);
        let open_score = evaluate(&open_file_middle);

        // Rook behind own pawn
        let mut blocked = position_with_kings(Side::White);
        blocked
            .board
            .add_piece(Side::White, Piece::Rook, Square::E1);
        blocked
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E2);
        let blocked_score = evaluate(&blocked);

        assert!(
            open_score > blocked_score,
            "Blocked file ({}) should score lower than open file ({})",
            blocked_score,
            open_score
        );
    }

    #[test]
    fn test_rook_on_semi_open_file_middle() {
        // Semi-open file (only enemy pawn)
        let mut semi_open = position_with_kings(Side::White);
        semi_open
            .board
            .add_piece(Side::White, Piece::Rook, Square::E1);
        semi_open
            .board
            .add_piece(Side::White, Piece::Pawn, Square::F2);
        semi_open
            .board
            .add_piece(Side::Black, Piece::Pawn, Square::E7);
        let semi_open_score = evaluate(&semi_open);

        // Closed file (both own and enemy pawns)
        let mut closed = position_with_kings(Side::White);
        closed.board.add_piece(Side::White, Piece::Rook, Square::E1);
        closed.board.add_piece(Side::White, Piece::Pawn, Square::E2);
        closed.board.add_piece(Side::Black, Piece::Pawn, Square::E7);
        let closed_score = evaluate(&closed);

        assert!(
            semi_open_score > closed_score,
            "Semi-open file should score higher than closed file: {} vs {}",
            semi_open_score,
            closed_score
        );
    }

    #[test]
    fn test_rook_file_activity() {
        // Test that the evaluation function gives bonuses for rook file control
        let mut open_file_middle = position_with_kings(Side::White);
        open_file_middle
            .board
            .add_piece(Side::White, Piece::Rook, Square::E1);
        let open_file_middle_score = evaluate(&open_file_middle);

        let mut open_file_side = position_with_kings(Side::White);
        open_file_side
            .board
            .add_piece(Side::White, Piece::Rook, Square::C1);
        let open_file_side_score = evaluate(&open_file_side);

        // Both rooks on open files should have similar evaluations (same bonus)
        // May differ slightly due to positional square tables
        assert!(
            (open_file_middle_score - open_file_side_score).abs() < 30,
            "Rooks on different open files should have similar bonus: {} vs {}",
            open_file_middle_score,
            open_file_side_score
        );

        assert!(open_file_middle_score > open_file_side_score)
    }
}

#[cfg(test)]
mod king_evaluation {
    use super::*;

    #[test]
    fn test_king_safety_with_queens_on_board() {
        // King with pawn shield (kingside castle)
        let mut safe_king = position_with_kings(Side::White);
        safe_king
            .board
            .remove_piece(Side::White, Piece::King, Square::E1);
        safe_king
            .board
            .add_piece(Side::White, Piece::King, Square::G1);
        safe_king
            .board
            .add_piece(Side::White, Piece::Pawn, Square::F2);
        safe_king
            .board
            .add_piece(Side::White, Piece::Pawn, Square::G2);
        safe_king
            .board
            .add_piece(Side::White, Piece::Pawn, Square::H2);
        safe_king
            .board
            .add_piece(Side::Black, Piece::Pawn, Square::A7);

        // Irrelevant opponent pawns to ensure equal material score
        safe_king
            .board
            .add_piece(Side::Black, Piece::Pawn, Square::B7);
        safe_king
            .board
            .add_piece(Side::Black, Piece::Pawn, Square::C7);
        safe_king
            .board
            .add_piece(Side::Black, Piece::Queen, Square::D8);

        let safe_score = evaluate(&safe_king);

        // Exposed king
        let mut exposed_king = position_with_kings(Side::White);
        exposed_king
            .board
            .remove_piece(Side::White, Piece::King, Square::E1);
        exposed_king
            .board
            .add_piece(Side::White, Piece::King, Square::E4);
        exposed_king
            .board
            .add_piece(Side::Black, Piece::Queen, Square::D8);

        let exposed_score = evaluate(&exposed_king);

        assert!(
            safe_score > exposed_score,
            "King with pawn shield should score higher when queens are on: {} vs {}",
            safe_score,
            exposed_score
        );
    }

    #[test]
    fn test_king_centralization_in_endgame() {
        // King in center (no queens = endgame)
        let mut central_king = position_with_kings(Side::White);
        central_king
            .board
            .remove_piece(Side::White, Piece::King, Square::E1);
        central_king
            .board
            .add_piece(Side::White, Piece::King, Square::E4);
        let central_score = evaluate(&central_king);

        // King in corner
        let corner_king = position_with_kings(Side::White);
        let corner_score = evaluate(&corner_king);

        assert!(
            central_score > corner_score,
            "Centralized king in endgame should score higher: {} vs {}",
            central_score,
            corner_score
        );
    }

    #[test]
    fn test_king_activity_without_queens() {
        // Without queens (endgame), king should prefer central squares
        let mut active = position_with_kings(Side::White);
        active
            .board
            .remove_piece(Side::White, Piece::King, Square::E1);
        active.board.add_piece(Side::White, Piece::King, Square::D4);
        let active_score = evaluate(&active);

        let passive = position_with_kings(Side::White);
        let passive_score = evaluate(&passive);

        assert!(
            active_score > passive_score,
            "Active king in endgame should score higher: {} vs {}",
            active_score,
            passive_score
        );
    }

    #[test]
    fn test_kingside_pawn_shield() {
        let mut with_shield = position_with_kings(Side::White);
        with_shield
            .board
            .remove_piece(Side::White, Piece::King, Square::E1);
        with_shield
            .board
            .add_piece(Side::White, Piece::King, Square::G1);
        with_shield
            .board
            .add_piece(Side::White, Piece::Pawn, Square::F2);
        with_shield
            .board
            .add_piece(Side::White, Piece::Pawn, Square::G2);
        with_shield
            .board
            .add_piece(Side::White, Piece::Pawn, Square::H2);
        with_shield
            .board
            .add_piece(Side::Black, Piece::Queen, Square::D8);
        let shield_score = evaluate(&with_shield);

        let mut without_shield = position_with_kings(Side::White);
        without_shield
            .board
            .remove_piece(Side::White, Piece::King, Square::E1);
        without_shield
            .board
            .add_piece(Side::White, Piece::King, Square::G1);
        without_shield
            .board
            .add_piece(Side::Black, Piece::Queen, Square::D8);
        let no_shield_score = evaluate(&without_shield);

        assert!(
            shield_score > no_shield_score,
            "King with pawn shield should score higher: {} vs {}",
            shield_score,
            no_shield_score
        );
    }

    #[test]
    fn test_queenside_pawn_shield() {
        let mut with_shield = position_with_kings(Side::White);
        with_shield
            .board
            .remove_piece(Side::White, Piece::King, Square::E1);
        with_shield
            .board
            .add_piece(Side::White, Piece::King, Square::B1);
        with_shield
            .board
            .add_piece(Side::White, Piece::Pawn, Square::A2);
        with_shield
            .board
            .add_piece(Side::White, Piece::Pawn, Square::B2);
        with_shield
            .board
            .add_piece(Side::White, Piece::Pawn, Square::C2);
        with_shield
            .board
            .add_piece(Side::Black, Piece::Queen, Square::D8);
        let shield_score = evaluate(&with_shield);

        let mut without_shield = position_with_kings(Side::White);
        without_shield
            .board
            .remove_piece(Side::White, Piece::King, Square::E1);
        without_shield
            .board
            .add_piece(Side::White, Piece::King, Square::B1);
        without_shield
            .board
            .add_piece(Side::Black, Piece::Queen, Square::D8);
        let no_shield_score = evaluate(&without_shield);

        assert!(
            shield_score > no_shield_score,
            "King with queenside pawn shield should score higher: {} vs {}",
            shield_score,
            no_shield_score
        );
    }
}

#[cfg(test)]
mod complex_positions {
    use super::*;

    #[test]
    fn test_material_vs_positional_tradeoff() {
        // Material advantage (extra piece)
        let mut material = position_with_kings(Side::White);
        material
            .board
            .add_piece(Side::White, Piece::Knight, Square::B1);
        let material_score = evaluate(&material);

        // Positional advantage (advanced passed pawn near promotion)
        let mut positional = position_with_kings(Side::White);
        positional
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E7);
        let positional_score = evaluate(&positional);

        // Material should generally be worth more than most positional factors
        assert!(
            material_score > positional_score,
            "Material advantage should typically outweigh positional: {} vs {}",
            material_score,
            positional_score
        );
    }

    #[test]
    fn test_complex_middlegame_position() {
        // 8 | r . . q k . . . |
        // 7 | . . . . . . . . |
        // 6 | . . n . . . . . |
        // 5 | . . b . p . . . |
        // 4 | . . B . P . . . |
        // 3 | . . . . . N . . |
        // 2 | . . . . . . . . |
        // 1 | R . . Q K . . . |
        //   +------------------+
        //     a b c d e f g h

        let mut position = position_with_kings(Side::White);

        // White pieces
        position
            .board
            .add_piece(Side::White, Piece::Rook, Square::A1);
        position
            .board
            .add_piece(Side::White, Piece::Knight, Square::F3);
        position
            .board
            .add_piece(Side::White, Piece::Bishop, Square::C4);
        position
            .board
            .add_piece(Side::White, Piece::Queen, Square::D1);
        position
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E4);

        // Black pieces
        position
            .board
            .add_piece(Side::Black, Piece::Rook, Square::A8);
        position
            .board
            .add_piece(Side::Black, Piece::Knight, Square::C6);
        position
            .board
            .add_piece(Side::Black, Piece::Bishop, Square::C5);
        position
            .board
            .add_piece(Side::Black, Piece::Queen, Square::D8);
        position
            .board
            .add_piece(Side::Black, Piece::Pawn, Square::E5);

        let score = evaluate(&position);

        // Should be roughly equal (material is balanced, slight positional differences)
        assert!(
            score.abs() < 150,
            "Balanced position should have score close to 0, got {}",
            score
        );
    }

    #[test]
    fn test_rook_endgame() {
        // 8 | r . . . k . . . |
        // 7 | . . . . . . . p |
        // 6 | . . . . . . . . |
        // 5 | . . . . . . . . |
        // 4 | . . . . . . . . |
        // 3 | . . . . . . . . |
        // 2 | . . . . . . . P |
        // 1 | R . . . K . . . |
        //   +------------------+
        //     a b c d e f g h

        let mut position = position_with_kings(Side::White);

        // Both sides have rook and pawns
        position
            .board
            .add_piece(Side::White, Piece::Rook, Square::A1);
        position
            .board
            .add_piece(Side::White, Piece::Pawn, Square::H2);

        position
            .board
            .add_piece(Side::Black, Piece::Rook, Square::A8);
        position
            .board
            .add_piece(Side::Black, Piece::Pawn, Square::H7);

        let score = evaluate(&position);

        assert!(
            score.abs() == 0,
            "Equal rook endgame should give an equal score, got {}",
            score
        );
    }

    #[test]
    fn test_evaluation_symmetry() {
        // 8 | . . . . k . . . |
        // 7 | . . . . . . . . |
        // 6 | . . . . . . . . |
        // 5 | . . . . . . . . |
        // 4 | . . . N P . . . |
        // 3 | . . . . . . . . |
        // 2 | . . . . . . . . |
        // 1 | . . . . K . . . |
        //   +------------------+
        //     a b c d e f g h

        // Create identical positions but mirror them
        let mut white_setup = position_with_kings(Side::White);
        white_setup
            .board
            .add_piece(Side::White, Piece::Knight, Square::D4);
        white_setup
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E4);
        let white_score = evaluate(&white_setup);

        // 8 | . . . . k . . . |
        // 7 | . . . . . . . . |
        // 6 | . . . . . . . . |
        // 5 | . . . n p . . . |
        // 4 | . . . . . . . . |
        // 3 | . . . . . . . . |
        // 2 | . . . . . . . . |
        // 1 | . . . . K . . . |
        //   +------------------+
        //     a b c d e f g h

        let mut black_setup = position_with_kings(Side::White);
        // Mirror the position for black
        black_setup
            .board
            .add_piece(Side::Black, Piece::Knight, Square::D5);
        black_setup
            .board
            .add_piece(Side::Black, Piece::Pawn, Square::E5);
        let black_score = evaluate(&black_setup);

        assert!(
            white_score == -black_score,
            "White ({}) should equal black ({}) score",
            white_score,
            black_score
        );
    }

    #[test]
    fn test_promotion_square_evaluation() {
        // 8 | . . . . k . . . |
        // 7 | . . . . P . . . |
        // 6 | . . . . . . . . |
        // 5 | . . . . . . . . |
        // 4 | . . . . . . . . |
        // 3 | . . . . . . . . |
        // 2 | . . . . . . . . |
        // 1 | . . . . K . . . |
        //   +------------------+
        //     a b c d e f g h

        // Pawn one square from promotion
        let mut near_promotion = position_with_kings(Side::White);
        near_promotion
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E7);
        let near_score = evaluate(&near_promotion);

        // 8 | . . . . k . . . |
        // 7 | . . . . . . . . |
        // 6 | . . . . . . . . |
        // 5 | . . . . . . . . |
        // 4 | . . . . . . . . |
        // 3 | . . . . . . . . |
        // 2 | . . . . P . . . |
        // 1 | . . . . K . . . |
        //   +------------------+
        //     a b c d e f g h

        // Pawn far from promotion
        let mut far_promotion = position_with_kings(Side::White);
        far_promotion
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E2);
        let far_score = evaluate(&far_promotion);

        assert!(
            near_score > far_score + 50,
            "Pawn near promotion should be significantly more valuable: {} vs {}",
            near_score,
            far_score
        );
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_unmoved_kings_only_position() {
        let position = position_with_kings(Side::White);
        let score = evaluate(&position);

        assert!(score == 0, "Kings-only position should be 0, got {}", score);
    }

    #[test]
    fn test_many_pieces_position() {
        let mut position = position_with_kings(Side::White);

        // Add many pieces for both sides
        for square in [Square::A1, Square::B1, Square::C1, Square::D1] {
            position.board.add_piece(Side::White, Piece::Rook, square);
        }
        for square in [Square::A8, Square::B8, Square::C8, Square::D8] {
            position.board.add_piece(Side::Black, Piece::Rook, square);
        }

        let score = evaluate(&position);

        // Should still be roughly balanced
        assert!(
            score.abs() < 200,
            "Position with many equal pieces should be balanced, got {}",
            score
        );
    }

    #[test]
    fn test_multiple_queens_after_promotion() {
        let mut position = position_with_kings(Side::White);

        // Multiple queens (possible after pawn promotion)
        position
            .board
            .add_piece(Side::White, Piece::Queen, Square::D1);
        position
            .board
            .add_piece(Side::White, Piece::Queen, Square::D2);

        position
            .board
            .add_piece(Side::Black, Piece::Queen, Square::D8);
        position
            .board
            .add_piece(Side::Black, Piece::Queen, Square::D7);

        let score = evaluate(&position);

        // Should handle multiple queens correctly
        assert!(
            score.abs() < 150,
            "Equal number of queens should be balanced, got {}",
            score
        );
    }

    #[test]
    fn test_evaluation_consistency() {
        // Create a position with some pieces
        let mut position = position_with_kings(Side::White);
        position
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E4);
        position
            .board
            .add_piece(Side::Black, Piece::Knight, Square::F6);

        // Evaluate the same position multiple times
        let score1 = evaluate(&position);
        let score2 = evaluate(&position);
        let score3 = evaluate(&position);

        assert_eq!(
            score1, score2,
            "Evaluation should be deterministic: {} vs {}",
            score1, score2
        );
        assert_eq!(
            score2, score3,
            "Evaluation should be deterministic: {} vs {}",
            score2, score3
        );
    }

    #[test]
    fn test_all_piece_types_evaluation() {
        let mut position = position_with_kings(Side::White);

        // Add one of each piece type for white
        position
            .board
            .add_piece(Side::White, Piece::Pawn, Square::E4);
        position
            .board
            .add_piece(Side::White, Piece::Knight, Square::F3);
        position
            .board
            .add_piece(Side::White, Piece::Bishop, Square::C4);
        position
            .board
            .add_piece(Side::White, Piece::Rook, Square::A1);
        position
            .board
            .add_piece(Side::White, Piece::Queen, Square::D1);

        let score = evaluate(&position);

        // With all pieces, white should have significant advantage
        assert!(
            score > 500,
            "Position with all white pieces should have large positive score, got {}",
            score
        );
    }
}
