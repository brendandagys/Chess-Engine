/// Unit tests for zobrist_hash module
mod hash_test_utils;

use chess_engine::{
    types::{Piece, Side, Square},
    zobrist_hash::{
        ZOBRIST_CASTLE_HASH, ZOBRIST_EN_PASSANT_HASH, ZOBRIST_HASH_TABLE,
        ZOBRIST_SIDE_TO_MOVE_HASH, initialize_zobrist_hash_tables,
    },
};
use hash_test_utils::*;
use std::collections::HashSet;
use std::sync::Once;

static INIT: Once = Once::new();

/// Ensure Zobrist tables are initialized exactly once for all tests
fn ensure_initialized() {
    INIT.call_once(|| {
        initialize_zobrist_hash_tables();
    });
}

#[test]
fn zobrist_tables_initialize_once() {
    // Initialize tables
    ensure_initialized();

    // Should succeed and be idempotent
    ensure_initialized();
    ensure_initialized();

    // All tables should be initialized
    assert!(ZOBRIST_HASH_TABLE.get().is_some());
    assert!(ZOBRIST_SIDE_TO_MOVE_HASH.get().is_some());
    assert!(ZOBRIST_CASTLE_HASH.get().is_some());
    assert!(ZOBRIST_EN_PASSANT_HASH.get().is_some());
}

#[test]
fn zobrist_piece_hashes_are_unique() {
    ensure_initialized();

    let hash_table = ZOBRIST_HASH_TABLE.get().unwrap();
    let mut seen = HashSet::new();

    // Check uniqueness across all sides, pieces, and squares
    for side in all_sides() {
        for piece in valid_pieces() {
            for square in Square::iter() {
                let hash = hash_table[side as usize][piece as usize][square as usize];

                // Each hash should be non-zero
                assert_ne!(
                    hash, 0,
                    "Hash for {:?} {:?} on {:?} is zero",
                    side, piece, square
                );

                // Each hash should be unique (collision check)
                assert!(
                    seen.insert(hash),
                    "Duplicate hash {} for {:?} {:?} on {:?}",
                    hash,
                    side,
                    piece,
                    square
                );
            }
        }
    }

    // We should have (2 sides) * (6 pieces) * (64 squares) = 768 unique hashes
    assert_eq!(seen.len(), 2 * 6 * 64);
}

#[test]
fn zobrist_empty_piece_not_hashed() {
    // This test verifies that Empty pieces are correctly skipped in hash initialization
    // We can't directly test the hash_table for Empty since it's not allocated for that index
    // Instead, we verify through the Hash::toggle_piece behavior in hash_tests.rs
    // This test serves as documentation that Empty pieces are intentionally excluded
    ensure_initialized();

    let hash_table = ZOBRIST_HASH_TABLE.get().unwrap();

    // Verify we only have hashes for valid pieces (0-5), not Empty (6)
    // The hash_table has dimensions [2 sides][6 pieces][64 squares]
    // so Piece::Empty (index 6) is out of bounds, which is correct by design
    assert_eq!(
        hash_table[0].len(),
        6,
        "Hash table should only have 6 piece types"
    );
}

#[test]
fn zobrist_side_to_move_hash_is_nonzero() {
    ensure_initialized();

    let side_hash = ZOBRIST_SIDE_TO_MOVE_HASH.get().unwrap();
    assert_ne!(*side_hash, 0, "Side-to-move hash should be non-zero");
}

#[test]
fn zobrist_castle_hashes_are_unique() {
    ensure_initialized();

    let castle_hashes = ZOBRIST_CASTLE_HASH.get().unwrap();
    let mut seen = HashSet::new();

    // All 16 castle states should have unique hashes
    for state in castle_states() {
        let hash = castle_hashes[state as usize];

        assert_ne!(hash, 0, "Castle hash for state {} is zero", state);
        assert!(
            seen.insert(hash),
            "Duplicate castle hash {} for state {}",
            hash,
            state
        );
    }

    assert_eq!(seen.len(), 16);
}

