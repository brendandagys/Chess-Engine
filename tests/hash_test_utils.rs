/// Shared test utilities for hash-related tests
use chess_engine::types::{Move, Piece, Side, Square};

/// Create a test move
pub fn create_test_move(from: Square, to: Square) -> Move {
    Move {
        from,
        to,
        promote: None,
        score: 0,
    }
}

/// Get all valid pieces (excluding Empty)
pub fn valid_pieces() -> Vec<Piece> {
    vec![
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ]
}

/// Get all sides
pub fn all_sides() -> Vec<Side> {
    vec![Side::White, Side::Black]
}

/// Sample squares for testing
pub fn sample_squares() -> Vec<Square> {
    vec![
        Square::A1, // Corner
        Square::H1, // Corner
        Square::A8, // Corner
        Square::H8, // Corner
        Square::E4, // Center
        Square::D4, // Center
        Square::E5, // Center
        Square::D5, // Center
    ]
}

/// All en passant files (0-7)
pub fn en_passant_files() -> Vec<u8> {
    (0..8).collect()
}

/// All castle states (0-15)
pub fn castle_states() -> Vec<u8> {
    (0..16).collect()
}
