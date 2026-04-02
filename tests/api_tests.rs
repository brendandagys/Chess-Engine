use chess_engine::api;

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[test]
fn get_legal_moves_starting_position() {
    let moves = api::get_legal_moves(START_FEN).unwrap();
    assert_eq!(moves.len(), 20); // 16 pawn + 4 knight moves
}

#[test]
fn get_legal_moves_invalid_fen() {
    assert!(api::get_legal_moves("garbage").is_err());
}

#[test]
fn evaluate_starting_position_near_zero() {
    let score = api::evaluate_position(START_FEN).unwrap();
    assert!(score.abs() < 1.0, "Starting position should be roughly equal, got {}", score);
}

#[test]
fn evaluate_white_up_queen() {
    // White has an extra queen
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let normal = api::evaluate_position(fen).unwrap();
    let fen_no_bq = "rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let without_bq = api::evaluate_position(fen_no_bq).unwrap();
    assert!(without_bq > normal, "Removing black queen should raise white eval");
}

#[test]
fn apply_moves_e2e4() {
    let result = api::apply_moves(START_FEN, &["e2e4"]).unwrap();
    assert!(result.contains("e3"), "FEN should show en passant square e3: {}", result);
    assert!(result.starts_with("rnbqkbnr/pppppppp/8/8/4P3"));
}

#[test]
fn apply_moves_sequence() {
    let result = api::apply_moves(START_FEN, &["e2e4", "e7e5", "g1f3"]).unwrap();
    assert!(result.contains("b "), "Should be black's turn after 3 half-moves: {}", result);
}

#[test]
fn apply_moves_illegal() {
    assert!(api::apply_moves(START_FEN, &["e2e5"]).is_err());
}

#[test]
fn is_square_attacked_opening() {
    // In the starting position, e2 is attacked by white (king, queen, bishop defend it)
    assert!(api::is_square_attacked(START_FEN, "e2", "white").unwrap());
    // e4 is not attacked by anyone in the starting position
    assert!(!api::is_square_attacked(START_FEN, "e4", "white").unwrap());
    assert!(!api::is_square_attacked(START_FEN, "e4", "black").unwrap());
}

#[test]
fn is_square_attacked_invalid_inputs() {
    assert!(api::is_square_attacked(START_FEN, "z9", "white").is_err());
    assert!(api::is_square_attacked(START_FEN, "e4", "green").is_err());
}

#[test]
fn get_top_moves_returns_requested_count() {
    let moves = api::get_top_moves(START_FEN, 3).unwrap();
    assert_eq!(moves.len(), 3);
}

#[test]
fn get_top_moves_scores_are_sorted() {
    let moves = api::get_top_moves(START_FEN, 5).unwrap();
    for w in moves.windows(2) {
        assert!(w[0].score >= w[1].score, "Moves should be sorted best-first");
    }
}
