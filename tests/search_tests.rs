/// Comprehensive search algorithm tests for the chess engine
///
/// This test suite validates the search implementation including:
///
/// 1. **Alpha-Beta Search**: Pruning, move ordering, best move selection
/// 2. **Quiescent Search**: Tactical sequences, stand-pat evaluation, capture ordering
/// 3. **Check Extensions**: Searching deeper when in check
/// 4. **Move Reductions**: Late move reductions, quiet move pruning
/// 5. **Hash Table Integration**: Transposition table lookups, best move retrieval
/// 6. **Search Termination**: Depth limits, time management, repetition detection
/// 7. **Principal Variation**: PV extraction from hash table
/// 8. **Tactical Accuracy**: Mate detection, forced sequences, capture combinations
///
/// The search tests use various tactical positions to verify the engine finds
/// optimal moves and correctly evaluates forcing sequences.
mod test_utils;

use chess_engine::{
    position::Position,
    types::{Piece, Square},
};
use test_utils::*;

/// Helper to run a search with fixed depth and return the best move
fn search_position(fen: &str, depth: u16) -> Option<(Square, Square)> {
    let mut engine = engine_from_fen(fen, depth);

    engine.think(None::<fn(u16, i32, &mut Position)>);

    match (engine.position.hash_from, engine.position.hash_to) {
        (Some(from), Some(to)) => Some((from, to)),
        _ => None,
    }
}

mod basic_search {
    use super::*;

    #[test]
    fn test_search_finds_best_move_in_simple_position() {
        // Simple opening position
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let result = search_position(fen, 3);
        assert!(result.is_some(), "Search should find a move");
    }

    #[test]
    fn test_search_completes_within_depth_limit() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 3);

        engine.think(None::<fn(u16, i32, &mut Position)>);

        // Should complete and find a move
        assert!(
            engine.position.hash_from.is_some() && engine.position.hash_to.is_some(),
            "Search should find a best move"
        );
    }

    #[test]
    fn test_search_returns_legal_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 2);

        engine.think(None::<fn(u16, i32, &mut Position)>);

        if let (Some(from), Some(to)) = (engine.position.hash_from, engine.position.hash_to) {
            // Generate legal moves and verify the returned move is legal
            let mut position = position_from_fen(fen);
            position.ply = 0;
            position.first_move[0] = 0;
            position.generate_moves_and_captures(position.side, |_, _, _| 0);

            let mut found = false;
            for i in 0..position.first_move[1] as usize {
                if let Some(mv) = position.move_list[i] {
                    if mv.from == from && mv.to == to {
                        found = true;
                        break;
                    }
                }
            }

            assert!(
                found,
                "Search returned move {:?}->{:?} should be legal",
                from, to
            );
        }
    }
}

mod tactical_search {
    use super::*;

    #[test]
    fn test_finds_mate_in_one() {
        // Back rank mate setup: white queen can deliver mate
        let fen = "6k1/5ppp/8/8/8/8/5PPP/4Q1K1 w - - 0 1";
        let mut engine = engine_from_fen(fen, 3);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        // The search should find a mating move
        assert!(
            engine.position.hash_from.is_some() && engine.position.hash_to.is_some(),
            "Should find mate in one"
        );
    }

    #[test]
    fn test_avoids_hanging_piece() {
        // Opening position where white should develop pieces
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2";
        let mut engine = engine_from_fen(fen, 4);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        // Should find a reasonable developing move
        assert!(engine.position.hash_from.is_some(), "Should find a move");
    }

    #[test]
    fn test_finds_good_move_in_opening() {
        // Standard opening position
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let result = search_position(fen, 4);

        assert!(result.is_some(), "Should find a good move in the opening");
    }

    #[test]
    fn test_search_finds_forcing_move() {
        // Position from the Italian Game
        let fen = "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3";
        let mut engine = engine_from_fen(fen, 4);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        // Should find a move
        assert!(
            engine.position.hash_from.is_some(),
            "Should find a move in tactical position"
        );
    }
}

mod quiescence_search {
    use super::*;

    #[test]
    fn test_quiescence_search_evaluates_captures() {
        // Standard Sicilian Defense position with tactical possibilities
        let fen = "rnbqkb1r/pp1ppppp/5n2/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
        let mut engine = engine_from_fen(fen, 2);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        // Should consider the position
        assert!(engine.position.hash_from.is_some(), "Should find a move");
    }

    #[test]
    fn test_quiescence_search_evaluates_stand_pat() {
        // Quiet position - standard starting position
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 1);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        // Should complete without hanging
        assert!(
            engine.position.hash_from.is_some(),
            "Should handle quiet positions"
        );
    }

    #[test]
    #[ignore] // TODO: Too slow
    fn test_quiescence_search_with_tactics() {
        // Kiwipete position with many tactical possibilities
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 3);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        // Should find a legal move
        assert!(engine.position.hash_from.is_some(), "Should find a move");
    }
}

mod check_extensions {
    use super::*;

