use chess_engine::{position::Position, zobrist_hash::initialize_zobrist_hash_tables};
use std::sync::Once;

fn ensure_zobrist_initialized() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        initialize_zobrist_hash_tables();
    });
}

fn reset_move_state(position: &mut Position) {
    position.ply = 0;
    position.first_move.iter_mut().for_each(|entry| *entry = -1);
    position.first_move[0] = 0;
    position.move_list.iter_mut().for_each(|slot| *slot = None);
}

/// Perform a perft (performance test) from a FEN position to a given depth.
/// Returns the number of leaf nodes (positions) at the target depth.
fn perft(position: &mut Position, depth: usize) -> u64 {
    if depth == 0 {
        return 1;
    }

    let current_ply = position.ply;
    let current_side = position.side;

    // Generate moves for the current side
    position.generate_moves(current_side);

    // Get the move range for current ply
    let start = position.first_move[current_ply].max(0) as usize;
    let end = if current_ply + 1 < position.first_move.len()
        && position.first_move[current_ply + 1] >= 0
    {
        position.first_move[current_ply + 1] as usize
    } else {
        // Find the end by looking for None
        position.move_list[start..]
            .iter()
            .position(|m| m.is_none())
            .map(|pos| start + pos)
            .unwrap_or(position.move_list.len())
    };

    let mut nodes = 0u64;

    // Iterate through each legal move
    for i in start..end {
        if let Some(move_) = position.move_list[i] {
            let from = move_.from;
            let to = move_.to;
            let promote = move_.promote;

            // Make the move
            if position.make_move_with_promotion(from, to, promote) {
                // Recursively count nodes at depth - 1
                nodes += perft(position, depth - 1);

                // Unmake the move
                position.take_back();
            }
        } else {
            break;
        }
    }

    nodes
}

/// Divide perft: Shows the node count for each root move
/// This is useful for debugging - you can compare move-by-move with a reference engine
fn perft_divide(position: &mut Position, depth: usize) -> u64 {
    let current_ply = position.ply;
    let current_side = position.side;

    position.generate_moves(current_side);

    let start = position.first_move[current_ply].max(0) as usize;
    let end = if current_ply + 1 < position.first_move.len()
        && position.first_move[current_ply + 1] >= 0
    {
        position.first_move[current_ply + 1] as usize
    } else {
        position.move_list[start..]
            .iter()
            .position(|m| m.is_none())
            .map(|pos| start + pos)
            .unwrap_or(position.move_list.len())
    };

    let mut total_nodes = 0u64;

    for i in start..end {
        if let Some(move_) = position.move_list[i] {
            let from = move_.from;
            let to = move_.to;
            let promote = move_.promote;

            if position.make_move_with_promotion(from, to, promote) {
                let nodes = if depth > 1 {
                    perft(position, depth - 1)
                } else {
                    1
                };

                // Print move and its node count
                let promote_str = match promote {
                    Some(piece) => format!("{:?}", piece)
                        .chars()
                        .next()
                        .unwrap()
                        .to_lowercase()
                        .to_string(),
                    None => String::new(),
                };
                println!(
                    "{}{}{}: {}",
                    square_to_algebraic(from),
                    square_to_algebraic(to),
                    promote_str,
                    nodes
                );

                total_nodes += nodes;
                position.take_back();
            }
        } else {
            break;
        }
    }

    println!("\nTotal nodes: {}", total_nodes);
    total_nodes
}

/// Convert a square to algebraic notation (e.g., E2, D4)
fn square_to_algebraic(square: chess_engine::types::Square) -> String {
    let file = (square as u8 % 8) as char;
    let rank = (square as u8 / 8) + 1;
    format!("{}{}", (b'a' + file as u8) as char, rank)
}

/// Create a position from a FEN string and prepare it for perft testing
fn position_from_fen(fen: &str) -> Position {
    ensure_zobrist_initialized();
    let mut position = Position::new();

    position
        .load_fen(fen)
        .expect(&format!("Failed to load FEN: {}", fen));

    position.set_material();
    reset_move_state(&mut position);

    position
}

// ============================================================================
// Starting Position Tests
// ============================================================================

#[test]
fn perft_starting_position_depth_1() {
    let mut position =
        position_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let nodes = perft(&mut position, 1);
    assert_eq!(
        nodes, 20,
        "Starting position at depth 1 should have 20 moves"
    );
}

#[test]
fn perft_starting_position_depth_2() {
    let mut position =
        position_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let nodes = perft(&mut position, 2);
    assert_eq!(
        nodes, 400,
        "Starting position at depth 2 should have 400 nodes"
    );
}

#[test]
fn perft_starting_position_depth_3() {
    let mut position =
        position_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let nodes = perft(&mut position, 3);
    assert_eq!(
        nodes, 8_902,
        "Starting position at depth 3 should have 8,902 nodes"
    );
}

