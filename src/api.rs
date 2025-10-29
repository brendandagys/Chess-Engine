#[cfg(feature = "api")]
use serde::{Deserialize, Serialize};

use crate::engine::{Engine, SearchSettings};
use crate::position::Position;
use crate::types::Square;

#[cfg_attr(feature = "api", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct AnalyzeRequest {
    pub fen: String,
    pub wtime_ms: Option<u64>,
    pub btime_ms: Option<u64>,
    pub winc_ms: Option<u64>,
    pub binc_ms: Option<u64>,
    pub movetime_ms: Option<u64>,
    pub depth: Option<u16>,
}

#[cfg_attr(feature = "api", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct AnalyzeResponse {
    pub best_move: String,           // The best move in UCI format (e.g., e2e4)
    pub ponder_move: Option<String>, // Suggested move to ponder (think ahead)
    pub evaluation: i32,             // Position evaluation in centipawns
    pub depth: u16,                  // Search depth reached
    pub nodes: u64,                  // Total nodes searched
    pub pv: Vec<String>,             // Principal variation (best line of play)
    pub time_ms: u64,                // Time spent searching in milliseconds
    pub fen_after_move: String,      // FEN string after applying best move
}

/// Main entry point for API consumers
/// Analyzes a chess position and returns the best move
pub fn analyze_position(request: AnalyzeRequest) -> Result<AnalyzeResponse, String> {
    let mut engine = Engine::new(
        request.wtime_ms,
        request.btime_ms,
        request.winc_ms,
        request.binc_ms,
        request.movetime_ms,
        request.depth,
    );

    engine.position =
        Position::from_fen(&request.fen).map_err(|e| format!("Invalid FEN: {}", e))?;

    // Generate moves to validate position is legal
    engine
        .position
        .generate_moves_and_captures(engine.position.side);

    if engine.position.first_move[1] == 0 {
        return Err("No legal moves in position (checkmate or stalemate)".to_string());
    }

    // Perform the search
    let result = engine.think_uci(false);

    if result.best_move.is_empty() {
        return Err("No best move found".to_string());
    }

    // Apply the best move to get the resulting FEN
    let (from, to, promote) = Engine::move_from_uci_string(&result.best_move)
        .map_err(|e| format!("Failed to parse best move: {}", e))?;

    // Make the move on a copy of the position to get the resulting FEN
    if !engine.position.make_move(from, to, promote) {
        return Err("Failed to make best move".to_string());
    }

    let fen_after_move = engine.position.to_fen();

    // Take back the move to restore original position
    engine.position.take_back_move();

    Ok(AnalyzeResponse {
        best_move: result.best_move,
        ponder_move: result.ponder_move,
        evaluation: result.evaluation,
        depth: result.depth,
        nodes: result.nodes,
        pv: result.pv,
        time_ms: result.time_ms,
        fen_after_move,
    })
}

/// Simpler interface with just FEN and depth
pub fn get_best_move(fen: &str, depth: u16) -> Result<AnalyzeResponse, String> {
    analyze_position(AnalyzeRequest {
        fen: fen.to_string(),
        depth: Some(depth),
        movetime_ms: None,
        wtime_ms: None,
        btime_ms: None,
        winc_ms: None,
        binc_ms: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_starting_position() {
        let request = AnalyzeRequest {
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            depth: Some(3),
            movetime_ms: None,
            wtime_ms: None,
            btime_ms: None,
            winc_ms: None,
            binc_ms: None,
        };

        let response = analyze_position(request).unwrap();
        assert!(!response.best_move.is_empty());
        assert_eq!(response.depth, 3);
        assert!(response.nodes > 0);
    }

    #[test]
    fn test_get_best_move_simple() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let response = get_best_move(fen, 3).unwrap();
        assert!(!response.best_move.is_empty());
        assert!(!response.fen_after_move.is_empty());
    }

    #[test]
    fn test_invalid_fen() {
        let request = AnalyzeRequest {
            fen: "invalid fen string".to_string(),
            depth: Some(3),
            movetime_ms: None,
            wtime_ms: None,
            btime_ms: None,
            winc_ms: None,
            binc_ms: None,
        };

        let result = analyze_position(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_tactical_position() {
        // Position with a forced checkmate in 1
        let fen = "r1bqkb1r/pppp1Qpp/2n2n2/4p3/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4";
        let response = get_best_move(fen, 4).unwrap();
        // Engine should find a move (though behavior depends on evaluation)
        assert!(!response.best_move.is_empty());
    }

    #[test]
    fn test_fen_after_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let response = get_best_move(fen, 3).unwrap();

        // The resulting FEN should be different from the starting position
        assert_ne!(response.fen_after_move, fen);

        // And should be a valid FEN string (basic check)
        assert!(response.fen_after_move.contains('/'));
    }
}
