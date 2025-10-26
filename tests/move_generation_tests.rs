/// Unit tests for move generation module
mod test_utils;

use chess_engine::{
    position::Position,
    time::TimeManager,
    types::{Board, Piece, Side, Square},
};
use test_utils::*;

fn empty_position_with_kings(side_to_move: Side) -> Position {
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
    position.ply_from_start_of_game = 0;
    position.fifty = 0;

    reset_move_state(&mut position);
    position
}

#[test]
fn initial_position_generates_20_white_moves() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());
    reset_move_state(&mut position);

    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert_eq!(moves.len(), 20);
    assert!(moves.contains(&(Square::E2, Square::E4)));
    assert!(moves.contains(&(Square::G1, Square::F3)));
}

#[test]
fn initial_position_generates_20_black_moves() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());
    reset_move_state(&mut position);

    position.generate_moves_and_captures(Side::Black);

    let moves = move_pairs(&position);
    assert_eq!(moves.len(), 20);
    assert!(moves.contains(&(Square::E7, Square::E5)));
    assert!(moves.contains(&(Square::G8, Square::F6)));
}

#[test]
fn capture_generation_only_returns_enemy_targets() {
    let mut position = empty_position_with_kings(Side::White);

    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::D4);
    position
        .board
        .add_piece(Side::White, Piece::Knight, Square::C3);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::D6);
    position
        .board
        .add_piece(Side::Black, Piece::Bishop, Square::G4);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::B5);

    reset_move_state(&mut position);
    position.generate_captures(Side::White);

    position.display_board(false);

    let moves = move_pairs(&position);
    let expected = vec![
        (Square::D4, Square::D6),
        (Square::D4, Square::G4),
        (Square::C3, Square::B5),
    ];

    assert_eq!(moves.len(), expected.len());

    for expected_move in &expected {
        assert!(moves.contains(expected_move));
    }

    for &(_, to) in &moves {
        assert_ne!(position.board.value[to as usize], Piece::Empty);
    }
}

#[test]
fn fen_arbitrary_bishop_captures_1() {
    test_fen_captures(
        "k7/8/4n3/1p6/2B5/8/8/6K1 w - - 0 1",
        Side::White,
        vec![(Square::C4, Square::B5), (Square::C4, Square::E6)],
    );
}

#[test]
fn fen_arbitrary_captures_1() {
    test_fen_captures(
        "r1b2r1k/4qp1p/p2ppb1Q/4nP2/1p1NP3/2N5/PPP4P/2KR1BR1 w - - 0 1",
        Side::White,
        vec![
            (Square::D4, Square::E6),
            (Square::F1, Square::A6),
            (Square::F5, Square::E6),
            (Square::H6, Square::F8),
            (Square::H6, Square::F6),
            (Square::H6, Square::H7),
        ],
    );

    test_fen_captures(
        "r1b2r1k/4qp1p/p2ppb1Q/4nP2/1p1NP3/2N5/PPP4P/2KR1BR1 w - - 0 1",
        Side::Black,
        vec![(Square::E6, Square::F5), (Square::B4, Square::C3)],
    );
}

#[test]
fn fen_arbitrary_captures_2() {
    test_fen_captures(
        "r4k1r/1b2bPR1/p4n2/3p4/4P2P/1q2B2B/PpP5/1K4R1 w - - 0 1",
        Side::White,
        vec![
            (Square::C2, Square::B3),
            (Square::E4, Square::D5),
            (Square::A2, Square::B3),
            (Square::B1, Square::B2),
        ],
    );

    test_fen_captures(
        "r4k1r/1b2bPR1/p4n2/3p4/4P2P/1q2B2B/PpP5/1K4R1 w - - 0 1",
        Side::Black,
        vec![
            (Square::D5, Square::E4),
            (Square::F6, Square::E4),
            (Square::H8, Square::H4),
            (Square::B3, Square::A2),
            (Square::B3, Square::C2),
            (Square::B3, Square::E3),
            (Square::F8, Square::F7),
            (Square::F8, Square::G7),
        ],
    );
}

