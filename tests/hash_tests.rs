/// Unit tests for hash module
mod hash_test_utils;

use chess_engine::{
    hash::Hash,
    types::{Piece, Side, Square},
    zobrist_hash::initialize_zobrist_hash_tables,
};
use hash_test_utils::*;
use std::sync::Once;

static INIT: Once = Once::new();

/// Ensure Zobrist tables are initialized exactly once for all tests
fn ensure_initialized() {
    INIT.call_once(|| {
        initialize_zobrist_hash_tables();
    });
}

#[test]
fn hash_new_initializes_to_zero() {
    ensure_initialized();
    let hash = Hash::new();
    assert_eq!(hash.current_key, 0, "New hash should start at 0");
}

#[test]
fn hash_store_and_probe_move() {
    ensure_initialized();
    let mut hash = Hash::new();
    let test_move = create_test_move(Square::E2, Square::E4);

    // Initially no move stored
    assert!(hash.probe().is_none(), "Should have no move initially");

    // Set a hash key and store a move
    hash.current_key = 12345;
    hash.store_move(test_move, 5, 100);

    // Should retrieve the same move
    let entry = hash.probe();
    assert!(entry.is_some());
    let entry = entry.unwrap();
    assert!(entry.best_move.is_some());
    let retrieved_move = entry.best_move.unwrap();
    assert_eq!(retrieved_move.from, test_move.from);
    assert_eq!(retrieved_move.to, test_move.to);
    assert_eq!(entry.depth, 5);
    assert_eq!(entry.score, 100);
}

#[test]
fn hash_probe_returns_none_for_different_key() {
    ensure_initialized();
    let mut hash = Hash::new();
    let test_move = create_test_move(Square::E2, Square::E4);

    // Store move at one hash key
    hash.current_key = 12345;
    hash.store_move(test_move, 5, 100);

    // Change hash key
    hash.current_key = 54321;

    // Should not retrieve the move (collision detection)
    assert!(
        hash.probe().is_none(),
        "Should not find move with different hash key"
    );
}

#[test]
fn hash_store_overwrites_previous_entry() {
    ensure_initialized();
    let mut hash = Hash::new();
    let move1 = create_test_move(Square::E2, Square::E4);
    let move2 = create_test_move(Square::D2, Square::D4);

    hash.current_key = 12345;

    // Store first move
    hash.store_move(move1, 5, 100);
    let entry = hash.probe().unwrap();
    assert_eq!(entry.best_move.unwrap().from, Square::E2);

    // Store second move with same hash and greater depth (overwrites)
    hash.store_move(move2, 6, 200);
    let entry = hash.probe().unwrap();
    assert_eq!(entry.best_move.unwrap().from, Square::D2);
    assert_eq!(entry.best_move.unwrap().to, Square::D4);
}

#[test]
fn hash_toggle_piece_changes_key() {
    ensure_initialized();
    let mut hash = Hash::new();
    let initial_key = hash.current_key;

    // Toggle a piece
    hash.toggle_piece(Side::White, Piece::Pawn, Square::E2);

    // Key should change
    assert_ne!(
        hash.current_key, initial_key,
        "Toggling piece should change hash"
    );
}

#[test]
fn hash_toggle_piece_twice_cancels() {
    ensure_initialized();
    let mut hash = Hash::new();
    let initial_key = hash.current_key;

    // Toggle piece on and off
    hash.toggle_piece(Side::White, Piece::Pawn, Square::E2);
    hash.toggle_piece(Side::White, Piece::Pawn, Square::E2);

    // Should return to original
    assert_eq!(
        hash.current_key, initial_key,
        "Double toggle should cancel out"
    );
}

#[test]
fn hash_toggle_empty_piece_no_effect() {
    ensure_initialized();
    let mut hash = Hash::new();
    hash.current_key = 12345;
    let initial_key = hash.current_key;

    // Toggling Empty piece should have no effect
    hash.toggle_piece(Side::White, Piece::Empty, Square::E2);

    assert_eq!(
        hash.current_key, initial_key,
        "Empty piece should not affect hash"
    );
}

