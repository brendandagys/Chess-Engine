use crate::{
    constants::NUM_FILES,
    types::{BitBoard, Square},
};

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

    #[inline]
    pub fn as_bit(self) -> u64 {
        1u64 << (self as u64)
    }

    /// 0-indexed (0-7)
    pub fn rank(self) -> u8 {
        self as u8 / NUM_FILES as u8
    }
    /// 0-indexed (0-7)
    pub fn file(self) -> u8 {
        (self as u8) % (NUM_FILES as u8)
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
        Ok(unsafe { std::mem::transmute::<u8, Square>(value as u8) })
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
        Ok(unsafe { std::mem::transmute::<u8, Square>(value) })
    }
}