#[test]
fn fen_pawn_captures_1() {
    test_fen_captures(
        "3k4/8/8/pppppppp/PPPPPPPP/8/8/3K4 w - - 0 1",
        Side::White,
        vec![
            (Square::A4, Square::B5),
            (Square::B4, Square::A5),
            (Square::B4, Square::C5),
            (Square::C4, Square::B5),
            (Square::C4, Square::D5),
            (Square::D4, Square::C5),
            (Square::D4, Square::E5),
            (Square::E4, Square::D5),
            (Square::E4, Square::F5),
            (Square::F4, Square::E5),
            (Square::F4, Square::G5),
            (Square::G4, Square::F5),
            (Square::G4, Square::H5),
            (Square::H4, Square::G5),
        ],
    );

    test_fen_captures(
        "3k4/8/8/pppppppp/PPPPPPPP/8/8/3K4 w - - 0 1",
        Side::Black,
        vec![
            (Square::A5, Square::B4),
            (Square::B5, Square::A4),
            (Square::B5, Square::C4),
            (Square::C5, Square::B4),
            (Square::C5, Square::D4),
            (Square::D5, Square::C4),
            (Square::D5, Square::E4),
            (Square::E5, Square::D4),
            (Square::E5, Square::F4),
            (Square::F5, Square::E4),
            (Square::F5, Square::G4),
            (Square::G5, Square::F4),
            (Square::G5, Square::H4),
            (Square::H5, Square::G4),
        ],
    );
}

#[test]
fn castling_moves_included_when_rights_available() {
    let mut position = empty_position_with_kings(Side::White);

    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::H1);
    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::A1);
    position.castle = 0b0011;

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E1, Square::G1)));
    assert!(moves.contains(&(Square::E1, Square::C1)));
}

#[test]
fn castling_moves_excluded_when_rights_unavailable() {
    let mut position = empty_position_with_kings(Side::White);

    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::H1);
    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::A1);
    position.castle = 0;

    reset_move_state(&mut position);

    // Uncomment to visualize the board state during debugging:
    // position.display_board(false);

    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);

    // Verify castling moves are not included
    assert!(!moves.contains(&(Square::E1, Square::G1))); // No kingside castle
    assert!(!moves.contains(&(Square::E1, Square::C1))); // No queenside castle

    // The king on E1 should have exactly 5 regular moves to empty adjacent squares
    let expected_king_moves = vec![
        (Square::E1, Square::D1), // Left
        (Square::E1, Square::D2), // Diagonal up-left
        (Square::E1, Square::E2), // Up
        (Square::E1, Square::F2), // Diagonal up-right
        (Square::E1, Square::F1), // Right
    ];

    // Verify each expected move exists
    for expected_move in &expected_king_moves {
        assert!(
            moves.contains(expected_move),
            "Expected king move {:?} not found. Generated moves: {:?}",
            expected_move,
            moves
        );
    }

    print!("{:?}", moves);

    // Count only king moves
    let king_moves: Vec<_> = moves
        .iter()
        .filter(|(from, _)| *from == Square::E1)
        .collect();

    assert_eq!(
        king_moves.len(),
        5,
        "Expected exactly 5 king moves, but got {}. King moves: {:?}",
        king_moves.len(),
        king_moves
    );
}

#[test]
fn pawn_promotion_moves_include_forward_and_capture() {
    let mut position = empty_position_with_kings(Side::White);

    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::A7);
    position
        .board
        .add_piece(Side::Black, Piece::Knight, Square::B8);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::A7, Square::A8)));
    assert!(moves.contains(&(Square::A7, Square::B8)));
}

#[test]
fn blocked_pawn_cannot_advance() {
    let mut position = empty_position_with_kings(Side::White);

    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::E2);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::E3);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(!moves.contains(&(Square::E2, Square::E3)));
    assert!(!moves.contains(&(Square::E2, Square::E4)));
}

// ============================================================================
// PAWN MOVE GENERATION TESTS
// ============================================================================

#[test]
fn white_pawn_single_push_from_starting_position() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::E2);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E2, Square::E3)));
    assert!(moves.contains(&(Square::E2, Square::E4)));
}

#[test]
fn black_pawn_single_push_from_starting_position() {
    let mut position = empty_position_with_kings(Side::Black);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::E7);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::Black);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E7, Square::E6)));
    assert!(moves.contains(&(Square::E7, Square::E5)));
}

#[test]
fn pawn_double_push_blocked() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::E2);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::E4);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E2, Square::E3)));
    assert!(!moves.contains(&(Square::E2, Square::E4)));
}