#[test]
fn perft_starting_position_depth_4() {
    let mut position =
        position_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let nodes = perft(&mut position, 4);
    assert_eq!(
        nodes, 197_281,
        "Starting position at depth 4 should have 197,281 nodes"
    );
}

#[test]
fn perft_starting_position_depth_5() {
    let mut position =
        position_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let nodes = perft(&mut position, 5);
    assert_eq!(
        nodes, 4_865_609,
        "Starting position at depth 5 should have 4,865,609 nodes"
    );
}

#[test]
#[ignore] // Takes ~35 seconds to run
fn perft_starting_position_depth_6() {
    let mut position =
        position_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let nodes = perft(&mut position, 6);
    assert_eq!(
        nodes, 119_060_324,
        "Starting position at depth 6 should have 119,060,324 nodes"
    );
}

#[test]
#[ignore] // Takes ~70 seconds in release mode
fn perft_starting_position_depth_7() {
    let mut position =
        position_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let nodes = perft(&mut position, 7);
    assert_eq!(
        nodes, 3_195_901_860,
        "Starting position at depth 7 should have 3,195,901,860 nodes"
    );
}

// ============================================================================
// Kiwipete Position Tests
// ============================================================================
// Famous perft testing position with many piece types and possibilities.

#[test]
fn perft_kiwipete_depth_1() {
    let mut position =
        position_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let nodes = perft(&mut position, 1);
    assert_eq!(
        nodes, 48,
        "Kiwipete position at depth 1 should have 48 moves"
    );
}

#[test]
fn perft_kiwipete_depth_2() {
    let mut position =
        position_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let nodes = perft(&mut position, 2);
    assert_eq!(
        nodes, 2_039,
        "Kiwipete position at depth 2 should have 2,039 nodes"
    );
}

#[test]
fn perft_kiwipete_depth_3() {
    let mut position =
        position_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let nodes = perft(&mut position, 3);
    assert_eq!(
        nodes, 97_862,
        "Kiwipete position at depth 3 should have 97,862 nodes"
    );
}

#[test]
fn perft_kiwipete_depth_4() {
    let mut position =
        position_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let nodes = perft(&mut position, 4);
    assert_eq!(
        nodes, 4_085_603,
        "Kiwipete position at depth 4 should have 4,085,603 nodes"
    );
}

#[test]
#[ignore] // Takes ~1 minute to run
fn perft_kiwipete_depth_5() {
    let mut position =
        position_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let nodes = perft(&mut position, 5);
    assert_eq!(
        nodes, 193_690_690,
        "Kiwipete position at depth 5 should have 193,690,690 nodes"
    );
}

// ============================================================================
// Position 3 - Tests castling
// ============================================================================

#[test]
fn perft_position3_depth_1() {
    let mut position = position_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let nodes = perft(&mut position, 1);
    assert_eq!(nodes, 14, "Position 3 at depth 1 should have 14 moves");
}

#[test]
fn perft_position3_depth_2() {
    let mut position = position_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let nodes = perft(&mut position, 2);
    assert_eq!(nodes, 191, "Position 3 at depth 2 should have 191 nodes");
}

#[test]
fn perft_position3_depth_3() {
    let mut position = position_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let nodes = perft(&mut position, 3);
    assert_eq!(
        nodes, 2_812,
        "Position 3 at depth 3 should have 2,812 nodes"
    );
}

#[test]
fn perft_position3_depth_4() {
    let mut position = position_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let nodes = perft(&mut position, 4);
    assert_eq!(
        nodes, 43_238,
        "Position 3 at depth 4 should have 43,238 nodes"
    );
}

#[test]
#[ignore] // Takes ~60 seconds to run
fn perft_position3_depth_5() {
    let mut position = position_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let nodes = perft(&mut position, 5);
    assert_eq!(
        nodes, 674_624,
        "Position 3 at depth 5 should have 674,624 nodes"
    );
}

// ============================================================================
// Position 4 - Tests en passant and promotion
// ============================================================================

#[test]
fn perft_position4_depth_1() {
    let mut position =
        position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let nodes = perft(&mut position, 1);
    assert_eq!(nodes, 6, "Position 4 at depth 1 should have 6 moves");
}

#[test]
fn perft_position4_depth_2() {
    let mut position =
        position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let nodes = perft(&mut position, 2);
    assert_eq!(nodes, 264, "Position 4 at depth 2 should have 264 nodes");
}

#[test]
fn perft_position4_depth_3() {
    let mut position =
        position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let nodes = perft(&mut position, 3);
    assert_eq!(
        nodes, 9_467,
        "Position 4 at depth 3 should have 9,467 nodes"
    );
}

#[test]
fn perft_position4_depth_4() {
    let mut position =
        position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let nodes = perft(&mut position, 4);
    assert_eq!(
        nodes, 422_333,
        "Position 4 at depth 4 should have 422,333 nodes"
    );
}