#[test]
fn zobrist_en_passant_hashes_are_unique() {
    ensure_initialized();

    let ep_hashes = ZOBRIST_EN_PASSANT_HASH.get().unwrap();
    let mut seen = HashSet::new();

    // All 8 en passant files should have unique hashes
    for file in en_passant_files() {
        let hash = ep_hashes[file as usize];

        assert_ne!(hash, 0, "En passant hash for file {} is zero", file);
        assert!(
            seen.insert(hash),
            "Duplicate en passant hash {} for file {}",
            hash,
            file
        );
    }

    assert_eq!(seen.len(), 8);
}

#[test]
fn zobrist_all_hashes_globally_unique() {
    ensure_initialized();

    let mut all_hashes = HashSet::new();

    // Collect all piece position hashes
    let hash_table = ZOBRIST_HASH_TABLE.get().unwrap();
    for side in all_sides() {
        for piece in valid_pieces() {
            for square in Square::iter() {
                let hash = hash_table[side as usize][piece as usize][square as usize];
                assert!(all_hashes.insert(hash), "Duplicate hash found: {}", hash);
            }
        }
    }

    // Add side-to-move hash
    let side_hash = *ZOBRIST_SIDE_TO_MOVE_HASH.get().unwrap();
    assert!(
        all_hashes.insert(side_hash),
        "Side-to-move hash collides with piece hash"
    );

    // Add castle hashes
    let castle_hashes = ZOBRIST_CASTLE_HASH.get().unwrap();
    for state in castle_states() {
        let hash = castle_hashes[state as usize];
        assert!(
            all_hashes.insert(hash),
            "Castle hash {} collides with another hash",
            hash
        );
    }

    // Add en passant hashes
    let ep_hashes = ZOBRIST_EN_PASSANT_HASH.get().unwrap();
    for file in en_passant_files() {
        let hash = ep_hashes[file as usize];
        assert!(
            all_hashes.insert(hash),
            "En passant hash {} collides with another hash",
            hash
        );
    }

    // Total: 768 (pieces) + 1 (side) + 16 (castle) + 8 (ep) = 793 unique hashes
    assert_eq!(all_hashes.len(), 768 + 1 + 16 + 8);
}

#[test]
fn zobrist_different_sides_different_hashes() {
    ensure_initialized();

    let hash_table = ZOBRIST_HASH_TABLE.get().unwrap();

    // Same piece on same square but different sides should have different hashes
    for piece in valid_pieces() {
        for square in sample_squares() {
            let white_hash = hash_table[Side::White as usize][piece as usize][square as usize];
            let black_hash = hash_table[Side::Black as usize][piece as usize][square as usize];

            assert_ne!(
                white_hash, black_hash,
                "Same hash for {:?} on {:?} for both sides",
                piece, square
            );
        }
    }
}

#[test]
fn zobrist_different_pieces_different_hashes() {
    ensure_initialized();

    let hash_table = ZOBRIST_HASH_TABLE.get().unwrap();

    // Different pieces on same square should have different hashes
    let test_square = Square::E4;
    let side = Side::White;

    let pieces = valid_pieces();
    for i in 0..pieces.len() {
        for j in (i + 1)..pieces.len() {
            let hash1 = hash_table[side as usize][pieces[i] as usize][test_square as usize];
            let hash2 = hash_table[side as usize][pieces[j] as usize][test_square as usize];

            assert_ne!(
                hash1, hash2,
                "Same hash for {:?} and {:?} on {:?}",
                pieces[i], pieces[j], test_square
            );
        }
    }
}

#[test]
fn zobrist_different_squares_different_hashes() {
    ensure_initialized();

    let hash_table = ZOBRIST_HASH_TABLE.get().unwrap();

    // Same piece on different squares should have different hashes
    let piece = Piece::Knight;
    let side = Side::White;

    let squares = sample_squares();
    for i in 0..squares.len() {
        for j in (i + 1)..squares.len() {
            let hash1 = hash_table[side as usize][piece as usize][squares[i] as usize];
            let hash2 = hash_table[side as usize][piece as usize][squares[j] as usize];

            assert_ne!(
                hash1, hash2,
                "Same hash for {:?} on {:?} and {:?}",
                piece, squares[i], squares[j]
            );
        }
    }
}