#[test]
fn pawn_cannot_double_push_from_non_starting_rank() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::E3);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E3, Square::E4)));
    assert!(!moves.contains(&(Square::E3, Square::E5)));
}

#[test]
fn pawn_diagonal_captures() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::E4);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::D5);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::F5);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E4, Square::D5)));
    assert!(moves.contains(&(Square::E4, Square::F5)));
    assert!(moves.contains(&(Square::E4, Square::E5)));
}

#[test]
fn pawn_cannot_capture_own_pieces() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::E4);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::D5);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::F5);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(!moves.contains(&(Square::E4, Square::D5)));
    assert!(!moves.contains(&(Square::E4, Square::F5)));
    assert!(moves.contains(&(Square::E4, Square::E5)));
}

#[test]
fn pawn_edge_file_captures() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::A4);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::B5);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::A4, Square::B5)));
    assert!(moves.contains(&(Square::A4, Square::A5)));
}

#[test]
fn pawn_promotion_all_pieces() {
    let mut position = empty_position_with_kings(Side::White);
    // Move black king out of the way
    position
        .board
        .remove_piece(Side::Black, Piece::King, Square::E8);
    position
        .board
        .add_piece(Side::Black, Piece::King, Square::A8);

    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::E7);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);

    // Pawn on 7th rank should be able to move to 8th rank
    // Note: Promotion piece selection happens in make_move, not during generation
    assert!(
        moves.contains(&(Square::E7, Square::E8)),
        "Pawn promotion move not found"
    );
}

#[test]
fn pawn_promotion_with_capture() {
    let mut position = empty_position_with_kings(Side::White);
    // Move kings out of the way
    position
        .board
        .remove_piece(Side::Black, Piece::King, Square::E8);
    position
        .board
        .add_piece(Side::Black, Piece::King, Square::A8);

    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::E7);
    position
        .board
        .add_piece(Side::Black, Piece::Rook, Square::D8);
    position
        .board
        .add_piece(Side::Black, Piece::Rook, Square::F8);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E7, Square::E8)));
    assert!(moves.contains(&(Square::E7, Square::D8)));
    assert!(moves.contains(&(Square::E7, Square::F8)));
}

#[test]
fn black_pawn_promotion() {
    let mut position = empty_position_with_kings(Side::Black);
    // Move white king out of the way
    position
        .board
        .remove_piece(Side::White, Piece::King, Square::E1);
    position
        .board
        .add_piece(Side::White, Piece::King, Square::A1);

    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::E2);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::Black);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E2, Square::E1)));
}

#[test]
fn white_pawn_on_7th_rank_can_only_promote() {
    let mut position = empty_position_with_kings(Side::White);
    // Move black king out of the way
    position
        .board
        .remove_piece(Side::Black, Piece::King, Square::E8);
    position
        .board
        .add_piece(Side::Black, Piece::King, Square::H8);

    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::D7);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    let pawn_moves: Vec<_> = moves
        .iter()
        .filter(|(from, _)| *from == Square::D7)
        .collect();

    // Should only have one forward move (to 8th rank for promotion)
    assert_eq!(pawn_moves.len(), 4);
    assert!(moves.contains(&(Square::D7, Square::D8)));
}

// ============================================================================
// KNIGHT MOVE GENERATION TESTS
// ============================================================================

#[test]
fn knight_generates_all_moves_from_center() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Knight, Square::E4);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    let expected_knight_moves = vec![
        (Square::E4, Square::D6),
        (Square::E4, Square::F6),
        (Square::E4, Square::G5),
        (Square::E4, Square::G3),
        (Square::E4, Square::F2),
        (Square::E4, Square::D2),
        (Square::E4, Square::C3),
        (Square::E4, Square::C5),
    ];

    for expected_move in &expected_knight_moves {
        assert!(
            moves.contains(expected_move),
            "Expected knight move {:?} not found",
            expected_move
        );
    }
}

#[test]
fn knight_moves_from_corner() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Knight, Square::A1);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    let expected_moves = vec![(Square::A1, Square::B3), (Square::A1, Square::C2)];

    for expected_move in &expected_moves {
        assert!(moves.contains(expected_move));
    }
}

#[test]
fn knight_cannot_capture_own_pieces() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Knight, Square::E4);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::D6);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::F6);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(!moves.contains(&(Square::E4, Square::D6)));
    assert!(!moves.contains(&(Square::E4, Square::F6)));
}

