use crate::{
    constants::{
        COLUMN, GAME_STACK, LSB_64_TABLE, MOVE_STACK, NORTH_EAST_DIAGONAL, NORTH_WEST_DIAGONAL,
        NUM_FILES, NUM_PIECE_TYPES, NUM_RANKS, NUM_SIDES, NUM_SQUARES, ROW,
    },
    types::{Game, Move, Side, Square},
};

// Right-most bit represents A1
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BitBoard(u64);

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

    fn set_bit(&mut self, square: Square) {
        self.0 |= square.as_bit();
    }

    fn clear_bit(&mut self, square: Square) {
        self.0 &= !square.as_bit();
    }

    /// Folding trick from chessprogramming.org
    /// https://www.chessprogramming.org/Matt_Taylor
    fn next_bit(&mut self) -> u8 {
        self.0 ^= self.0 - 1;
        let folded = self.0 ^ (self.0 >> 32);
        LSB_64_TABLE[(folded as usize) * (0x78291ACF >> 26)]
    }
}

pub struct Position {
    // [side][piece]
    bit_pieces: [[BitBoard; NUM_PIECE_TYPES]; NUM_SIDES],
    // [side]
    bit_units: [BitBoard; NUM_SIDES],
    bit_all: BitBoard,
    // &'ed with `bit_all`. 0-result means nothing blocking the line
    bit_between: [[BitBoard; NUM_SQUARES]; NUM_SQUARES],
    // Square and those after it in vector are 0
    bit_after: [[BitBoard; NUM_SQUARES]; NUM_SQUARES],
    bit_pawn_all_captures: [[BitBoard; NUM_SQUARES]; NUM_SIDES],
    bit_pawn_left_captures: [[BitBoard; NUM_SQUARES]; NUM_SIDES],
    bit_pawn_right_captures: [[BitBoard; NUM_SQUARES]; NUM_SIDES],
    bit_pawn_defends: [[BitBoard; NUM_SQUARES]; NUM_SIDES],
    bit_pawn_moves: [[BitBoard; NUM_SQUARES]; NUM_SIDES],
    bit_knight_moves: [BitBoard; NUM_SQUARES],
    bit_bishop_moves: [BitBoard; NUM_SQUARES],
    bit_rook_moves: [BitBoard; NUM_SQUARES],
    bit_queen_moves: [BitBoard; NUM_SQUARES],
    bit_king_moves: [BitBoard; NUM_SQUARES],
    mask_passed: [[BitBoard; NUM_SQUARES]; NUM_SIDES],
    mask_path: [[BitBoard; NUM_SQUARES]; NUM_SIDES],
    mask: [BitBoard; NUM_SQUARES],
    not_mask: [BitBoard; NUM_SQUARES],
    mask_column: [BitBoard; NUM_SQUARES],
    mask_isolated: [BitBoard; NUM_SQUARES],
    mask_kingside: BitBoard,
    mask_queenside: BitBoard,
    not_a_file: BitBoard,
    not_h_file: BitBoard,
    pawn_plus_index: [[i32; NUM_SQUARES]; NUM_SIDES],
    pawn_double_index: [[i32; NUM_SQUARES]; NUM_SIDES],
    pawn_left_index: [[i32; NUM_SQUARES]; NUM_SIDES],
    pawn_right_index: [[i32; NUM_SQUARES]; NUM_SIDES],
    ranks: [[u8; NUM_SQUARES]; NUM_SIDES],
    move_list: [Option<Move>; MOVE_STACK],
    game_list: [Option<Game>; GAME_STACK],
}

impl Position {
    fn get_ranks() -> [[u8; NUM_SQUARES]; NUM_SIDES] {
        let mut ranks = [[0; NUM_SQUARES]; NUM_SIDES];

        for square in 0..NUM_SQUARES {
            ranks[Side::White as usize][square] = ROW[square];
            ranks[Side::Black as usize][square] = 7 - ROW[square];
        }

        ranks
    }

