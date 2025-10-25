mod test_utils;

use chess_engine::{
    position::Position,
    time::TimeManager,
    types::{Piece, Side, Square},
};
use test_utils::*;

#[test]
fn test_load_starting_position_from_fen() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Load the standard starting position
    let result = position.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    assert!(result.is_ok(), "Failed to load starting position FEN");

    // Verify white is to move
    assert_eq!(position.side, Side::White);
    assert_eq!(position.other_side, Side::Black);

    // Verify castling rights (all enabled: 0b1111 = 15)
    assert_eq!(position.castle, 0b1111);

    // Check some piece positions
    assert!(
        position.board.bit_pieces[Side::White as usize][Piece::King as usize]
            .is_bit_set(Square::E1)
    );
    assert!(
        position.board.bit_pieces[Side::Black as usize][Piece::King as usize]
            .is_bit_set(Square::E8)
    );
    assert!(
        position.board.bit_pieces[Side::White as usize][Piece::Pawn as usize]
            .is_bit_set(Square::E2)
    );
    assert!(
        position.board.bit_pieces[Side::Black as usize][Piece::Pawn as usize]
            .is_bit_set(Square::E7)
    );
}

#[test]
fn test_load_custom_position_from_fen() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Load a position with a specific setup (Scandinavian Defense after 1.e4 d5)
    let result = position.from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2");
    assert!(result.is_ok(), "Failed to load custom position FEN");

    // Verify white is to move
    assert_eq!(position.side, Side::White);

    // Check that black pawn is on d5
    assert!(
        position.board.bit_pieces[Side::Black as usize][Piece::Pawn as usize]
            .is_bit_set(Square::D5)
    );

    // Check that white pawn is on e4
    assert!(
        position.board.bit_pieces[Side::White as usize][Piece::Pawn as usize]
            .is_bit_set(Square::E4)
    );

    // Check that e2 is empty (pawn moved to e4)
    assert!(
        !position.board.bit_pieces[Side::White as usize][Piece::Pawn as usize]
            .is_bit_set(Square::E2)
    );
}

#[test]
fn test_load_position_with_black_to_move() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Load a position with black to move
    let result = position.from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
    assert!(result.is_ok(), "Failed to load position with black to move");

    // Verify black is to move
    assert_eq!(position.side, Side::Black);
    assert_eq!(position.other_side, Side::White);
}

#[test]
fn test_load_position_with_limited_castling() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Position where only white can castle kingside and black can castle queenside
    let result = position.from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w Kq - 0 1");
    assert!(
        result.is_ok(),
        "Failed to load position with limited castling"
    );

    // Verify castling rights (K=1, q=8, so 0b1001 = 9)
    assert_eq!(position.castle, 0b1001);
}

#[test]
fn test_load_position_with_no_castling() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Position with no castling rights
    let result = position.from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w - - 0 1");
    assert!(result.is_ok(), "Failed to load position with no castling");

    // Verify no castling rights
    assert_eq!(position.castle, 0);
}

#[test]
fn test_halfmove_clock_parsing() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Position with halfmove clock = 5
    let result = position.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 5 1");
    assert!(result.is_ok(), "Failed to load FEN with halfmove clock");

    // Verify halfmove clock
    assert_eq!(position.fifty, 5, "Halfmove clock should be 5");
}

#[test]
fn test_fullmove_number_parsing_white_to_move() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Position at move 10, white to move
    let result = position.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 10");
    assert!(result.is_ok(), "Failed to load FEN with fullmove number");

    // Verify ply_from_start_of_game
    // Move 10, white to move = (10-1)*2 + 0 = 18 halfmoves
    assert_eq!(
        position.ply_from_start_of_game, 18,
        "Ply from start should be 18"
    );
}

#[test]
fn test_fullmove_number_parsing_black_to_move() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Position at move 10, black to move
    let result = position.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 10");
    assert!(result.is_ok(), "Failed to load FEN with fullmove number");

    // Verify ply_from_start_of_game
    // Move 10, black to move = (10-1)*2 + 1 = 19 halfmoves
    assert_eq!(
        position.ply_from_start_of_game, 19,
        "Ply from start should be 19"
    );
}

#[test]
fn test_default_values_when_fields_missing() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Minimal FEN (only first 3 fields)
    let result = position.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq");
    assert!(
        result.is_ok(),
        "Failed to load minimal FEN (missing optional fields)"
    );

    // Verify defaults
    assert_eq!(position.fifty, 0, "Halfmove clock should default to 0");
    assert_eq!(
        position.ply_from_start_of_game, 0,
        "Ply should default to 0 for white"
    );
}

#[test]
fn test_en_passant_validation_white() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Valid en passant square for white (rank 6)
    let result = position.from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 1");
    assert!(
        result.is_ok(),
        "Should accept valid en passant square on rank 6 for white"
    );

    // Invalid en passant square for white (wrong rank)
    let result2 = position.from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e3 0 1");
    assert!(
        result2.is_err(),
        "Should reject invalid en passant square on rank 3 for white"
    );
}

#[test]
fn test_en_passant_validation_black() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Valid en passant square for black (rank 3)
    let result = position.from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
    assert!(
        result.is_ok(),
        "Should accept valid en passant square on rank 3 for black"
    );

    // Invalid en passant square for black (wrong rank)
    let result2 = position.from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e6 0 1");
    assert!(
        result2.is_err(),
        "Should reject invalid en passant square on rank 6 for black"
    );
}

#[test]
fn test_complete_fen_with_all_fields() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Complete FEN with all 6 fields
    let result =
        position.from_fen("r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq e6 3 3");
    assert!(
        result.is_ok(),
        "Failed to load complete FEN with all fields"
    );

    // Verify all fields
    assert_eq!(position.side, Side::White);
    assert_eq!(position.castle, 0b1111); // KQkq
    assert_eq!(position.fifty, 3);
    assert_eq!(position.ply_from_start_of_game, 4); // (3-1)*2 + 0 = 4

    // Verify some piece positions
    assert!(
        position.board.bit_pieces[Side::Black as usize][Piece::Knight as usize]
            .is_bit_set(Square::C6)
    );
    assert!(
        position.board.bit_pieces[Side::White as usize][Piece::Knight as usize]
            .is_bit_set(Square::F3)
    );
}

#[test]
fn test_invalid_halfmove_clock() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Invalid halfmove clock (not a number)
    let result = position.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - abc 1");
    assert!(
        result.is_err(),
        "Should reject invalid halfmove clock format"
    );
}

#[test]
fn test_invalid_fullmove_number() {
    ensure_zobrist_initialized();
    let mut position = Position::new(TimeManager::default());

    // Invalid fullmove number (not a number)
    let result = position.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 xyz");
    assert!(
        result.is_err(),
        "Should reject invalid fullmove number format"
    );

    // Invalid fullmove number (zero)
    let result2 = position.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 0");
    assert!(
        result2.is_err(),
        "Should reject fullmove number less than 1"
    );
}