#[test]
fn knight_can_capture_enemy_pieces() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Knight, Square::E4);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::D6);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::F6);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E4, Square::D6)));
    assert!(moves.contains(&(Square::E4, Square::F6)));
}

// ============================================================================
// BISHOP MOVE GENERATION TESTS
// ============================================================================

#[test]
fn bishop_generates_diagonal_moves() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Bishop, Square::D4);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);

    // Northeast diagonal
    assert!(moves.contains(&(Square::D4, Square::E5)));
    assert!(moves.contains(&(Square::D4, Square::F6)));
    assert!(moves.contains(&(Square::D4, Square::G7)));
    assert!(moves.contains(&(Square::D4, Square::H8)));

    // Northwest diagonal
    assert!(moves.contains(&(Square::D4, Square::C5)));
    assert!(moves.contains(&(Square::D4, Square::B6)));
    assert!(moves.contains(&(Square::D4, Square::A7)));

    // Southeast diagonal
    assert!(moves.contains(&(Square::D4, Square::E3)));
    assert!(moves.contains(&(Square::D4, Square::F2)));
    assert!(moves.contains(&(Square::D4, Square::G1)));

    // Southwest diagonal
    assert!(moves.contains(&(Square::D4, Square::C3)));
    assert!(moves.contains(&(Square::D4, Square::B2)));
    assert!(moves.contains(&(Square::D4, Square::A1)));
}

#[test]
fn bishop_blocked_by_own_pieces() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Bishop, Square::D4);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::E5);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::C3);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);

    // Should not move through or capture own pieces
    assert!(!moves.contains(&(Square::D4, Square::E5)));
    assert!(!moves.contains(&(Square::D4, Square::F6)));
    assert!(!moves.contains(&(Square::D4, Square::C3)));
    assert!(!moves.contains(&(Square::D4, Square::B2)));

    // Should still have other diagonal moves
    assert!(moves.contains(&(Square::D4, Square::C5)));
    assert!(moves.contains(&(Square::D4, Square::E3)));
}

#[test]
fn bishop_captures_enemy_piece_but_cannot_move_beyond() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Bishop, Square::D4);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::F6);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);

    // Can capture
    assert!(moves.contains(&(Square::D4, Square::F6)));
    // Cannot move beyond
    assert!(!moves.contains(&(Square::D4, Square::G7)));
}

// ============================================================================
// ROOK MOVE GENERATION TESTS
// ============================================================================

#[test]
fn rook_generates_horizontal_and_vertical_moves() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::D4);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);

    // Vertical up
    assert!(moves.contains(&(Square::D4, Square::D5)));
    assert!(moves.contains(&(Square::D4, Square::D6)));
    assert!(moves.contains(&(Square::D4, Square::D7)));
    assert!(moves.contains(&(Square::D4, Square::D8)));

    // Vertical down
    assert!(moves.contains(&(Square::D4, Square::D3)));
    assert!(moves.contains(&(Square::D4, Square::D2)));
    assert!(moves.contains(&(Square::D4, Square::D1)));

    // Horizontal right
    assert!(moves.contains(&(Square::D4, Square::E4)));
    assert!(moves.contains(&(Square::D4, Square::F4)));
    assert!(moves.contains(&(Square::D4, Square::G4)));
    assert!(moves.contains(&(Square::D4, Square::H4)));

    // Horizontal left
    assert!(moves.contains(&(Square::D4, Square::C4)));
    assert!(moves.contains(&(Square::D4, Square::B4)));
    assert!(moves.contains(&(Square::D4, Square::A4)));
}

#[test]
fn rook_blocked_by_own_pieces() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::D4);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::D6);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::F4);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);

    assert!(!moves.contains(&(Square::D4, Square::D6)));
    assert!(!moves.contains(&(Square::D4, Square::D7)));
    assert!(!moves.contains(&(Square::D4, Square::F4)));
    assert!(!moves.contains(&(Square::D4, Square::G4)));

    assert!(moves.contains(&(Square::D4, Square::D5)));
    assert!(moves.contains(&(Square::D4, Square::E4)));
}

#[test]
fn rook_captures_enemy_but_cannot_move_beyond() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::D4);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::D6);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);

    assert!(moves.contains(&(Square::D4, Square::D6)));
    assert!(!moves.contains(&(Square::D4, Square::D7)));
}

