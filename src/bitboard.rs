use crate::{
    constants::{NUM_FILES, NUM_RANKS},
    types::{BitBoard, Square},
};

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
        self.0 &= self.0 - 1; // Clear least significant bit
        bit_position
    }

    /// Returns the next bit, without mutating the original
    pub fn next_bit(&self) -> u8 {
        let mut copy = *self; // BitBoard is Copy
        copy.next_bit_mut()
    }
}