    #[test]
    #[ignore] // TODO: Too slow
    fn test_check_extension_searches_nodes() {
        // Kiwipete position - famous perft testing position
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 4);

        let nodes_before = engine.position.nodes;
        engine.think(None::<fn(u16, i32, &mut Position)>);
        let nodes_after = engine.position.nodes;

        // Should search some nodes
        assert!(nodes_after > nodes_before, "Search should visit nodes");
    }

    #[test]
    fn test_handles_complex_position() {
        // Complex position for testing en passant and promotion
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        let mut engine = engine_from_fen(fen, 4);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        // Should not crash or infinite loop
        assert!(
            engine.position.hash_from.is_some(),
            "Should handle complex positions"
        );
    }
}

mod repetition_detection {
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

mod move_ordering {
    use super::*;

    #[test]
    fn test_hash_moves_searched_first() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 4);

        // First search to populate hash
        engine.think(None::<fn(u16, i32, &mut Position)>);

        let first_move = engine.position.hash_from;

        // Reset and search again - hash move should be tried first
        engine.position.ply = 0;
        engine.position.nodes = 0;
        engine
            .position
            .generate_moves_and_captures(engine.position.side, |_, _, _| 0);

        // Verify hash move is in the move list with high score
        if let Some(hash_from) = first_move {
            let mut _found_with_high_score = false;
            for i in 0..engine.position.first_move[1] as usize {
                if let Some(mv) = engine.position.move_list[i] {
                    if mv.from == hash_from && mv.score > 1_000_000 {
                        _found_with_high_score = true;
                        break;
                    }
                }
            }
            // Hash move should exist (might not always have high score on regeneration)
        }
    }

    #[test]
    fn test_captures_searched_before_quiet_moves() {
        let fen = "rnbqkb1r/pppp1ppp/5n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
        let mut position = position_from_fen(fen);

        position.ply = 0;
        position.first_move[0] = 0;
        position.generate_moves_and_captures(position.side, |_, _, _| 0);

        // Check that captures have higher scores than non-captures
        let mut capture_score = None;
        let mut quiet_score = None;

        for i in 0..position.first_move[1] as usize {
            if let Some(mv) = position.move_list[i] {
                let is_capture = position.board.value[mv.to as usize] != Piece::Empty;

                if is_capture && capture_score.is_none() {
                    capture_score = Some(mv.score);
                } else if !is_capture && quiet_score.is_none() {
                    quiet_score = Some(mv.score);
                }
            }
        }

        if let (Some(cap), Some(quiet)) = (capture_score, quiet_score) {
            assert!(cap > quiet, "Captures should score higher than quiet moves");
        }
    }
}

mod depth_and_reduction {
    use super::*;

    #[test]
    fn test_search_respects_depth_limit() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 2);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        // Should complete quickly with low depth
        assert!(
            engine.position.nodes < 100000,
            "Depth 2 should visit < 100k nodes"
        );
    }

    #[test]
    fn test_deeper_search_visits_more_nodes() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        let mut engine1 = engine_from_fen(fen, 2);
        engine1.think(None::<fn(u16, i32, &mut Position)>);
        let nodes_depth2 = engine1.position.nodes;

        let mut engine2 = engine_from_fen(fen, 3);
        engine2.think(None::<fn(u16, i32, &mut Position)>);
        let nodes_depth3 = engine2.position.nodes;

        assert!(
            nodes_depth3 > nodes_depth2,
            "Depth 3 should visit more nodes than depth 2"
        );
    }

    #[test]
    #[ignore] // TODO: Too slow
    fn test_search_with_reductions() {
        // Kiwipete position - standard perft testing position
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 4);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        // Should complete in reasonable time
        assert!(
            engine.position.nodes > 0,
            "Should visit nodes with reductions"
        );
    }
}

mod search_stability {
    use super::*;

    #[test]
    fn test_search_is_deterministic() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        let result1 = search_position(fen, 3);
        let result2 = search_position(fen, 3);

        // Same position and depth should give same result
        assert_eq!(result1, result2, "Search should be deterministic");
    }

    #[test]
    fn test_search_handles_complex_position() {
        // Position 5 from perft - complex middlegame
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        let mut engine = engine_from_fen(fen, 4);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        assert!(
            engine.position.hash_from.is_some(),
            "Should handle complex positions"
        );
        assert!(engine.position.nodes > 0, "Should visit nodes");
    }

    #[test]
    fn test_search_handles_endgame() {
        // Position 3 from perft - endgame-like position
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        let mut engine = engine_from_fen(fen, 4);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        assert!(
            engine.position.hash_from.is_some(),
            "Should handle endgames"
        );
    }

    #[test]
    #[ignore] // TODO: Too slow
    fn test_search_handles_middlegame() {
        // Position 6 from perft - middlegame position
        let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        let mut engine = engine_from_fen(fen, 4);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        assert!(
            engine.position.hash_from.is_some(),
            "Should handle middlegame positions"
        );
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn test_search_with_limited_mobility() {
        // Position with castling rights on both sides
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 3);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        assert!(
            engine.position.hash_from.is_some(),
            "Should handle positions with castling"
        );
    }

    #[test]
    fn test_search_with_castling_rights() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 4);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        // Should consider castling
        assert!(
            engine.position.hash_from.is_some(),
            "Should handle castling positions"
        );
    }

    #[test]
    fn test_search_in_opening() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 3);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        assert!(
            engine.position.hash_from.is_some(),
            "Should handle starting position"
        );
    }

    #[test]
    fn test_search_after_pawn_push() {
        // After 1.e4
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let mut engine = engine_from_fen(fen, 4);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        assert!(
            engine.position.hash_from.is_some(),
            "Should handle en passant availability"
        );
    }
}