// ============================================================================
// QUEEN MOVE GENERATION TESTS
// ============================================================================

#[test]
fn queen_generates_rook_and_bishop_moves() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Queen, Square::D4);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);

    // Should have rook-like moves
    assert!(moves.contains(&(Square::D4, Square::D8)));
    assert!(moves.contains(&(Square::D4, Square::A4)));

    // Should have bishop-like moves
    assert!(moves.contains(&(Square::D4, Square::H8)));
    assert!(moves.contains(&(Square::D4, Square::A1)));
}

#[test]
fn queen_blocked_on_all_directions() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Queen, Square::D4);

    // Block all directions with own pieces
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::D5);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::E5);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::E4);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::E3);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::D3);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::C3);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::C4);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::C5);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    let queen_moves: Vec<_> = moves
        .iter()
        .filter(|(from, _)| *from == Square::D4)
        .collect();

    assert_eq!(
        queen_moves.len(),
        0,
        "Queen should have no moves when surrounded"
    );
}

// ============================================================================
// KING MOVE GENERATION TESTS
// ============================================================================

#[test]
fn king_generates_all_adjacent_moves() {
    let mut position = empty_position_with_kings(Side::White);
    // Remove default king and place at D4
    position
        .board
        .remove_piece(Side::White, Piece::King, Square::E1);
    position
        .board
        .add_piece(Side::White, Piece::King, Square::D4);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    let expected_moves = vec![
        (Square::D4, Square::C5),
        (Square::D4, Square::D5),
        (Square::D4, Square::E5),
        (Square::D4, Square::C4),
        (Square::D4, Square::E4),
        (Square::D4, Square::C3),
        (Square::D4, Square::D3),
        (Square::D4, Square::E3),
    ];

    for expected_move in &expected_moves {
        assert!(
            moves.contains(expected_move),
            "Expected king move {:?} not found",
            expected_move
        );
    }
}

#[test]
fn king_cannot_capture_own_pieces() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .remove_piece(Side::White, Piece::King, Square::E1);
    position
        .board
        .add_piece(Side::White, Piece::King, Square::D4);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::D5);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::E4);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(!moves.contains(&(Square::D4, Square::D5)));
    assert!(!moves.contains(&(Square::D4, Square::E4)));
}

#[test]
fn king_can_capture_enemy_pieces() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .remove_piece(Side::White, Piece::King, Square::E1);
    position
        .board
        .add_piece(Side::White, Piece::King, Square::D4);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::D5);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::E4);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::D4, Square::D5)));
    assert!(moves.contains(&(Square::D4, Square::E4)));
}

#[test]
fn king_moves_from_corner() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .remove_piece(Side::White, Piece::King, Square::E1);
    position
        .board
        .add_piece(Side::White, Piece::King, Square::A1);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    let expected_moves = vec![
        (Square::A1, Square::A2),
        (Square::A1, Square::B2),
        (Square::A1, Square::B1),
    ];

    for expected_move in &expected_moves {
        assert!(moves.contains(expected_move));
    }
}

// ============================================================================
// CASTLING TESTS
// ============================================================================

#[test]
fn white_kingside_castle_clear_path() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::H1);
    position.castle = 0b0001; // White kingside only

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E1, Square::G1)));
}

#[test]
fn white_queenside_castle_clear_path() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::A1);
    position.castle = 0b0010; // White queenside only

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E1, Square::C1)));
}

#[test]
fn black_kingside_castle_clear_path() {
    let mut position = empty_position_with_kings(Side::Black);
    position
        .board
        .add_piece(Side::Black, Piece::Rook, Square::H8);
    position.castle = 0b0100; // Black kingside only

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::Black);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E8, Square::G8)));
}

#[test]
fn black_queenside_castle_clear_path() {
    let mut position = empty_position_with_kings(Side::Black);
    position
        .board
        .add_piece(Side::Black, Piece::Rook, Square::A8);
    position.castle = 0b1000; // Black queenside only

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::Black);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E8, Square::C8)));
}

#[test]
fn castle_blocked_by_piece() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::H1);
    position
        .board
        .add_piece(Side::White, Piece::Knight, Square::F1);
    position.castle = 0b0001;

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(!moves.contains(&(Square::E1, Square::G1)));
}