    fn get_pawn_masks() -> (
        [[BitBoard; NUM_SQUARES]; NUM_SIDES],
        [BitBoard; NUM_SQUARES],
        [[BitBoard; NUM_SQUARES]; NUM_SIDES],
        [BitBoard; NUM_SQUARES],
        [[i32; NUM_SQUARES]; NUM_SIDES],
        [[i32; NUM_SQUARES]; NUM_SIDES],
        [[BitBoard; NUM_SQUARES]; NUM_SIDES],
        [[BitBoard; NUM_SQUARES]; NUM_SIDES],
        [[BitBoard; NUM_SQUARES]; NUM_SIDES],
        [[BitBoard; NUM_SQUARES]; NUM_SIDES],
        [[i32; NUM_SQUARES]; NUM_SIDES],
        [[i32; NUM_SQUARES]; NUM_SIDES],
        BitBoard,
        BitBoard,
    ) {
        let mut mask_passed = [[BitBoard(0); NUM_SQUARES]; NUM_SIDES];
        let mut mask_isolated = [BitBoard(0); NUM_SQUARES];
        let mut mask_path = [[BitBoard(0); NUM_SQUARES]; NUM_SIDES];

        let mut mask_column = [BitBoard(0); NUM_SQUARES];

        let mut pawn_left_index = [[-1; NUM_SQUARES]; NUM_SIDES];
        let mut pawn_right_index = [[-1; NUM_SQUARES]; NUM_SIDES];

        let mut bit_pawn_left_captures = [[BitBoard(0); NUM_SQUARES]; NUM_SIDES];
        let mut bit_pawn_right_captures = [[BitBoard(0); NUM_SQUARES]; NUM_SIDES];
        let mut bit_pawn_all_captures = [[BitBoard(0); NUM_SQUARES]; NUM_SIDES];

        let mut bit_pawn_defends = [[BitBoard(0); NUM_SQUARES]; NUM_SIDES];

        let mut pawn_plus_index = [[-1; NUM_SQUARES]; NUM_SIDES];
        let mut pawn_double_index = [[-1; NUM_SQUARES]; NUM_SIDES];

        let mut not_a_file = BitBoard(0);
        let mut not_h_file = BitBoard(0);

        for square in Square::iter() {
            for square_2 in Square::iter() {
                // Passed pawns
                if COLUMN[square as usize].abs_diff(COLUMN[square_2 as usize]) < 2 {
                    if ROW[square as usize] < ROW[square_2 as usize] && ROW[square_2 as usize] < 7 {
                        mask_passed[Side::White as usize][square as usize].set_bit(square_2);
                    }

                    if ROW[square as usize] > ROW[square_2 as usize] && ROW[square_2 as usize] > 0 {
                        mask_passed[Side::Black as usize][square as usize].set_bit(square_2);
                    }
                }

                // Isolated pawns
                if COLUMN[square as usize].abs_diff(COLUMN[square_2 as usize]) == 1 {
                    mask_isolated[square as usize].set_bit(square_2);
                }

                // Pawn paths
                if COLUMN[square as usize] == COLUMN[square_2 as usize] {
                    if ROW[square as usize] < ROW[square_2 as usize] {
                        mask_path[Side::White as usize][square as usize].set_bit(square_2);
                    }

                    if ROW[square as usize] > ROW[square_2 as usize] {
                        mask_path[Side::Black as usize][square as usize].set_bit(square_2);
                    }
                }

                // Column mask
                if COLUMN[square as usize] == COLUMN[square_2 as usize] {
                    mask_column[square as usize].set_bit(square_2);
                }
            }

            // Pawn left
            if COLUMN[square as usize] > 0 {
                if ROW[square as usize] < 7 {
                    pawn_left_index[Side::White as usize][square as usize] = square as i32 + 7;

                    let pawn_left_index_casted = pawn_left_index[Side::White as usize]
                        [square as usize]
                        .try_into()
                        .expect("Failed to cast pawn left index from i32 to Square");

                    bit_pawn_all_captures[Side::White as usize][square as usize]
                        .set_bit(pawn_left_index_casted);
                    bit_pawn_left_captures[Side::White as usize][square as usize]
                        .set_bit(pawn_left_index_casted);
                }

                if ROW[square as usize] > 0 {
                    pawn_left_index[Side::Black as usize][square as usize] = square as i32 - 9;

                    let pawn_left_index_casted = pawn_left_index[Side::Black as usize]
                        [square as usize]
                        .try_into()
                        .expect("Failed to cast pawn left index from i32 to Square");

                    bit_pawn_all_captures[Side::Black as usize][square as usize]
                        .set_bit(pawn_left_index_casted);
                    bit_pawn_left_captures[Side::Black as usize][square as usize]
                        .set_bit(pawn_left_index_casted);
                }
            }

            // Pawn right
            if COLUMN[square as usize] < 7 {
                if ROW[square as usize] < 7 {
                    pawn_right_index[Side::White as usize][square as usize] = square as i32 + 9;

                    let pawn_right_index_casted = pawn_right_index[Side::White as usize]
                        [square as usize]
                        .try_into()
                        .expect("Failed to cast pawn right index from i32 to Square");

                    bit_pawn_all_captures[Side::White as usize][square as usize]
                        .set_bit(pawn_right_index_casted);
                    bit_pawn_right_captures[Side::White as usize][square as usize]
                        .set_bit(pawn_right_index_casted);
                }

                if ROW[square as usize] > 0 {
                    pawn_right_index[Side::Black as usize][square as usize] = square as i32 - 7;

                    let pawn_right_index_casted = pawn_right_index[Side::Black as usize]
                        [square as usize]
                        .try_into()
                        .expect("Failed to cast pawn right index from i32 to Square");

                    bit_pawn_all_captures[Side::Black as usize][square as usize]
                        .set_bit(pawn_right_index_casted);
                    bit_pawn_right_captures[Side::Black as usize][square as usize]
                        .set_bit(pawn_right_index_casted);
                }
            }

            // Pawn defends - pawns that defend this square
            bit_pawn_defends[Side::White as usize][square as usize] =
                bit_pawn_all_captures[Side::Black as usize][square as usize];

            bit_pawn_defends[Side::Black as usize][square as usize] =
                bit_pawn_all_captures[Side::White as usize][square as usize];

            // Pawn movements
            if ROW[square as usize] < 7 {
                pawn_plus_index[Side::White as usize][square as usize] = square as i32 + 8;
            }
            if ROW[square as usize] < 6 {
                pawn_double_index[Side::White as usize][square as usize] = square as i32 + 16;
            }

            if ROW[square as usize] > 0 {
                pawn_plus_index[Side::Black as usize][square as usize] = square as i32 - 8;
            }
            if ROW[square as usize] > 1 {
                pawn_double_index[Side::Black as usize][square as usize] = square as i32 - 16;
            }

            not_a_file = BitBoard(!mask_column[0].0);
            not_h_file = BitBoard(!mask_column[7].0);
        }

        (
            mask_passed,
            mask_isolated,
            mask_path,
            mask_column,
            pawn_left_index,
            pawn_right_index,
            bit_pawn_left_captures,
            bit_pawn_right_captures,
            bit_pawn_all_captures,
            bit_pawn_defends,
            pawn_plus_index,
            pawn_double_index,
            not_a_file,
            not_h_file,
        )
    }