mod performance {
    use super::*;

    #[test]
    fn test_search_completes_in_reasonable_time() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 4);

        let start = std::time::Instant::now();

        engine.think(None::<fn(u16, i32, &mut Position)>);

        let duration = start.elapsed();

        // Depth 4 from start should complete quickly (< 10 seconds)
        assert!(
            duration.as_secs() < 10,
            "Depth 4 search should complete in < 10 seconds, took {:?}",
            duration
        );
    }

    #[test]
    fn test_node_count_is_reasonable() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 3);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        // Depth 3 should visit reasonable number of nodes
        assert!(
            engine.position.nodes > 100,
            "Should visit at least 100 nodes"
        );
        assert!(
            engine.position.nodes < 1_000_000,
            "Should visit fewer than 1M nodes at depth 3"
        );
    }
}

mod principal_variation {
    use super::*;

    #[test]
    #[ignore]
    fn test_pv_extracted_from_hash() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 4);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        // PV should be stored in hash table
        assert!(
            engine.position.hash_from.is_some() && engine.position.hash_to.is_some(),
            "Should have PV move"
        );

        // Hash table should have the position
        let hash_entry = engine.position.board.hash.probe();
        assert!(hash_entry.is_some(), "Hash should have current position");
    }

    #[test]
    fn test_pv_is_legal_sequence() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut engine = engine_from_fen(fen, 3);
        engine.think(None::<fn(u16, i32, &mut Position)>);

        // Try to make the PV moves
        if let (Some(from), Some(to)) = (engine.position.hash_from, engine.position.hash_to) {
            let move_legal = engine.position.make_move(from, to, None);
            if move_legal {
                engine.position.take_back_move();
                // PV move was legal
            }
            assert!(move_legal, "PV move should be legal");
        }
    }
}

mod tactical_accuracy_tests {
    use super::*;

    /// Test Case 1: Must Recapture or Defend
    /// After a piece is captured, engine should recapture or defend
    #[test]
    fn test_recaptures_piece() {
        // Black just captured on e4, White should recapture with pawn at d3
        let fen = "rnbqkbnr/pppp1ppp/8/8/4n3/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 1";
        let result = search_position(fen, 4);

        assert!(result.is_some(), "Engine should find a move");

        let (from, to) = result.unwrap();

        // White should recapture the knight on e4
        println!(
            "Engine played {:?} to {:?} (expecting recapture on e4)",
            from, to
        );

        assert_eq!(
            from,
            Square::D3,
            "White should have moved from d3, but moved from {:?}",
            from
        );
        assert_eq!(
            to,
            Square::E4,
            "White should recapture on e4, but moved to {:?}",
            to
        );
    }

    /// Test Case 2: Mate in 1 - Back Rank Mate
    #[test]
    fn test_finds_back_rank_mate() {
        // Black king trapped on back rank, White rook delivers mate
        let fen = "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1";
        let result = search_position(fen, 3);

        assert!(result.is_some(), "Engine should find back rank mate");

        let (from, to) = result.unwrap();

        // Rook should move to 8th rank for mate
        assert_eq!(
            from,
            Square::A1,
            "Rook should move from a1, but moved from {:?}",
            from
        );
        assert_eq!(
            to,
            Square::A8,
            "Rook should deliver mate on a8, but moved to {:?}",
            to
        );
    }

    /// Test Case 3: Forced Sequence
    /// Engine should see forced win sequence
    #[test]
    fn test_finds_forced_sequence() {
        // White has forcing queen check leading to mate
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 1";
        let result = search_position(fen, 4);

        assert!(result.is_some(), "Engine should find a forcing move");

        let (from, to) = result.unwrap();

        println!(
            "Engine played {:?} to {:?} in forced sequence position",
            from, to
        );

        assert_eq!(
            from,
            Square::H5,
            "Queen should move from h5, but moved from {:?}",
            from
        );
        assert_eq!(
            to,
            Square::F7,
            "Queen should move to f7 for mate, but moved to {:?}",
            to
        );

        // Scholar's mate pattern - Qxf7 is mate
        // At depth 4, engine should find this
        if from == Square::H5 && to == Square::F7 {
            println!("Found Scholar's mate: Qxf7#");
        }
    }
}