#[test]
fn hash_different_pieces_produce_different_hashes() {
    let mut hash1 = Hash::new();
    let mut hash2 = Hash::new();

    // Add different pieces at the same square
    hash1.toggle_piece(Side::White, Piece::Pawn, Square::E4);
    hash2.toggle_piece(Side::White, Piece::Knight, Square::E4);

    assert_ne!(
        hash1.current_key, hash2.current_key,
        "Different pieces should give different hashes"
    );
}

#[test]
fn hash_different_squares_produce_different_hashes() {
    let mut hash1 = Hash::new();
    let mut hash2 = Hash::new();

    // Add same piece at different squares
    hash1.toggle_piece(Side::White, Piece::Pawn, Square::E2);
    hash2.toggle_piece(Side::White, Piece::Pawn, Square::E4);

    assert_ne!(
        hash1.current_key, hash2.current_key,
        "Different squares should give different hashes"
    );
}

#[test]
fn hash_different_sides_produce_different_hashes() {
    let mut hash1 = Hash::new();
    let mut hash2 = Hash::new();

    // Add same piece for different sides
    hash1.toggle_piece(Side::White, Piece::Pawn, Square::E4);
    hash2.toggle_piece(Side::Black, Piece::Pawn, Square::E4);

    assert_ne!(
        hash1.current_key, hash2.current_key,
        "Different sides should give different hashes"
    );
}

#[test]
fn hash_toggle_side_to_move_changes_key() {
    ensure_initialized();
    let mut hash = Hash::new();
    let initial_key = hash.current_key;

    // Toggle side to move
    hash.toggle_side_to_move();

    assert_ne!(
        hash.current_key, initial_key,
        "Toggling side should change hash"
    );
}

#[test]
fn hash_toggle_side_to_move_twice_cancels() {
    ensure_initialized();
    let mut hash = Hash::new();
    let initial_key = hash.current_key;

    // Toggle side twice
    hash.toggle_side_to_move();
    hash.toggle_side_to_move();

    assert_eq!(
        hash.current_key, initial_key,
        "Double toggle side should cancel out"
    );
}

#[test]
fn hash_update_castle_rights_same_state_no_change() {
    ensure_initialized();
    let mut hash = Hash::new();
    hash.current_key = 12345;
    let initial_key = hash.current_key;

    // Update with same state
    hash.update_castle_rights(5, 5);

    assert_eq!(
        hash.current_key, initial_key,
        "Same castle state should not change hash"
    );
}

#[test]
fn hash_update_castle_rights_different_states() {
    ensure_initialized();
    let mut hash = Hash::new();
    let initial_key = hash.current_key;

    // Change castle rights
    hash.update_castle_rights(0, 15);

    assert_ne!(
        hash.current_key, initial_key,
        "Different castle rights should change hash"
    );
}

#[test]
fn hash_update_castle_rights_xor_property() {
    ensure_initialized();
    let mut hash = Hash::new();

    // Go from state 0 to 5
    hash.update_castle_rights(0, 5);
    let key_at_5 = hash.current_key;

    // Go from state 5 to 10
    hash.update_castle_rights(5, 10);

    // Go back from 10 to 5
    hash.update_castle_rights(10, 5);

    assert_eq!(
        hash.current_key, key_at_5,
        "Reverting castle rights should return to previous hash"
    );
}

#[test]
fn hash_update_en_passant_same_file_no_change() {
    ensure_initialized();
    let mut hash = Hash::new();
    hash.current_key = 12345;
    let initial_key = hash.current_key;

    // Update with same file
    hash.update_en_passant(Some(3), Some(3));

    assert_eq!(
        hash.current_key, initial_key,
        "Same en passant file should not change hash"
    );
}

#[test]
fn hash_update_en_passant_both_none_no_change() {
    ensure_initialized();
    let mut hash = Hash::new();
    hash.current_key = 12345;
    let initial_key = hash.current_key;

    // Update with both None
    hash.update_en_passant(None, None);

    assert_eq!(
        hash.current_key, initial_key,
        "Both None should not change hash"
    );
}

#[test]
fn hash_update_en_passant_add_file() {
    ensure_initialized();
    let mut hash = Hash::new();
    let initial_key = hash.current_key;

    // Add en passant file
    hash.update_en_passant(None, Some(4));

    assert_ne!(
        hash.current_key, initial_key,
        "Adding en passant file should change hash"
    );
}

