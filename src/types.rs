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

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    White = 0,
    Black = 1,
}

#[derive(Clone, Copy)]
pub struct Move {
    start: Square,
    destination: Square,
    promote: Option<Piece>,
    score: usize, // Used when sorting moves. Higher scores are searched first.
}

#[derive(Clone, Copy)]
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