#[test]
fn both_castles_available() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::H1);
    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::A1);
    position.castle = 0b0011; // Both white castles

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::E1, Square::G1)));
    assert!(moves.contains(&(Square::E1, Square::C1)));
}

// ============================================================================
// EN PASSANT TESTS
// ============================================================================
// Note: En passant tests are complex because they require proper game history.
// The en_passant generation function checks game_list[ply_from_start_of_game - 1]
// which requires the game state to be set up correctly through make_move.
// These tests are commented out pending refactoring of the test approach.

// TODO: Refactor these tests to properly set up en passant scenarios
// The challenge is that reset_move_state() clears ply but we need ply_from_start_of_game
// to be preserved for en passant detection.

#[test]
fn en_passant_capture_after_double_pawn_push() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // This test demonstrates en passant after a double pawn push
    // but requires careful coordination between make_move and generate_moves
    assert!(position.make_move(Square::E2, Square::E4));
    assert!(position.make_move(Square::A7, Square::A6));
    assert!(position.make_move(Square::E4, Square::E5));
    assert!(position.make_move(Square::D7, Square::D5));

    // Reset move list but preserve game history
    position.ply = 0;
    position.first_move.iter_mut().for_each(|entry| *entry = -1);
    position.first_move[0] = 0;
    position.move_list.iter_mut().for_each(|slot| *slot = None);

    position.generate_moves_and_captures(position.side);
    let moves = move_pairs(&position);

    assert!(moves.contains(&(Square::E5, Square::D6)));
}

#[test]
fn black_en_passant_capture() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // White pawn advances two squares
    assert!(position.make_move(Square::A2, Square::A3));
    assert!(position.make_move(Square::E7, Square::E5));
    assert!(position.make_move(Square::A3, Square::A4));
    assert!(position.make_move(Square::E5, Square::E4));

    // White pawn on d2 advances to d4, black can capture en passant
    assert!(position.make_move(Square::D2, Square::D4));

    // Reset move list but preserve game history
    position.ply = 0;
    position.first_move.iter_mut().for_each(|entry| *entry = -1);
    position.first_move[0] = 0;
    position.move_list.iter_mut().for_each(|slot| *slot = None);

    position.generate_moves_and_captures(position.side);
    let moves = move_pairs(&position);

    assert!(moves.contains(&(Square::E4, Square::D3)));
}

#[test]
fn en_passant_in_captures_only() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    assert!(position.make_move(Square::E2, Square::E4));
    assert!(position.make_move(Square::A7, Square::A6));
    assert!(position.make_move(Square::E4, Square::E5));
    assert!(position.make_move(Square::D7, Square::D5));

    // Reset move list but preserve game history
    position.ply = 0;
    position.first_move.iter_mut().for_each(|entry| *entry = -1);
    position.first_move[0] = 0;
    position.move_list.iter_mut().for_each(|slot| *slot = None);

    position.generate_captures(position.side);
    let moves = move_pairs(&position);

    // En passant should appear in capture generation
    assert!(moves.contains(&(Square::E5, Square::D6)));
}

#[test]
fn en_passant_both_sides_corrected() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Set up a scenario where black has pawns on both sides
    assert!(position.make_move(Square::E2, Square::E4));
    assert!(position.make_move(Square::D7, Square::D5));
    assert!(position.make_move(Square::E4, Square::E5));
    assert!(position.make_move(Square::D5, Square::D4));
    assert!(position.make_move(Square::A2, Square::A4));
    assert!(position.make_move(Square::D4, Square::D3));
    assert!(position.make_move(Square::B2, Square::B4));
    assert!(position.make_move(Square::A7, Square::A5));

    // Now white has pawn on b4, black just moved a7-a5
    // Black pawn on a5 can be captured en passant by white pawn on b4

    // Reset move list but preserve game history
    position.ply = 0;
    position.first_move.iter_mut().for_each(|entry| *entry = -1);
    position.first_move[0] = 0;
    position.move_list.iter_mut().for_each(|slot| *slot = None);

    position.generate_moves_and_captures(position.side);
    let moves = move_pairs(&position);

    assert!(moves.contains(&(Square::B4, Square::A5)));
}

// ============================================================================
// CAPTURE GENERATION TESTS
// ============================================================================

