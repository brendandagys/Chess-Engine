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

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    White = 0,
    Black = 1,
}

pub struct Move {
    start: Square,
    destination: Square,
    promote: Option<Piece>,
    score: usize, // Used when sorting moves. Higher scores are searched first.
}

pub struct Game {
    start: Square,
    destination: Square,
    promote: Option<Piece>,
    capture: Option<Square>,
    fifty: u8,   // Moves since last pawn move or capture
    castle: u8,  // Castle permissions
    hash: usize, // Number to help test for repetition
    lock: usize, // Number to help test for repetition
}