#[test]
fn hash_update_en_passant_remove_file() {
    ensure_initialized();
    let mut hash = Hash::new();

    // Add then remove en passant file
    hash.update_en_passant(None, Some(4));
    let with_ep = hash.current_key;

    hash.update_en_passant(Some(4), None);

    assert_eq!(
        hash.current_key, 0,
        "Removing en passant should return to zero"
    );
    assert_ne!(with_ep, 0, "Hash with EP should be non-zero");
}

#[test]
fn hash_update_en_passant_change_file() {
    ensure_initialized();
    let mut hash = Hash::new();

    // Set to file 3
    hash.update_en_passant(None, Some(3));
    let key_at_3 = hash.current_key;

    // Change to file 5
    hash.update_en_passant(Some(3), Some(5));
    let key_at_5 = hash.current_key;

    // All three should be different
    assert_ne!(key_at_3, 0);
    assert_ne!(key_at_5, 0);
    assert_ne!(key_at_3, key_at_5);
}

#[test]
fn hash_update_en_passant_xor_property() {
    ensure_initialized();
    let mut hash = Hash::new();

    // Add file 4
    hash.update_en_passant(None, Some(4));
    let key_with_4 = hash.current_key;

    // Remove file 4
    hash.update_en_passant(Some(4), None);

    assert_eq!(hash.current_key, 0, "Adding and removing should cancel");

    // Add file 4 again
    hash.update_en_passant(None, Some(4));

    assert_eq!(
        hash.current_key, key_with_4,
        "Re-adding should give same hash"
    );
}

#[test]
fn hash_complex_position_builds_incrementally() {
    ensure_initialized();
    let mut hash = Hash::new();

    // Build a position incrementally
    hash.toggle_piece(Side::White, Piece::Pawn, Square::E2);
    hash.toggle_piece(Side::White, Piece::Knight, Square::B1);
    hash.toggle_piece(Side::Black, Piece::Pawn, Square::E7);
    hash.update_castle_rights(0, 15); // All castle rights
    hash.toggle_side_to_move(); // Black to move

    let full_hash = hash.current_key;
    assert_ne!(full_hash, 0);

    // Create same position in different order
    let mut hash2 = Hash::new();
    hash2.toggle_piece(Side::Black, Piece::Pawn, Square::E7);
    hash2.toggle_piece(Side::White, Piece::Knight, Square::B1);
    hash2.toggle_piece(Side::White, Piece::Pawn, Square::E2);
    hash2.toggle_side_to_move();
    hash2.update_castle_rights(0, 15);

    // Should be same hash (XOR is commutative)
    assert_eq!(
        hash.current_key, hash2.current_key,
        "Order shouldn't matter for same position"
    );
}

#[test]
fn hash_move_simulation() {
    ensure_initialized();
    let mut hash = Hash::new();

    // Starting position with pawn on e2
    hash.toggle_piece(Side::White, Piece::Pawn, Square::E2);
    let after_setup = hash.current_key;

    // Simulate move e2-e4
    hash.toggle_piece(Side::White, Piece::Pawn, Square::E2); // Remove from e2
    hash.toggle_piece(Side::White, Piece::Pawn, Square::E4); // Add to e4
    hash.toggle_side_to_move(); // Change turn

    let after_move = hash.current_key;
    assert_ne!(after_move, after_setup);

    // Undo the move
    hash.toggle_piece(Side::White, Piece::Pawn, Square::E4); // Remove from e4
    hash.toggle_piece(Side::White, Piece::Pawn, Square::E2); // Add back to e2
    hash.toggle_side_to_move(); // Change turn back

    assert_eq!(
        hash.current_key, after_setup,
        "Undoing move should restore hash"
    );
}

#[test]
fn hash_table_collision_handling() {
    ensure_initialized();
    let mut hash = Hash::new();
    let move1 = create_test_move(Square::E2, Square::E4);
    let move2 = create_test_move(Square::D2, Square::D4);

    // Store move at key1
    hash.current_key = 1000;
    hash.store_move(move1, 5, 100);

    // Change to different key that might collide in table
    // (Same index after modulo NUM_HASH_SLOTS but different key)
    hash.current_key = 1001;
    hash.store_move(move2, 5, 200);

    // Should retrieve move2, not move1 (collision detected by hash_key check)
    let entry = hash.probe().unwrap();
    assert_eq!(entry.best_move.unwrap().from, Square::D2);
    assert_eq!(entry.best_move.unwrap().to, Square::D4);
}

