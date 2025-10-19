use crate::{
    constants::{
        INIT_BOARD, INIT_COLOR, NUM_FILES, NUM_PIECE_TYPES, NUM_RANKS, NUM_SIDES, NUM_SQUARES,
    },
    hash::Hash,
};

/// Right-most bit represents A1
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BitBoard(pub u64);

impl BitBoard {
    #[allow(dead_code)]
    pub fn print(&self) {
        for rank in (0..NUM_RANKS).rev() {
            for file in 0..NUM_FILES {
                let bit = (self.0 >> (rank * NUM_FILES + file)) & 1u64;
                print!("{} ", if bit == 1 { "1" } else { "." });
            }
            println!();
        }
        println!();
    }

    pub fn set_bit(&mut self, square: Square) {
        self.0 |= square.as_bit();
    }

    pub fn clear_bit(&mut self, square: Square) {
        self.0 &= !square.as_bit();
    }

    pub fn is_bit_set(&self, square: Square) -> bool {
        (self.0 & square.as_bit()) != 0
    }

    /// Returns the square index (0-63) of the least significant bit that is set
    /// Folding trick from chessprogramming.org
    /// https://www.chessprogramming.org/Matt_Taylor
    pub fn next_bit_mut(&mut self) -> u8 {
        if self.0 == 0 {
            return 64; // No bits set
        }

        let bit_position = self.0.trailing_zeros() as u8;
        self.0 &= self.0 - 1; // Clear the least significant bit
        bit_position
    }

    /// Returns the next bit, without mutating the original
    pub fn next_bit(&self) -> u8 {
        let mut copy = self.clone();
        copy.next_bit_mut()
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Square {
  A1 = 0, B1, C1, D1, E1, F1, G1, H1,
  A2, B2, C2, D2, E2, F2, G2, H2,
  A3, B3, C3, D3, E3, F3, G3, H3,
  A4, B4, C4, D4, E4, F4, G4, H4,
  A5, B5, C5, D5, E5, F5, G5, H5,
  A6, B6, C6, D6, E6, F6, G6, H6,
  A7, B7, C7, D7, E7, F7, G7, H7,
  A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    #[rustfmt::skip]
    pub fn iter() -> impl Iterator<Item = Square> {
        [
            Square::A1, Square::B1, Square::C1, Square::D1, Square::E1, Square::F1, Square::G1, Square::H1,
            Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2,
            Square::A3, Square::B3, Square::C3, Square::D3, Square::E3, Square::F3, Square::G3, Square::H3,
            Square::A4, Square::B4, Square::C4, Square::D4, Square::E4, Square::F4, Square::G4, Square::H4,
            Square::A5, Square::B5, Square::C5, Square::D5, Square::E5, Square::F5, Square::G5, Square::H5,
            Square::A6, Square::B6, Square::C6, Square::D6, Square::E6, Square::F6, Square::G6, Square::H6,
            Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7,
            Square::A8, Square::B8, Square::C8, Square::D8, Square::E8, Square::F8, Square::G8, Square::H8,
        ].into_iter()
    }
}

impl Square {
    #[inline]
    pub fn as_bit(self) -> u64 {
        1u64 << (self as u64)
    }
}

impl From<BitBoard> for Square {
    fn from(bitboard: BitBoard) -> Self {
        if bitboard.0.count_ones() != 1 {
            panic!("BitBoard must have exactly one bit set to convert to Square"); // TODO: Remove panic and use TryFrom
        }

        let index = bitboard.0.trailing_zeros() as u8;

        // SAFETY: We've verified index is in range 0-63, which matches our enum variants
        unsafe { std::mem::transmute(index) }
    }
}

impl TryFrom<i32> for Square {
    type Error = &'static str;

    /// Converts from a number representing the square index
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 || value > 63 {
            return Err("Square index out of range (must be 0-63)");
        }

        // SAFETY: We've verified value is in range 0-63, which matches our enum variants
        Ok(unsafe { std::mem::transmute(value as u8) })
    }
}

impl TryFrom<u8> for Square {
    type Error = &'static str;

    /// Converts from a number representing the square index
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 63 {
            return Err("Square index out of range (must be 0-63)");
        }

        // SAFETY: We've verified value is in range 0-63, which matches our enum variants
        Ok(unsafe { std::mem::transmute(value) })
    }
}

/// From white's viewpoint. Used in move generation.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    N = 0,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    Pawn = 0,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    Empty,
}

impl Piece {
    pub const fn value(self) -> i32 {
        match self {
            Piece::Pawn => 100,
            Piece::Knight => 300,
            Piece::Bishop => 300,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 10000,
            Piece::Empty => 0,
        }
    }

    pub fn iter() -> impl Iterator<Item = Piece> {
        [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
            Piece::Empty,
        ]
        .into_iter()
    }
}

impl TryFrom<u8> for Piece {
    type Error = &'static str;