#[test]
fn generate_captures_includes_only_captures() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::D4);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::D6);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::F4);

    reset_move_state(&mut position);
    position.generate_captures(Side::White);

    let moves = move_pairs(&position);

    // Should include captures
    assert!(moves.contains(&(Square::D4, Square::D6)));
    assert!(moves.contains(&(Square::D4, Square::F4)));

    // Should not include non-captures
    assert!(!moves.contains(&(Square::D4, Square::D5)));
    assert!(!moves.contains(&(Square::D4, Square::E4)));
}

#[test]
fn generate_captures_multiple_pieces() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::D4);
    position
        .board
        .add_piece(Side::White, Piece::Knight, Square::C3);
    position
        .board
        .add_piece(Side::Black, Piece::Pawn, Square::D6);
    position
        .board
        .add_piece(Side::Black, Piece::Bishop, Square::B5);

    reset_move_state(&mut position);
    position.generate_captures(Side::White);

    let moves = move_pairs(&position);
    assert!(moves.contains(&(Square::D4, Square::D6)));
    assert!(moves.contains(&(Square::C3, Square::B5)));
}

#[test]
fn generate_captures_pawn_promotions() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::D7);
    position
        .board
        .add_piece(Side::Black, Piece::Rook, Square::E8);

    reset_move_state(&mut position);
    position.generate_captures(Side::White);

    let moves = move_pairs(&position);
    // Capturing promotion
    assert!(moves.contains(&(Square::D7, Square::E8)));
}

// ============================================================================
// EDGE CASES AND COMPLEX SCENARIOS
// ============================================================================

#[test]
fn multiple_pieces_same_type_generate_independently() {
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Knight, Square::B1);
    position
        .board
        .add_piece(Side::White, Piece::Knight, Square::G1);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);

    // Both knights should have moves
    let knight_b1_moves: Vec<_> = moves
        .iter()
        .filter(|(from, _)| *from == Square::B1)
        .collect();
    let knight_g1_moves: Vec<_> = moves
        .iter()
        .filter(|(from, _)| *from == Square::G1)
        .collect();

    assert!(knight_b1_moves.len() > 0);
    assert!(knight_g1_moves.len() > 0);
}

#[test]
fn empty_board_except_kings_generates_king_moves_only() {
    let mut position = empty_position_with_kings(Side::White);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);

    // All moves should be from the king
    for (from, _) in &moves {
        assert_eq!(*from, Square::E1, "Only king should have moves");
    }
}

#[test]
fn pinned_piece_scenarios() {
    // Test that move generation doesn't filter out illegal moves due to pins
    // (This would be handled by make_move validation)
    let mut position = empty_position_with_kings(Side::White);
    position
        .board
        .add_piece(Side::White, Piece::Bishop, Square::E2);
    position
        .board
        .add_piece(Side::Black, Piece::Rook, Square::E8);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);

    // Bishop should generate moves even though it's pinned
    // (move legality is checked when the move is made)
    let bishop_moves: Vec<_> = moves
        .iter()
        .filter(|(from, _)| *from == Square::E2)
        .collect();

    assert!(bishop_moves.len() > 0, "Bishop should generate moves");
}

#[test]
fn all_pieces_blocked_generates_only_king_moves() {
    let mut position = empty_position_with_kings(Side::White);

    // Surround white pieces with own pawns - but knights can still jump
    position
        .board
        .add_piece(Side::White, Piece::Rook, Square::A1);
    position
        .board
        .add_piece(Side::White, Piece::Knight, Square::B1);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::A2);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::B2);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::C2);

    // Block knight destinations
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::A3);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::C3);
    position
        .board
        .add_piece(Side::White, Piece::Pawn, Square::D2);

    reset_move_state(&mut position);
    position.generate_moves_and_captures(Side::White);

    let moves = move_pairs(&position);

    // Rook should have no moves (blocked)
    let rook_moves: Vec<_> = moves
        .iter()
        .filter(|(from, _)| *from == Square::A1)
        .collect();

    // Knight should have no moves (all destinations blocked by own pieces)
    let knight_moves: Vec<_> = moves
        .iter()
        .filter(|(from, _)| *from == Square::B1)
        .collect();

    assert_eq!(rook_moves.len(), 0, "Blocked rook should have no moves");
    assert_eq!(knight_moves.len(), 0, "Blocked knight should have no moves");
}