#[test]
fn hash_all_pieces_affect_hash_uniquely() {
    let pieces = valid_pieces();
    let mut hashes = Vec::new();

    // Add each piece type and record hash
    for piece in &pieces {
        let mut hash = Hash::new();
        hash.toggle_piece(Side::White, *piece, Square::E4);
        hashes.push(hash.current_key);
    }

    // All should be different
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            assert_ne!(
                hashes[i], hashes[j],
                "{:?} and {:?} produced same hash",
                pieces[i], pieces[j]
            );
        }
    }
}

#[test]
fn hash_all_castle_states_produce_different_hashes() {
    let mut hashes = Vec::new();

    // Test all 16 castle states
    for state in castle_states() {
        let mut hash = Hash::new();
        hash.update_castle_rights(0, state);
        hashes.push((state, hash.current_key));
    }

    // Each state should produce unique hash
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            assert_ne!(
                hashes[i].1, hashes[j].1,
                "Castle states {} and {} produced same hash",
                hashes[i].0, hashes[j].0
            );
        }
    }
}

#[test]
fn hash_all_en_passant_files_produce_different_hashes() {
    let mut hashes = Vec::new();

    // Test all 8 en passant files
    for file in en_passant_files() {
        let mut hash = Hash::new();
        hash.update_en_passant(None, Some(file));
        hashes.push((file, hash.current_key));
    }

    // Each file should produce unique hash
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            assert_ne!(
                hashes[i].1, hashes[j].1,
                "En passant files {} and {} produced same hash",
                hashes[i].0, hashes[j].0
            );
        }
    }
}

#[test]
fn hash_position_uniqueness() {
    // Create two similar but different positions
    let mut hash1 = Hash::new();
    hash1.toggle_piece(Side::White, Piece::Pawn, Square::E4);
    hash1.toggle_piece(Side::Black, Piece::Pawn, Square::E5);

    let mut hash2 = Hash::new();
    hash2.toggle_piece(Side::White, Piece::Pawn, Square::E4);
    hash2.toggle_piece(Side::Black, Piece::Pawn, Square::D5); // Different square

    assert_ne!(
        hash1.current_key, hash2.current_key,
        "Different positions should have different hashes"
    );
}

#[test]
fn hash_side_to_move_affects_uniqueness() {
    // Same pieces, white to move
    let mut hash1 = Hash::new();
    hash1.toggle_piece(Side::White, Piece::King, Square::E1);
    hash1.toggle_piece(Side::Black, Piece::King, Square::E8);

    // Same pieces, black to move
    let mut hash2 = Hash::new();
    hash2.toggle_piece(Side::White, Piece::King, Square::E1);
    hash2.toggle_piece(Side::Black, Piece::King, Square::E8);
    hash2.toggle_side_to_move();

    assert_ne!(
        hash1.current_key, hash2.current_key,
        "Side to move should affect hash"
    );
}

#[test]
fn hash_castle_rights_affect_uniqueness() {
    // Position with castling available
    let mut hash1 = Hash::new();
    hash1.toggle_piece(Side::White, Piece::King, Square::E1);
    hash1.update_castle_rights(0, 3); // White can castle both sides

    // Same position, no castling
    let mut hash2 = Hash::new();
    hash2.toggle_piece(Side::White, Piece::King, Square::E1);
    hash2.update_castle_rights(0, 0); // No castling

    assert_ne!(
        hash1.current_key, hash2.current_key,
        "Castle rights should affect hash"
    );
}

#[test]
fn hash_en_passant_affects_uniqueness() {
    // Position with en passant available
    let mut hash1 = Hash::new();
    hash1.toggle_piece(Side::White, Piece::Pawn, Square::E5);
    hash1.update_en_passant(None, Some(4)); // e-file

    // Same position, no en passant
    let mut hash2 = Hash::new();
    hash2.toggle_piece(Side::White, Piece::Pawn, Square::E5);

    assert_ne!(
        hash1.current_key, hash2.current_key,
        "En passant should affect hash"
    );
}