#[test]
#[ignore] // Takes ~45 seconds to run
fn perft_position4_depth_5() {
    let mut position =
        position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let nodes = perft(&mut position, 5);
    assert_eq!(
        nodes, 15_833_292,
        "Position 4 at depth 5 should have 15,833,292 nodes"
    );
}

#[test]
#[ignore] // Takes ~3 minutes to run
fn perft_position4_depth_6() {
    let mut position =
        position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let nodes = perft(&mut position, 6);
    assert_eq!(
        nodes, 706_045_033,
        "Position 4 at depth 6 should have 706,045,033 nodes"
    );
}

// ============================================================================
// Position 5 - Another complex position
// ============================================================================

#[test]
fn perft_position5_depth_1() {
    let mut position =
        position_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let nodes = perft(&mut position, 1);
    assert_eq!(nodes, 44, "Position 5 at depth 1 should have 44 moves");
}

#[test]
fn perft_position5_depth_2() {
    let mut position =
        position_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let nodes = perft(&mut position, 2);
    assert_eq!(
        nodes, 1_486,
        "Position 5 at depth 2 should have 1,486 nodes"
    );
}

#[test]
fn perft_position5_depth_3() {
    let mut position =
        position_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let nodes = perft(&mut position, 3);
    assert_eq!(
        nodes, 62_379,
        "Position 5 at depth 3 should have 62,379 nodes"
    );
}

#[test]
fn perft_position5_depth_4() {
    let mut position =
        position_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let nodes = perft(&mut position, 4);
    assert_eq!(
        nodes, 2_103_487,
        "Position 5 at depth 4 should have 2,103,487 nodes"
    );
}

#[test]
#[ignore] // Takes ~22 seconds to run
fn perft_position5_depth_5() {
    let mut position =
        position_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let nodes = perft(&mut position, 5);
    assert_eq!(
        nodes, 89_941_194,
        "Position 5 at depth 5 should have 89,941,194 nodes"
    );
}

// ============================================================================
// Position 6 - Tests discovered check and pins
// ============================================================================

#[test]
fn perft_position6_depth_1() {
    let mut position = position_from_fen(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    );
    let nodes = perft(&mut position, 1);
    assert_eq!(nodes, 46, "Position 6 at depth 1 should have 46 moves");
}

#[test]
fn perft_position6_depth_2() {
    let mut position = position_from_fen(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    );
    let nodes = perft(&mut position, 2);
    assert_eq!(
        nodes, 2_079,
        "Position 6 at depth 2 should have 2,079 nodes"
    );
}

#[test]
fn perft_position6_depth_3() {
    let mut position = position_from_fen(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    );
    let nodes = perft(&mut position, 3);
    assert_eq!(
        nodes, 89_890,
        "Position 6 at depth 3 should have 89,890 nodes"
    );
}

#[test]
fn perft_position6_depth_4() {
    let mut position = position_from_fen(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    );
    let nodes = perft(&mut position, 4);
    assert_eq!(
        nodes, 3_894_594,
        "Position 6 at depth 4 should have 3,894,594 nodes"
    );
}

#[test]
#[ignore] // Takes ~45 seconds to run
fn perft_position6_depth_5() {
    let mut position = position_from_fen(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    );
    let nodes = perft(&mut position, 5);
    assert_eq!(
        nodes, 164_075_551,
        "Position 6 at depth 5 should have 164,075,551 nodes"
    );
}

// ============================================================================
// Divide Tests - For debugging move generation
// ============================================================================
// These tests use perft_divide to show the node count for each legal move,
// which is useful for comparing against reference engines to identify bugs.

#[test]
fn divide_starting_position_depth_3() {
    // Quick sanity check showing divide output at low depth
    let mut position =
        position_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let nodes = perft_divide(&mut position, 3);
    assert_eq!(
        nodes, 8_902,
        "Starting position at depth 3 should have 8,902 nodes"
    );
}

#[test]
#[ignore] // Takes ~30 seconds - useful for debugging depth 6 issues
fn divide_starting_position_depth_6() {
    // Shows the node count for each of the 20 possible opening moves
    // Compare this output with a reference engine to find discrepancies
    let mut position =
        position_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let nodes = perft_divide(&mut position, 6);
    assert_eq!(
        nodes, 119_060_324,
        "Starting position at depth 6 should have 119,060,324 nodes"
    );
}

#[test]
#[ignore] // Takes ~1 minute - useful for debugging position 4 issues
fn divide_position4_depth_4() {
    // Position 4 tests promotions and en passant - divide helps debug these
    let mut position =
        position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let nodes = perft_divide(&mut position, 4);
    assert_eq!(
        nodes, 422_333,
        "Position 4 at depth 4 should have 422,333 nodes"
    );
}
