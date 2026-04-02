//! Agent-facing API for using the chess engine as a tool.
//!
//! All functions accept FEN strings and return simple types suitable for
//! serialization / tool-call responses.

use std::panic;

use crate::{
    constants::{INFINITY_SCORE, NUM_SIDES, NUM_SQUARES},
    position::Position,
    types::{Board, Side, Square},
};

/// Fixed search depth used for `get_top_moves`. Shallow enough to be fast
/// across all root moves while still producing tactically meaningful scores.
const TOP_MOVES_DEPTH: u16 = 4;

/// A scored move returned by [`get_top_moves`].
#[derive(Debug, Clone)]
pub struct ScoredMove {
    /// UCI move string (e.g. `"e2e4"`, `"e7e8q"`).
    pub mv: String,
    /// Evaluation from the side-to-move's perspective, in pawns.
    /// Positive = good for side to move.
    pub score: f64,
}

/// Return the top `n` moves for the position given by `fen`, ranked by engine
/// evaluation (best first). Each move is searched to [`TOP_MOVES_DEPTH`] so
/// scores are directly comparable across moves.
pub fn get_top_moves(fen: &str, n: usize) -> Result<Vec<ScoredMove>, String> {
    let mut pos = Position::from_fen(fen).map_err(|e| e.to_string())?;
    pos.set_material_scores();

    // Generate all pseudo-legal moves at the root
    pos.ply = 0;
    pos.first_move[0] = 0;
    pos.generate_moves_and_captures(pos.side, |_, _, _| 0);

    let mut scored: Vec<ScoredMove> = Vec::new();
    let move_end = pos.first_move[1]; // first_move[ply+1] where ply=0

    for i in pos.first_move[0]..move_end {
        let mv = match pos.move_list[i as usize] {
            Some(m) => m,
            None => continue,
        };

        if !pos.make_move(mv.from, mv.to, mv.promote) {
            continue; // Illegal (leaves king in check)
        }

        // Search from the opponent's perspective, then negate.
        // Wrapped in catch_unwind because the engine uses panics as control
        // flow for NodeLimitReached / TimeExhausted.
        let mut history_table = [[[0isize; NUM_SQUARES]; NUM_SQUARES]; NUM_SIDES]; // TODO: Can `.search()` default this?
        let search_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            pos.search(
                -INFINITY_SCORE,
                INFINITY_SCORE,
                TOP_MOVES_DEPTH - 1,
                &mut history_table,
                None,
            )
        }));

        // Restore position to the pre-root-move state regardless of panic
        while pos.ply > 0 {
            pos.take_back_move();
        }

        if let Ok(score) = search_result {
            scored.push(ScoredMove {
                mv: Board::move_to_uci_string(mv.from, mv.to, mv.promote, false),
                score: -score as f64 / 100.0, // Negate: score is from opponent's POV
            });
        }
        // If search panicked (aborted early), skip this move
    }

    // Best first (highest score = best for side-to-move)
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    scored.truncate(n);

    Ok(scored)
}

/// Apply a sequence of moves (in UCI notation, e.g. `["e2e4", "e7e5"]`) to the
/// position given by `fen` and return the resulting FEN string.
pub fn apply_moves(fen: &str, moves: &[&str]) -> Result<String, String> {
    let mut pos = Position::from_fen(fen).map_err(|e| e.to_string())?;

    for move_str in moves {
        if !pos.get_legal_moves().contains(&move_str.to_string()) {
            return Err(format!("Illegal move: {}", move_str));
        }

        let mv = Board::move_from_uci_string(move_str)?;
        pos.make_move(mv.from, mv.to, mv.promote);
    }

    Ok(pos.to_fen())
}

/// Return a static evaluation of the position (no search), in pawns, from
/// **white's perspective**. Positive = white is better.
pub fn evaluate_position(fen: &str) -> Result<f64, String> {
    let mut pos = Position::from_fen(fen).map_err(|e| e.to_string())?;
    pos.set_material_scores();
    // Resolves captures but doesn't do a full tree search
    let raw = pos.quiescence_search(-INFINITY_SCORE, INFINITY_SCORE, 0, Some(100_000));
    let white_score = if pos.side == Side::White { raw } else { -raw };

    Ok(white_score as f64 / 100.0)
}

/// Return all legal moves in the position as UCI strings (e.g. `"e2e4"`).
pub fn get_legal_moves(fen: &str) -> Result<Vec<String>, String> {
    let mut pos = Position::from_fen(fen).map_err(|e| e.to_string())?;
    Ok(pos.get_legal_moves())
}

/// Return whether `square` (e.g. `"e4"`) is attacked by `by_color`
/// (`"white"` or `"black"`).
pub fn is_square_attacked(fen: &str, square: &str, by_color: &str) -> Result<bool, String> {
    let pos = Position::from_fen(fen).map_err(|e| e.to_string())?;

    let side = match by_color.to_lowercase().as_str() {
        "white" | "w" => Side::White,
        "black" | "b" => Side::Black,
        _ => {
            return Err(format!(
                "Invalid color: '{}'. Use 'white' or 'black'.",
                by_color
            ));
        }
    };

    if square.len() != 2 {
        return Err(format!(
            "Invalid square: '{}'. Use algebraic notation like 'e4'.",
            square
        ));
    }

    let chars: Vec<char> = square.chars().collect();
    let file = chars[0] as i32 - 'a' as i32;
    let rank = chars[1] as i32 - '1' as i32;

    if !(0..8).contains(&file) || !(0..8).contains(&rank) {
        return Err(format!(
            "Invalid square: '{}'. Files a-h, ranks 1-8.",
            square
        ));
    }

    let sq_index = (rank * 8 + file) as u8;
    let sq = Square::try_from(sq_index).map_err(|e| e.to_string())?;

    Ok(pos.is_square_attacked_by_side(side, sq))
}
