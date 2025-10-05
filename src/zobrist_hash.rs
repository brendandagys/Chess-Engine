use rand::{Rng, thread_rng};
use std::sync::OnceLock;

use crate::{
    constants::{HASH_SIZE, NUM_PIECE_TYPES, NUM_SIDES, NUM_SQUARES},
    types::{Board, Piece, Side, Square},
};

// Global hash tables for Zobrist hashing
pub static ZOBRIST_HASH_TABLE: OnceLock<[[[u64; NUM_SQUARES]; NUM_PIECE_TYPES]; NUM_SIDES]> =
    OnceLock::new();
pub static ZOBRIST_LOCK_TABLE: OnceLock<[[[u64; NUM_SQUARES]; NUM_PIECE_TYPES]; NUM_SIDES]> =
    OnceLock::new();

pub fn random(x: u64) -> u64 {
    if x == 0 {
        return 0;
    }

    thread_rng().gen_range(0..=x)
}

/// Initialize hash and lock tables with random values
pub fn initialize_zobrist_hash_tables() {
    if ZOBRIST_HASH_TABLE.get().is_some() && ZOBRIST_LOCK_TABLE.get().is_some() {
        return;
    }

    let mut hash_table = [[[0u64; NUM_SQUARES]; NUM_PIECE_TYPES]; NUM_SIDES];
    let mut lock_table = [[[0u64; NUM_SQUARES]; NUM_PIECE_TYPES]; NUM_SIDES];

    for side in Side::iter() {
        for piece in Piece::iter() {
            if piece == Piece::Empty {
                continue;
            }

            for square in Square::iter() {
                hash_table[side as usize][piece as usize][square as usize] =
                    random(HASH_SIZE as u64);
                lock_table[side as usize][piece as usize][square as usize] =
                    random(HASH_SIZE as u64);
            }
        }
    }

    ZOBRIST_HASH_TABLE
        .set(hash_table)
        .expect("Hash table already initialized");
    ZOBRIST_LOCK_TABLE
        .set(lock_table)
        .expect("Lock table already initialized");
}

pub fn get_zobrist_value(
    key_or_lock: OnceLock<[[[u64; NUM_SQUARES]; NUM_PIECE_TYPES]; NUM_SIDES]>,
    board: &Board,
) -> u64 {
    let mut key = 0u64;

    for square in Square::iter() {
        let piece = board.value[square as usize];

        if piece != Piece::Empty {
            let side = board.bit_units[Side::White as usize]
                .is_bit_set(square)
                .then(|| Side::White)
                .unwrap_or(Side::Black);

            key ^= key_or_lock.get().unwrap()[side as usize][piece as usize][square as usize];
        }
    }

    key
}