    fn get_queenside_and_kingside_masks() -> (BitBoard, BitBoard) {
        let mut mask_queenside = BitBoard(0);
        let mut mask_kingside = BitBoard(0);

        for square in Square::iter() {
            if COLUMN[square as usize] < 2 {
                mask_queenside.set_bit(square)
            } else if COLUMN[square as usize] > 5 {
                mask_kingside.set_bit(square)
            }
        }

        (mask_queenside, mask_kingside)
    }

    fn get_base_masks() -> ([BitBoard; NUM_SQUARES], [BitBoard; NUM_SQUARES]) {
        let mut mask = [BitBoard(0); NUM_SQUARES];
        let mut not_mask = [BitBoard(!0); NUM_SQUARES];

        for square in Square::iter() {
            mask[square as usize].set_bit(square);
            not_mask[square as usize].clear_bit(square);
        }

        (mask, not_mask)
    }

    fn get_bit_between() -> [[BitBoard; NUM_SQUARES]; NUM_SQUARES] {
        let mut bit_between = [[BitBoard(0); NUM_SQUARES]; NUM_SQUARES];

        fn compute_between_and_set_bitboard(
            bitboard: &mut BitBoard,
            square: Square,
            square_2: Square,
            increment: u8,
        ) {
            let start_index = (square as u8).min(square_2 as u8) + increment;
            let end_index = (square as u8).max(square_2 as u8);

            let mut current_index = start_index;

            while current_index < end_index {
                bitboard.set_bit(
                    current_index
                        .try_into()
                        .expect("Failed to convert square index to Square"),
                );

                current_index += increment;
            }
        }

        for square in Square::iter() {
            for square_2 in Square::iter() {
                let bitboard = &mut bit_between[square as usize][square_2 as usize];
                // Same rank
                if ROW[square as usize] == ROW[square_2 as usize] {
                    compute_between_and_set_bitboard(bitboard, square, square_2, 1);
                }

                // Same file
                if COLUMN[square as usize] == COLUMN[square_2 as usize] {
                    compute_between_and_set_bitboard(bitboard, square, square_2, 8);
                }

                // Northwest diagonal
                if NORTH_WEST_DIAGONAL[square as usize] == NORTH_WEST_DIAGONAL[square_2 as usize] {
                    compute_between_and_set_bitboard(bitboard, square, square_2, 7);
                }

                // Northeast diagonal
                if NORTH_EAST_DIAGONAL[square as usize] == NORTH_EAST_DIAGONAL[square_2 as usize] {
                    compute_between_and_set_bitboard(bitboard, square, square_2, 9);
                }
            }
        }

        bit_between
    }