    /// Converts from a number representing the piece
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 6 {
            return Err("Piece index out of range (must be 0-6)");
        }

        // SAFETY: We've verified value is in range 0-6, which matches our enum variants
        Ok(unsafe { std::mem::transmute(value) })
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    White = 0,
    Black = 1,
}

impl Side {
    pub fn iter() -> impl Iterator<Item = Side> {
        [Side::White, Side::Black].into_iter()
    }

    pub fn opponent(self) -> Side {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}

impl TryFrom<u8> for Side {
    type Error = &'static str;

    /// Converts from a number representing the piece
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 1 {
            return Err("Side index out of range (must be 0-1)");
        }

        // SAFETY: We've verified value is in range 0-1, which matches our enum variants
        Ok(unsafe { std::mem::transmute(value) })
    }
}

#[derive(Clone, Copy)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promote: Option<Piece>,
    pub score: isize, // Used when sorting moves. Higher scores are searched first.
}

#[derive(Clone, Copy)]
pub struct Game {
    pub from: Square,
    pub to: Square,
    pub promote: Option<Piece>,
    pub capture: Piece,              // Can be an empty piece
    pub fifty: u8,                   // Moves since last pawn move or capture (up to 100-ply)
    pub castle: u8,                  // Castle permissions
    pub hash: u64, // Zobrist hash key for position comparison and repetition detection
    pub en_passant_file: Option<u8>, // File (0-7) where en passant is available, if any
}

impl Game {
    pub fn new() -> Self {
        Self {
            from: Square::A1,
            to: Square::A1,
            promote: None,
            capture: Piece::Empty,
            fifty: 0,
            castle: 0,
            hash: 0,
            en_passant_file: None,
        }
    }
}

pub struct Board {
    pub value: [Piece; NUM_SQUARES],
    pub bit_pieces: [[BitBoard; NUM_PIECE_TYPES]; NUM_SIDES], // [side][piece]
    pub bit_units: [BitBoard; NUM_SIDES],                     // [side]
    pub bit_all: BitBoard,
    pub hash: Hash,
}

impl Board {
    pub fn new() -> Self {
        let mut board = [Piece::Empty; NUM_SQUARES];

        let mut bit_pieces = [[BitBoard(0); NUM_PIECE_TYPES]; NUM_SIDES];
        let mut bit_units = [BitBoard(0); NUM_SIDES];
        let mut bit_all = BitBoard(0);

        let mut hash = Hash::new();

        for square in Square::iter() {
            let piece = Piece::try_from(INIT_BOARD[square as usize]).unwrap();
            let side_val = INIT_COLOR[square as usize];

            if piece != Piece::Empty && side_val < 2 {
                let side = Side::try_from(side_val).unwrap();
                board[square as usize] = piece;
                hash.toggle_piece(side, piece, square);
                bit_pieces[side as usize][piece as usize].set_bit(square);
                bit_units[side as usize].set_bit(square);
                bit_all.set_bit(square);
            }
        }

        Self {
            value: board,
            bit_pieces,
            bit_units,
            bit_all,
            hash,
        }
    }

    pub fn empty() -> Self {
        Self {
            value: [Piece::Empty; NUM_SQUARES],
            bit_pieces: [[BitBoard(0); NUM_PIECE_TYPES]; NUM_SIDES],
            bit_units: [BitBoard(0); NUM_SIDES],
            bit_all: BitBoard(0),
            hash: Hash::new(),
        }
    }

    pub fn add_piece(&mut self, side: Side, piece: Piece, square: Square) {
        if piece == Piece::Empty {
            return;
        }

        self.value[square as usize] = piece;
        self.hash.toggle_piece(side, piece, square);
        self.bit_pieces[side as usize][piece as usize].set_bit(square);
        self.bit_units[side as usize].set_bit(square);
        self.bit_all.set_bit(square);
    }

    pub fn remove_piece(&mut self, side: Side, piece: Piece, square: Square) {
        if piece == Piece::Empty {
            return;
        }

        self.hash.toggle_piece(side, piece, square);
        self.value[square as usize] = Piece::Empty;
        self.bit_pieces[side as usize][piece as usize].clear_bit(square);
        self.bit_units[side as usize].clear_bit(square);
        self.bit_all.clear_bit(square);
    }

    pub fn update_piece(&mut self, side: Side, piece: Piece, from: Square, to: Square) {
        if piece == Piece::Empty {
            return;
        }

        self.bit_units[side as usize].clear_bit(from);
        self.bit_units[side as usize].set_bit(to);

        self.bit_all.clear_bit(from);
        self.bit_all.set_bit(to);

        self.hash.toggle_piece(side, piece, from);
        self.hash.toggle_piece(side, piece, to);

        self.value[from as usize] = Piece::Empty;
        self.value[to as usize] = piece;

        self.bit_pieces[side as usize][piece as usize].clear_bit(from);
        self.bit_pieces[side as usize][piece as usize].set_bit(to);
    }
}
