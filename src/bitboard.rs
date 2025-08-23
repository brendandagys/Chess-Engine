use crate::constants::{NUM_FILES, NUM_PIECE_TYPES, NUM_RANKS, NUM_SIDES, NUM_SQUARES};

// Right-most bit represents A1
pub type BitBoard = u64;

pub fn from_array(arr: [[u8; NUM_FILES]; NUM_RANKS]) -> BitBoard {
    let mut bitboard = 0;

    for rank in 0..NUM_RANKS {
        for file in 0..NUM_FILES {
            if arr[rank][file] == 1 {
                bitboard |= 1u64 << (rank * 8 + file);
            }
        }
    }

    bitboard
}

pub fn print_bitboard(bitboard: BitBoard) {
    for rank in (0..NUM_RANKS).rev() {
        for file in 0..NUM_FILES {
            let bit = (bitboard >> (rank * NUM_FILES + file)) & 1;
            print!("{} ", if bit == 1 { "1" } else { "." });
        }
        println!();
    }
    println!();
}

pub struct Position {
    // [side][piece]
    pub bit_pieces: [[BitBoard; NUM_PIECE_TYPES]; NUM_SIDES],
    // [side]
    pub bit_units: [BitBoard; NUM_SIDES],
    pub bit_all: BitBoard,
    // &'ed with `bit_all`. 0-result means nothing blocking the line
    pub bit_between: [[BitBoard; NUM_SQUARES]; NUM_SQUARES],
}

impl Position {
    /// Creates a new empty position (all bitboards = 0)
    pub fn new() -> Self {
        Self {
            bit_pieces: [[0; NUM_PIECE_TYPES]; NUM_SIDES],
            bit_units: [0; NUM_SIDES],
            bit_all: 0,
            bit_between: [[0; NUM_SQUARES]; NUM_SQUARES],
        }
    }
}
