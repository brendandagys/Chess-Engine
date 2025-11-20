use crate::{
    constants::{NUM_PIECE_TYPES, NUM_SIDES, NUM_SQUARES},
    hash::Hash,
};

/// Right-most bit represents A1
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BitBoard(pub u64);

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

pub struct Board {
    pub value: [Piece; NUM_SQUARES],
    pub bit_pieces: [[BitBoard; NUM_PIECE_TYPES]; NUM_SIDES], // [side][piece]
    pub bit_units: [BitBoard; NUM_SIDES],                     // [side]
    pub bit_all: BitBoard,
    pub hash: Hash,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GameState {
    InProgress,
    Checkmate(Side), // Winner
    Stalemate,
    DrawByRepetition,
    DrawByFiftyMoveRule,
    DrawByInsufficientMaterial,
}

#[derive(Debug)]
pub struct MoveData {
    pub from: Square,
    pub to: Square,
    pub promote: Option<Piece>,
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

        Ok(unsafe { std::mem::transmute::<u8, Side>(value) })
    }
}

#[derive(Clone, Copy)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promote: Option<Piece>,
    pub score: isize, // Used when sorting moves. Higher scores are searched first.
}

#[derive(Clone, Copy, Debug)]
pub struct Game {
    pub from: Square,
    pub to: Square,
    pub promote: Option<Piece>,
    pub capture: Piece,                          // Can be an empty piece
    pub fifty: u8,  // Moves since last pawn move or capture (up to 100-ply)
    pub castle: u8, // Castle permissions
    pub hash: u64,  // Zobrist hash key for position comparison and repetition detection
    pub en_passant_file: Option<u8>, // File (0-7) where en passant is available, if any
    pub en_passant_adjacent_opponent_pawn: bool, // Whether opponent has a pawn adjacent to the en passant pawn
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
            en_passant_adjacent_opponent_pawn: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Difficulty {
    Beginner,
    Easy,
    Medium,
    Hard,
    Expert,
    Master,
}

impl Difficulty {
    pub fn max_depth(&self) -> u8 {
        match self {
            Difficulty::Beginner => 1,
            Difficulty::Easy => 2,
            Difficulty::Medium => 3,
            Difficulty::Hard => 4,
            Difficulty::Expert => 5,
            Difficulty::Master => 6,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Difficulty::Beginner => "Beginner",
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
            Difficulty::Expert => "Expert",
            Difficulty::Master => "Master",
        }
    }

    pub fn iter() -> impl Iterator<Item = Difficulty> {
        [
            Difficulty::Beginner,
            Difficulty::Easy,
            Difficulty::Medium,
            Difficulty::Hard,
            Difficulty::Expert,
            Difficulty::Master,
        ]
        .into_iter()
    }
}