#[test]
fn zobrist_xor_property_piece_toggle() {
    ensure_initialized();

    let hash_table = ZOBRIST_HASH_TABLE.get().unwrap();
    let piece_hash = hash_table[Side::White as usize][Piece::Knight as usize][Square::E4 as usize];

    // XORing twice should cancel out
    let mut hash = 0u64;
    hash ^= piece_hash; // Add piece
    assert_eq!(hash, piece_hash);

    hash ^= piece_hash; // Remove piece
    assert_eq!(hash, 0, "XOR twice should cancel out");
}

#[test]
fn zobrist_xor_property_side_toggle() {
    ensure_initialized();

    let side_hash = *ZOBRIST_SIDE_TO_MOVE_HASH.get().unwrap();

    // Toggling side twice should cancel out
    let mut hash = 0u64;
    hash ^= side_hash; // Black to move
    hash ^= side_hash; // White to move
    assert_eq!(hash, 0, "Toggling side twice should return to original");
}

#[test]
fn zobrist_xor_is_commutative() {
    ensure_initialized();

    let hash_table = ZOBRIST_HASH_TABLE.get().unwrap();
    let hash1 = hash_table[Side::White as usize][Piece::Pawn as usize][Square::E2 as usize];
    let hash2 = hash_table[Side::White as usize][Piece::Knight as usize][Square::E4 as usize];
    let hash3 = hash_table[Side::Black as usize][Piece::Pawn as usize][Square::D7 as usize];

    // Order shouldn't matter
    let result1 = hash1 ^ hash2 ^ hash3;
    let result2 = hash3 ^ hash1 ^ hash2;
    let result3 = hash2 ^ hash3 ^ hash1;

    assert_eq!(result1, result2);
    assert_eq!(result2, result3);
}

#[test]
fn zobrist_castle_different_states() {
    ensure_initialized();

    let castle_hashes = ZOBRIST_CASTLE_HASH.get().unwrap();

    // Some specific castle state comparisons
    let no_castle = castle_hashes[0]; // 0b0000
    let wk_only = castle_hashes[1]; // 0b0001 (white kingside)
    let wq_only = castle_hashes[2]; // 0b0010 (white queenside)
    let all_castle = castle_hashes[15]; // 0b1111 (all)

    // All should be different
    assert_ne!(no_castle, wk_only);
    assert_ne!(no_castle, wq_only);
    assert_ne!(no_castle, all_castle);
    assert_ne!(wk_only, wq_only);
    assert_ne!(wk_only, all_castle);
    assert_ne!(wq_only, all_castle);
}

#[test]
fn zobrist_hash_distribution_is_random() {
    ensure_initialized();

    let hash_table = ZOBRIST_HASH_TABLE.get().unwrap();
    let mut hashes = Vec::new();

    // Collect sample hashes
    for side in all_sides() {
        for piece in valid_pieces() {
            for square in sample_squares() {
                hashes.push(hash_table[side as usize][piece as usize][square as usize]);
            }
        }
    }

    // Check that bits are roughly evenly distributed
    let mut bit_counts = [0u32; 64];
    for hash in &hashes {
        for bit in 0..64 {
            if (hash >> bit) & 1 == 1 {
                bit_counts[bit] += 1;
            }
        }
    }

    // Each bit should be set roughly half the time
    // With 96 samples, expect ~48 ± reasonable variance
    let sample_count = hashes.len() as f64;
    for (bit, &count) in bit_counts.iter().enumerate() {
        let ratio = count as f64 / sample_count;
        assert!(
            ratio > 0.25 && ratio < 0.75,
            "Bit {} set ratio {:.2} is not random-ish (expected ~0.5)",
            bit,
            ratio
        );
    }
}
