#![allow(dead_code)]

use std::sync::Once;

/// Shared test utilities for hash-related tests
use chess_engine::{
    engine::Engine,
    position::Position,
    types::{Move, Piece, Side, Square},
    zobrist_hash::initialize_zobrist_hash_tables,
};

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

pub fn ensure_zobrist_initialized() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        initialize_zobrist_hash_tables();
    });
}

pub fn reset_move_state(position: &mut Position) {
    position.ply = 0;
    position.first_move.iter_mut().for_each(|entry| *entry = -1);
    position.first_move[0] = 0;
    position.move_list.iter_mut().for_each(|slot| *slot = None);
}

pub fn position_from_fen(fen: &str) -> Position {
    ensure_zobrist_initialized();
    let mut position = Position::from_fen(fen).expect(&format!("Failed to load FEN: {}", fen));
    position.set_material_scores();
    reset_move_state(&mut position);
    position.display_board(false);
    position
}

pub fn engine_from_fen(fen: &str, depth: u16) -> Engine {
    ensure_zobrist_initialized();
    let mut engine = Engine::new(None, None, None, None, None, Some(depth));
    engine.position = Position::from_fen(fen).expect(&format!("Failed to load FEN: {}", fen));
    engine.position.set_material_scores();
    reset_move_state(&mut engine.position);
    engine.position.display_board(false);
    engine
}

pub fn move_pairs(position: &Position) -> Vec<(Square, Square)> {
    let start = position
        .first_move
        .get(position.ply)
        .copied()
        .unwrap_or(0)
        .max(0) as usize;

    if let Some(&end_value) = position.first_move.get(position.ply + 1) {
        if end_value >= 0 {
            let end = end_value as usize;
            if end >= start {
                return position.move_list[start..end]
                    .iter()
                    .filter_map(|entry| entry.as_ref().map(|mv| (mv.from, mv.to)))
                    .collect();
            }
        }
    }

    position
        .move_list
        .iter()
        .skip(start)
        .take_while(|entry| entry.is_some())
        .filter_map(|entry| entry.as_ref().map(|mv| (mv.from, mv.to)))
        .collect()
}

pub fn test_fen_captures(fen: &str, side: Side, expected: Vec<(Square, Square)>) {
    let mut position = position_from_fen(fen);
    position.generate_captures(side);

    let moves = move_pairs(&position);

    assert_eq!(
        moves.len(),
        expected.len(),
        "Expected {} captures, got {}. Generated moves: {:?}",
        expected.len(),
        moves.len(),
        moves
    );

    for expected_move in &expected {
        assert!(
            moves.contains(expected_move),
            "Expected capture {:?} not found. Generated moves: {:?}",
            expected_move,
            moves
        );
    }
}
