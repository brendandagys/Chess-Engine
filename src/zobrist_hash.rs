use rand::{Rng, thread_rng};
use std::sync::OnceLock;

use crate::{
    constants::{NUM_PIECE_TYPES, NUM_SIDES, NUM_SQUARES},
    types::{Piece, Side, Square},
};

// Global Zobrist random number tables for hashing
pub static ZOBRIST_HASH_TABLE: OnceLock<[[[u64; NUM_SQUARES]; NUM_PIECE_TYPES]; NUM_SIDES]> =
    OnceLock::new();

// Zobrist keys for position state
pub static ZOBRIST_SIDE_TO_MOVE_HASH: OnceLock<u64> = OnceLock::new();
pub static ZOBRIST_CASTLE_HASH: OnceLock<[u64; 16]> = OnceLock::new(); // 16 castle states (4 bits)
pub static ZOBRIST_EN_PASSANT_HASH: OnceLock<[u64; 8]> = OnceLock::new(); // 8 files (A-H)

/// Generate a random 64-bit number for Zobrist hashing
fn random_u64() -> u64 {
    thread_rng().r#gen()
}

/// Initialize Zobrist hash tables with random values (called once)
pub fn initialize_zobrist_hash_tables() {
    ZOBRIST_HASH_TABLE.get_or_init(|| {
        let mut hash_table = [[[0u64; NUM_SQUARES]; NUM_PIECE_TYPES]; NUM_SIDES];

        for side in Side::iter() {
            for piece in Piece::iter() {
                if piece == Piece::Empty {
                    continue;
                }
                for square in Square::iter() {
                    hash_table[side as usize][piece as usize][square as usize] = random_u64();
                }
            }
        }
        hash_table
    });

    ZOBRIST_SIDE_TO_MOVE_HASH.get_or_init(|| random_u64());

    ZOBRIST_CASTLE_HASH.get_or_init(|| {
        let mut castle_hash = [0u64; 16];
        for i in 0..16 {
            castle_hash[i] = random_u64();
        }
        castle_hash
    });

    ZOBRIST_EN_PASSANT_HASH.get_or_init(|| {
        let mut ep_hash = [0u64; 8];
        for i in 0..8 {
            ep_hash[i] = random_u64();
        }
        ep_hash
    });
}