    fn get_bit_after() -> [[BitBoard; NUM_SQUARES]; NUM_SQUARES] {
        let mut bit_after = [[BitBoard(0); NUM_SQUARES]; NUM_SQUARES];

        fn compute_after_and_set_bitboard(
            bitboard: &mut BitBoard,
            square: Square,
            square_2: Square,
            edge_squares: (Square, Square),
            increments: (i32, i32),
        ) {
            let mut current_index = square_2 as i32;

            if (square as u8) < (square_2 as u8) {
                while current_index <= (edge_squares.1 as i32) {
                    bitboard.set_bit(
                        current_index
                            .try_into()
                            .expect("Failed to convert square index to Square"),
                    );

                    current_index += increments.1;
                }
            } else {
                while current_index >= (edge_squares.0 as i32) {
                    bitboard.set_bit(
                        current_index
                            .try_into()
                            .expect("Failed to convert square index to Square"),
                    );

                    current_index += increments.0;
                }
            }
        }

        for square in Square::iter() {
            for square_2 in Square::iter() {
                let bitboard = &mut bit_after[square as usize][square_2 as usize];

                // Same rank
                if ROW[square as usize] == ROW[square_2 as usize] {
                    compute_after_and_set_bitboard(
                        bitboard,
                        square,
                        square_2,
                        (
                            (ROW[square_2 as usize] * 8)
                                .try_into()
                                .expect("Failed to convert square index to Square"),
                            (ROW[square_2 as usize] * 8 + 7)
                                .try_into()
                                .expect("Failed to convert square index to Square"),
                        ),
                        (-1, 1),
                    );
                }

                // Same file
                if COLUMN[square as usize] == COLUMN[square_2 as usize] {
                    compute_after_and_set_bitboard(
                        bitboard,
                        square,
                        square_2,
                        (
                            COLUMN[square_2 as usize]
                                .try_into()
                                .expect("Failed to convert square index to Square"),
                            (COLUMN[square_2 as usize] + 56)
                                .try_into()
                                .expect("Failed to convert square index to Square"),
                        ),
                        (-8, 8),
                    );
                }

                // Northwest diagonal
                if NORTH_WEST_DIAGONAL[square as usize] == NORTH_WEST_DIAGONAL[square_2 as usize] {
                    compute_after_and_set_bitboard(
                        bitboard,
                        square,
                        square_2,
                        (
                            Position::get_edge(square, -7),
                            Position::get_edge(square, 7),
                        ),
                        (-7, 7),
                    );
                }

                // Northeast diagonal
                if NORTH_EAST_DIAGONAL[square as usize] == NORTH_EAST_DIAGONAL[square_2 as usize] {
                    compute_after_and_set_bitboard(
                        bitboard,
                        square,
                        square_2,
                        (
                            Position::get_edge(square, -9),
                            Position::get_edge(square, 9),
                        ),
                        (-9, 9),
                    );
                }
            }
        }

        // Zeros will represent the square and those after it
        for square in Square::iter() {
            for square_2 in Square::iter() {
                bit_after[square as usize][square_2 as usize].0 =
                    !bit_after[square as usize][square_2 as usize].0
            }
        }

        bit_after
    }

