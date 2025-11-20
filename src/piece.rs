use crate::types::Piece;

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

    pub const fn traditional_value(self) -> u8 {
        match self {
            Piece::Pawn => 1,
            Piece::Knight => 3,
            Piece::Bishop => 3,
            Piece::Rook => 5,
            Piece::Queen => 9,
            Piece::King => 0, // King is invaluable in traditional scoring
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
        Ok(unsafe { std::mem::transmute::<u8, Piece>(value) })
    }
}
