use crate::{
    constants::{LSB_64_TABLE, NUM_FILES, NUM_RANKS},
    types::Square,
};

// Right-most bit represents A1
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BitBoard(pub u64);

impl BitBoard {
    fn print(&self) {
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
    pub fn next_bit(&mut self) -> u8 {
        self.0 ^= self.0 - 1;
        let folded = self.0 ^ (self.0 >> 32);
        LSB_64_TABLE[(folded as usize) * (0x78291ACF >> 26)]
    }
}