    /// Return the square at the border of the board when going
    /// in the direction of `plus` from the given square.
    fn get_edge(square: Square, plus: i32) -> Square {
        let mut square_index = square as i32;

        loop {
            square_index += plus;

            if COLUMN[square_index as usize] == 0
                || COLUMN[square_index as usize] == 7
                || ROW[square_index as usize] == 0
                || ROW[square_index as usize] == 7
            {
                break;
            }
        }

        square_index
            .try_into()
            .expect("Failed to convert square index to Square")
    }

    fn new() -> Self {
        let (mask_queenside, mask_kingside) = Self::get_queenside_and_kingside_masks();

        let (
            mask_passed,
            mask_isolated,
            mask_path,
            mask_column,
            pawn_left_index,
            pawn_right_index,
            bit_pawn_left_captures,
            bit_pawn_right_captures,
            bit_pawn_all_captures,
            bit_pawn_defends,
            pawn_plus_index,
            pawn_double_index,
            not_a_file,
            not_h_file,
        ) = Self::get_pawn_masks();

        let (mask, not_mask) = Self::get_base_masks();

        Self {
            move_list: [None; MOVE_STACK],
            game_list: [None; GAME_STACK],
            // Dynamic
            bit_pieces: [[BitBoard(0); NUM_PIECE_TYPES]; NUM_SIDES],
            bit_units: [BitBoard(0); NUM_SIDES],
            bit_all: BitBoard(0),
            // Static
            bit_between: Self::get_bit_between(),
            bit_after: Self::get_bit_after(),
            bit_pawn_all_captures,
            bit_pawn_left_captures,
            bit_pawn_right_captures,
            bit_pawn_defends,
            bit_pawn_moves: [[BitBoard(0); NUM_SQUARES]; NUM_SIDES],
            bit_knight_moves: [BitBoard(0); NUM_SQUARES],
            bit_bishop_moves: [BitBoard(0); NUM_SQUARES],
            bit_rook_moves: [BitBoard(0); NUM_SQUARES],
            bit_queen_moves: [BitBoard(0); NUM_SQUARES],
            bit_king_moves: [BitBoard(0); NUM_SQUARES],
            mask_passed,
            mask_path,
            mask,
            not_mask,
            mask_column,
            mask_isolated,
            mask_kingside,
            mask_queenside,
            not_a_file,
            not_h_file,
            pawn_plus_index,
            pawn_double_index,
            pawn_left_index,  // "Left" for both sides is toward A file
            pawn_right_index, // "Right" for both sides is toward H file
            ranks: Self::get_ranks(),
        }
    }
}
