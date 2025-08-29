use crate::{
    bitboard::BitBoard,
    constants::{
        BISHOP_CAPTURE_SCORE, BISHOP_SCORE, CAPTURE_SCORE, CASTLE_MASK, COLUMN,
        FLIPPED_BOARD_SQUARE, GAME_STACK, KING_CAPTURE_SCORE, KING_ENDGAME_SCORE, KING_SCORE,
        KNIGHT_CAPTURE_SCORE, KNIGHT_SCORE, MAX_PLY, MOVE_STACK, NORTH_EAST_DIAGONAL,
        NORTH_WEST_DIAGONAL, NUM_PIECE_TYPES, NUM_SIDES, NUM_SQUARES, PASSED_SCORE,
        PAWN_CAPTURE_SCORE, PAWN_SCORE, QUEEN_CAPTURE_SCORE, QUEEN_SCORE, REVERSE_SQUARE,
        ROOK_CAPTURE_SCORE, ROOK_SCORE, ROW,
    },
    types::{Board, Game, Move, Piece, Side, Square},
};

pub struct Position {
    // DYNAMIC
    move_list: [Option<Move>; MOVE_STACK],
    first_move: [isize; MAX_PLY], // First move location for each ply in the move list (ply 1: 0, ply 2: first_move[1])
    game_list: [Option<Game>; GAME_STACK],
    fifty: u8,                     // Moves since last pawn move or capture (up to 100-ply)
    nodes: usize, // Total nodes (position in search tree) searched since start of turn
    ply: usize, // How many half-moves deep in current search tree; resets each search ("move" = both players take a turn)
    ply_from_start_of_game: usize, // Total half-moves from start of game (take-backs, fifty-move rule)
    board: Board,
    history_table: [[isize; NUM_SQUARES]; NUM_SQUARES], // [from][to] = score
    pawn_material_score: [usize; NUM_SIDES],
    piece_material_score: [usize; NUM_SIDES],
    castle: u8, // Castle permissions
    turn: Side,
    // STATIC
    side: Side,
    other_side: Side,
    square_score: [[[i32; NUM_SQUARES]; NUM_PIECE_TYPES]; NUM_SIDES],
    king_endgame_score: [[i32; NUM_SQUARES]; NUM_SIDES],
    passed_pawns_score: [[i32; NUM_SQUARES]; NUM_SIDES], // Score for 7th rank is built into `square_score`
    bit_between: [[BitBoard; NUM_SQUARES]; NUM_SQUARES], // &'ed with `bit_all`. 0-result means nothing blocking the line
    bit_after: [[BitBoard; NUM_SQUARES]; NUM_SQUARES], // Square and those after it in vector are 0
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

    fn add_move(&mut self, from: Square, to: Square, move_count: &mut isize) {
        let move_ = Move {
            from,
            to,
            promote: None,
            score: self.history_table[from as usize][to as usize],
        };

        self.move_list[*move_count as usize] = Some(move_);
        *move_count += 1;
    }

    fn add_capture(&mut self, from: Square, to: Square, score: isize, move_count: &mut isize) {
        let move_ = Move {
            from,
            to,
            promote: None,
            score: score + CAPTURE_SCORE as isize,
        };

        self.move_list[*move_count as usize] = Some(move_);
        *move_count += 1;
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

    fn get_score_tables() -> (
        [[[i32; NUM_SQUARES]; NUM_PIECE_TYPES]; NUM_SIDES],
        [[i32; NUM_SQUARES]; NUM_SIDES],
        [[i32; NUM_SQUARES]; NUM_SIDES],
    ) {
        let mut square_score = [[[0; NUM_SQUARES]; NUM_PIECE_TYPES]; NUM_SIDES];
        let mut king_endgame_score = [[0; NUM_SQUARES]; NUM_SIDES];
        let mut passed_pawns_score = [[0; NUM_SQUARES]; NUM_SIDES];

        let (white, black) = (Side::White as usize, Side::Black as usize);
        let (pawn, knight, bishop, rook, queen, king) = (
            Piece::Pawn as usize,
            Piece::Knight as usize,
            Piece::Bishop as usize,
            Piece::Rook as usize,
            Piece::Queen as usize,
            Piece::King as usize,
        );

        for square in Square::iter() {
            // Square score
            let sq = square as usize;
            let flipped_sq = FLIPPED_BOARD_SQUARE[sq] as usize;

            square_score[white][pawn][sq] = PAWN_SCORE[sq] + Piece::Pawn.value();
            square_score[white][knight][sq] = KNIGHT_SCORE[sq] + Piece::Knight.value();
            square_score[white][bishop][sq] = BISHOP_SCORE[sq] + Piece::Bishop.value();
            square_score[white][rook][sq] = ROOK_SCORE[sq] + Piece::Rook.value();
            square_score[white][queen][sq] = QUEEN_SCORE[sq] + Piece::Queen.value();
            square_score[white][king][sq] = KING_SCORE[sq];

            square_score[black][pawn][sq] = PAWN_SCORE[flipped_sq] + Piece::Pawn.value();
            square_score[black][knight][sq] = KNIGHT_SCORE[flipped_sq] + Piece::Knight.value();
            square_score[black][bishop][sq] = BISHOP_SCORE[flipped_sq] + Piece::Bishop.value();
            square_score[black][rook][sq] = ROOK_SCORE[flipped_sq] + Piece::Rook.value();
            square_score[black][queen][sq] = QUEEN_SCORE[flipped_sq] + Piece::Queen.value();
            square_score[black][king][sq] = KING_SCORE[flipped_sq];

            // King endgame score
            king_endgame_score[white][sq] = KING_ENDGAME_SCORE[sq] - square_score[white][king][sq];
            king_endgame_score[black][sq] = KING_ENDGAME_SCORE[sq] - square_score[black][king][sq];

            // Passed pawns score
            passed_pawns_score[white][sq] = PASSED_SCORE[sq];
            passed_pawns_score[black][sq] = PASSED_SCORE[flipped_sq];
        }

        (square_score, king_endgame_score, passed_pawns_score)
    }

    fn new_position(&mut self) {
        self.piece_material_score = [0; NUM_SIDES];
        self.pawn_material_score = [0; NUM_SIDES];

        for square in Square::iter() {
            let piece = self.board.value[square as usize];

            if piece != Piece::Empty {
                self.board.add_piece(
                    if self.board.bit_units[Side::White as usize].is_bit_set(square) {
                        Side::White
                    } else {
                        Side::Black
                    },
                    piece,
                    square,
                );
            }
        }
    }

    fn display_move(from: Square, to: Square) {
        [from, to].iter().for_each(|&square| {
            print!(
                "{}{}",
                (COLUMN[square as usize] + b'a') as char,
                (ROW[square as usize] + b'1') as char
            );

            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        });
    }

    fn get_knight_moves() -> [BitBoard; NUM_SQUARES] {
        let mut bit_knight_moves = [BitBoard(0); NUM_SQUARES];

        for square in Square::iter() {
            if ROW[square as usize] < 6 && COLUMN[square as usize] < 7 {
                bit_knight_moves[square as usize].set_bit((square as i32 + 17).try_into().unwrap());
            }
            if ROW[square as usize] < 7 && COLUMN[square as usize] < 6 {
                bit_knight_moves[square as usize].set_bit((square as i32 + 10).try_into().unwrap());
            }
            if ROW[square as usize] < 6 && COLUMN[square as usize] > 0 {
                bit_knight_moves[square as usize].set_bit((square as i32 + 15).try_into().unwrap());
            }
            if ROW[square as usize] < 7 && COLUMN[square as usize] > 1 {
                bit_knight_moves[square as usize].set_bit((square as i32 + 6).try_into().unwrap());
            }
            if ROW[square as usize] > 1 && COLUMN[square as usize] < 7 {
                bit_knight_moves[square as usize].set_bit((square as i32 - 15).try_into().unwrap());
            }
            if ROW[square as usize] > 0 && COLUMN[square as usize] < 6 {
                bit_knight_moves[square as usize].set_bit((square as i32 - 6).try_into().unwrap());
            }
            if ROW[square as usize] > 1 && COLUMN[square as usize] > 0 {
                bit_knight_moves[square as usize].set_bit((square as i32 - 17).try_into().unwrap());
            }
            if ROW[square as usize] > 0 && COLUMN[square as usize] > 1 {
                bit_knight_moves[square as usize].set_bit((square as i32 - 10).try_into().unwrap());
            }
        }

        bit_knight_moves
    }

    fn get_king_moves() -> [BitBoard; NUM_SQUARES] {
        let mut bit_king_moves = [BitBoard(0); NUM_SQUARES];

        for square in Square::iter() {
            if COLUMN[square as usize] > 0 {
                bit_king_moves[square as usize].set_bit((square as i32 - 1).try_into().unwrap());
            }
            if COLUMN[square as usize] < 7 {
                bit_king_moves[square as usize].set_bit((square as i32 + 1).try_into().unwrap());
            }
            if ROW[square as usize] > 0 {
                bit_king_moves[square as usize].set_bit((square as i32 - 8).try_into().unwrap());
            }
            if ROW[square as usize] < 7 {
                bit_king_moves[square as usize].set_bit((square as i32 + 8).try_into().unwrap());
            }
            if COLUMN[square as usize] < 7 && ROW[square as usize] < 7 {
                bit_king_moves[square as usize].set_bit((square as i32 + 9).try_into().unwrap());
            }
            if COLUMN[square as usize] > 0 && ROW[square as usize] < 7 {
                bit_king_moves[square as usize].set_bit((square as i32 + 7).try_into().unwrap());
            }
            if COLUMN[square as usize] > 0 && ROW[square as usize] > 0 {
                bit_king_moves[square as usize].set_bit((square as i32 - 9).try_into().unwrap());
            }
            if COLUMN[square as usize] < 7 && ROW[square as usize] > 0 {
                bit_king_moves[square as usize].set_bit((square as i32 - 7).try_into().unwrap());
            }
        }

        bit_king_moves
    }

    fn get_queen_rook_bishop_moves() -> (
        [BitBoard; NUM_SQUARES],
        [BitBoard; NUM_SQUARES],
        [BitBoard; NUM_SQUARES],
    ) {
        let mut bit_queen_moves = [BitBoard(0); NUM_SQUARES];
        let mut bit_rook_moves = [BitBoard(0); NUM_SQUARES];
        let mut bit_bishop_moves = [BitBoard(0); NUM_SQUARES];

        for square in Square::iter() {
            for square_2 in Square::iter() {
                if square != square_2 {
                    if NORTH_WEST_DIAGONAL[square as usize]
                        == NORTH_WEST_DIAGONAL[square_2 as usize]
                        || NORTH_EAST_DIAGONAL[square as usize]
                            == NORTH_EAST_DIAGONAL[square_2 as usize]
                    {
                        bit_queen_moves[square as usize].set_bit(square_2);
                        bit_bishop_moves[square as usize].set_bit(square_2);
                    }

                    if ROW[square as usize] == ROW[square_2 as usize]
                        || COLUMN[square as usize] == COLUMN[square_2 as usize]
                    {
                        bit_queen_moves[square as usize].set_bit(square_2);
                        bit_rook_moves[square as usize].set_bit(square_2);
                    }
                }
            }
        }

        (bit_queen_moves, bit_rook_moves, bit_bishop_moves)
    }

    pub fn is_square_attacked_by_side(&self, side: Side, square: Square) -> bool {
        let bit_pieces = self.board.bit_pieces[side as usize];

        if (self.bit_pawn_defends[side as usize][square as usize].0
            & bit_pieces[Piece::Pawn as usize].0)
            != 0
        {
            return true;
        }

        if (self.bit_knight_moves[square as usize].0 & bit_pieces[Piece::Knight as usize].0) != 0 {
            return true;
        }

        let mut b1 = BitBoard(
            (self.bit_rook_moves[square as usize].0
                & (bit_pieces[Piece::Rook as usize].0 | bit_pieces[Piece::Queen as usize].0))
                | (self.bit_bishop_moves[square as usize].0
                    & (bit_pieces[Piece::Bishop as usize].0 | bit_pieces[Piece::Queen as usize].0)),
        );

        while b1.0 != 0 {
            let attacking_piece = b1.next_bit();

            if (self.bit_between[attacking_piece as usize][square as usize].0
                & self.board.bit_all.0)
                == 0
            {
                return true;
            }
        }

        if (self.bit_king_moves[square as usize].0 & bit_pieces[Piece::King as usize].0) != 0 {
            return true;
        }

        false
    }

    pub fn get_square_of_lowest_value_attacker_of_square(
        &self,
        side: Side,
        square: Square,
    ) -> Option<Square> {
        for pawn_capture in [
            self.bit_pawn_left_captures[side.opponent() as usize][square as usize],
            self.bit_pawn_right_captures[side.opponent() as usize][square as usize],
        ] {
            if (pawn_capture.0 & self.board.bit_pieces[side as usize][Piece::Pawn as usize].0) != 0
            {
                return Some(pawn_capture.into());
            }
        }

        let mut b1 = BitBoard(
            self.bit_knight_moves[square as usize].0
                & self.board.bit_pieces[side as usize][Piece::Knight as usize].0,
        );

        if b1.0 != 0 {
            return Some(BitBoard(b1.next_bit().into()).into());
        }

        for (piece, bit_moves) in [
            (Piece::Bishop, self.bit_bishop_moves),
            (Piece::Rook, self.bit_rook_moves),
            (Piece::Queen, self.bit_queen_moves),
        ] {
            let mut b1 = BitBoard(
                bit_moves[square as usize].0
                    & self.board.bit_pieces[side as usize][piece as usize].0,
            );

            while b1.0 != 0 {
                let attacking_piece = b1.next_bit();

                if (self.bit_between[attacking_piece as usize][square as usize].0
                    & self.board.bit_all.0)
                    == 0
                {
                    return Some(BitBoard(attacking_piece.into()).into());
                }
            }
        }

        let mut b1 = BitBoard(
            self.bit_king_moves[square as usize].0
                & self.board.bit_pieces[side as usize][Piece::King as usize].0,
        );

        if b1.0 != 0 {
            return Some(BitBoard(b1.next_bit().into()).into());
        }

        None
    }

    fn generate_en_passant_moves(&mut self, side: Side, move_count: &mut isize) {
        let last_game_entry = self.game_list[self.ply_from_start_of_game - 1 as usize];

        if let Some(entry) = last_game_entry {
            let last_square_opponent_moved_from = entry.from;
            let last_square_opponent_moved_to = entry.to;

            if self.board.value[last_square_opponent_moved_to as usize] == Piece::Pawn
                && (last_square_opponent_moved_from as i32 - last_square_opponent_moved_to as i32)
                    .abs()
                    == 16
            {
                // En passant from left side
                if COLUMN[last_square_opponent_moved_to as usize] > 0
                    && self.board.bit_pieces[side as usize][Piece::Pawn as usize].is_bit_set(
                        (last_square_opponent_moved_to as i32 - 1)
                            .try_into()
                            .expect("Failed to convert square index to Square"),
                    )
                {
                    self.add_capture(
                        (last_square_opponent_moved_to as i32 - 1)
                            .try_into()
                            .expect("Failed to convert square index to Square"),
                        (last_square_opponent_moved_to as i32
                            + self.pawn_plus_index[side as usize]
                                [last_square_opponent_moved_to as usize])
                            .try_into()
                            .expect("Failed to convert square index to Square"),
                        10,
                        move_count,
                    );
                }

                // En passant from right side
                if COLUMN[last_square_opponent_moved_to as usize] < 7
                    && self.board.bit_pieces[side as usize][Piece::Pawn as usize].is_bit_set(
                        (last_square_opponent_moved_to as i32 + 1)
                            .try_into()
                            .expect("Failed to convert square index to Square"),
                    )
                {
                    self.add_capture(
                        (last_square_opponent_moved_to as i32 + 1)
                            .try_into()
                            .expect("Failed to convert square index to Square"),
                        (last_square_opponent_moved_to as i32
                            + self.pawn_plus_index[side as usize]
                                [last_square_opponent_moved_to as usize])
                            .try_into()
                            .expect("Failed to convert square index to Square"),
                        10,
                        move_count,
                    );
                }
            }
        }
    }

    fn generate_castle_moves(&mut self, side: Side, move_count: &mut isize) {
        match side {
            Side::White => {
                // Kingside
                if self.castle & 1 != 0
                    && (self.bit_between[Square::H1 as usize][Square::E1 as usize].0
                        & self.board.bit_all.0)
                        == 0
                {
                    self.add_move(Square::E1, Square::G1, move_count);
                }
                // Queenside
                if self.castle & 2 != 0
                    && (self.bit_between[Square::A1 as usize][Square::E1 as usize].0
                        & self.board.bit_all.0)
                        == 0
                {
                    self.add_move(Square::E1, Square::C1, move_count);
                }
            }
            Side::Black => {
                // Kingside
                if self.castle & 4 != 0
                    && (self.bit_between[Square::E8 as usize][Square::H8 as usize].0
                        & self.board.bit_all.0)
                        == 0
                {
                    self.add_move(Square::E8, Square::G8, move_count);
                }
                // Queenside
                if self.castle & 8 != 0
                    && (self.bit_between[Square::E8 as usize][Square::A8 as usize].0
                        & self.board.bit_all.0)
                        == 0
                {
                    self.add_move(Square::E8, Square::C8, move_count);
                }
            }
        }
    }

    fn generate_king_captures(&mut self, side: Side, king_square: u8, move_count: &mut isize) {
        let mut king_captures = BitBoard(
            self.bit_king_moves[king_square as usize].0
                & self.board.bit_units[side.opponent() as usize].0,
        );

        while king_captures.0 != 0 {
            let square_to = king_captures.next_bit();

            self.add_capture(
                king_square
                    .try_into()
                    .expect("Failed to convert king_square to Square"),
                square_to
                    .try_into()
                    .expect("Failed to convert square_to to Square"),
                KING_CAPTURE_SCORE[self.board.value[square_to as usize] as usize] as isize,
                move_count,
            );
        }
    }

    fn generate_moves_and_captures(&mut self, side: Side) {
        let mut left_pawn_captures;
        let mut right_pawn_captures;
        let mut unblocked_pawns;

        let mut move_count = self.first_move[self.ply];

        self.generate_en_passant_moves(side, &mut move_count);
        self.generate_castle_moves(side, &mut move_count);

        // Pawns
        match side {
            Side::White => {
                left_pawn_captures = BitBoard(
                    self.board.bit_pieces[side as usize][Piece::Pawn as usize].0
                        & ((self.board.bit_units[side.opponent() as usize].0 & self.not_h_file.0)
                            >> 7),
                );
                right_pawn_captures = BitBoard(
                    self.board.bit_pieces[side as usize][Piece::Pawn as usize].0
                        & ((self.board.bit_units[side.opponent() as usize].0 & self.not_a_file.0)
                            >> 9),
                );
                unblocked_pawns = BitBoard(
                    self.board.bit_pieces[side as usize][Piece::Pawn as usize].0
                        & !(self.board.bit_all.0 >> 8),
                );
            }
            Side::Black => {
                left_pawn_captures = BitBoard(
                    self.board.bit_pieces[side as usize][Piece::Pawn as usize].0
                        & ((self.board.bit_units[side.opponent() as usize].0 & self.not_h_file.0)
                            << 9),
                );
                right_pawn_captures = BitBoard(
                    self.board.bit_pieces[side as usize][Piece::Pawn as usize].0
                        & ((self.board.bit_units[side.opponent() as usize].0 & self.not_a_file.0)
                            << 7),
                );
                unblocked_pawns = BitBoard(
                    self.board.bit_pieces[side as usize][Piece::Pawn as usize].0
                        & !(self.board.bit_all.0 << 8),
                );
            }
        }

        while left_pawn_captures.0 != 0 {
            let square_from = left_pawn_captures.next_bit();
            let mut victim = self.bit_pawn_left_captures[side as usize][square_from as usize];

            self.add_capture(
                square_from
                    .try_into()
                    .expect("Failed to convert square_from to Square"),
                victim
                    .try_into()
                    .expect("Failed to convert victim to Square"),
                PAWN_CAPTURE_SCORE[self.board.value[victim.next_bit() as usize] as usize] as isize,
                &mut move_count,
            );
        }

        while right_pawn_captures.0 != 0 {
            let square_from = right_pawn_captures.next_bit();
            let mut victim = self.bit_pawn_right_captures[side as usize][square_from as usize];

            self.add_capture(
                square_from
                    .try_into()
                    .expect("Failed to convert square_from to Square"),
                victim
                    .try_into()
                    .expect("Failed to convert victim to Square"),
                PAWN_CAPTURE_SCORE[self.board.value[victim.next_bit() as usize] as usize] as isize,
                &mut move_count,
            );
        }

        while unblocked_pawns.0 != 0 {
            let square_from = unblocked_pawns.next_bit();
            let to = self.pawn_plus_index[side as usize][square_from as usize];

            self.add_move(
                square_from
                    .try_into()
                    .expect("Failed to convert square_from to Square"),
                to.try_into()
                    .expect("Failed to convert pawn plus index to Square"),
                &mut move_count,
            );

            if self.ranks[side as usize][square_from as usize] == 1
                && self.board.value
                    [self.pawn_double_index[side as usize][square_from as usize] as usize]
                    == Piece::Empty
            {
                self.add_move(
                    square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square"),
                    self.pawn_double_index[side as usize][square_from as usize]
                        .try_into()
                        .expect("Failed to convert pawn double index to Square"),
                    &mut move_count,
                );
            }
        }

        // Knights
        let mut knights = BitBoard(self.board.bit_pieces[side as usize][Piece::Knight as usize].0);

        while knights.0 != 0 {
            let square_from = knights.next_bit();

            let mut knight_captures = BitBoard(
                self.bit_knight_moves[square_from as usize].0
                    & self.board.bit_units[side.opponent() as usize].0,
            );

            while knight_captures.0 != 0 {
                let square_to = knight_captures.next_bit();

                self.add_capture(
                    square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square"),
                    square_to
                        .try_into()
                        .expect("Failed to convert square_to to Square"),
                    KNIGHT_CAPTURE_SCORE[self.board.value[square_to as usize] as usize] as isize,
                    &mut move_count,
                );
            }

            let mut knight_moves =
                BitBoard(self.bit_knight_moves[square_from as usize].0 & !self.board.bit_all.0);

            while knight_moves.0 != 0 {
                let square_to = knight_moves.next_bit();

                self.add_move(
                    square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square"),
                    square_to
                        .try_into()
                        .expect("Failed to convert square_to to Square"),
                    &mut move_count,
                );
            }

            // Bishops, rooks, queens
            for (piece, bit_moves, capture_score) in [
                (Piece::Bishop, self.bit_bishop_moves, BISHOP_CAPTURE_SCORE),
                (Piece::Rook, self.bit_rook_moves, ROOK_CAPTURE_SCORE),
                (Piece::Queen, self.bit_queen_moves, QUEEN_CAPTURE_SCORE),
            ] {
                let mut pieces = BitBoard(self.board.bit_pieces[side as usize][piece as usize].0);

                while pieces.0 != 0 {
                    let square_from = pieces.next_bit();
                    let mut possible_moves = BitBoard(bit_moves[square_from as usize].0);

                    // Remove squares blocked by friendly units and squares after them
                    let mut moves_to_self_occupied_squares =
                        BitBoard(possible_moves.0 & self.board.bit_units[side as usize].0);

                    while moves_to_self_occupied_squares.0 != 0 {
                        let square_to = moves_to_self_occupied_squares.next_bit();

                        moves_to_self_occupied_squares.0 &=
                            self.bit_after[square_from as usize][square_to as usize].0;

                        possible_moves.0 &=
                            self.bit_after[square_from as usize][square_to as usize].0;
                    }

                    let mut possible_captures = BitBoard(
                        possible_moves.0 & self.board.bit_units[side.opponent() as usize].0,
                    );

                    while possible_captures.0 != 0 {
                        let square_to = possible_captures.next_bit();

                        if (self.bit_between[square_from as usize][square_to as usize].0
                            & self.board.bit_all.0)
                            == 0
                        {
                            self.add_capture(
                                square_from
                                    .try_into()
                                    .expect("Failed to convert square_from to Square"),
                                square_to
                                    .try_into()
                                    .expect("Failed to convert square_to to Square"),
                                capture_score[self.board.value[square_to as usize] as usize]
                                    as isize,
                                &mut move_count,
                            );
                        }

                        possible_captures.0 &=
                            self.bit_after[square_from as usize][square_to as usize].0;

                        possible_moves.0 &=
                            self.bit_after[square_from as usize][square_to as usize].0;
                    }

                    while possible_moves.0 != 0 {
                        let square_to = possible_moves.next_bit();

                        self.add_move(
                            square_from
                                .try_into()
                                .expect("Failed to convert square_from to Square"),
                            square_to
                                .try_into()
                                .expect("Failed to convert square_to to Square"),
                            &mut move_count,
                        );
                    }
                }
            }

            // King
            let king_square = self.board.bit_pieces[side as usize][Piece::King as usize].next_bit();

            self.generate_king_captures(side, king_square, &mut move_count);

            let mut king_moves =
                BitBoard(self.bit_king_moves[king_square as usize].0 & !self.board.bit_all.0);

            while king_moves.0 != 0 {
                let square_to = king_moves.next_bit();

                self.add_move(
                    king_square
                        .try_into()
                        .expect("Failed to convert king_square to Square"),
                    square_to
                        .try_into()
                        .expect("Failed to convert square_to to Square"),
                    &mut move_count,
                );
            }

            self.first_move[self.ply + 1] = move_count;
        }
    }

    pub fn generate_captures(&mut self, side: Side) {
        let mut move_count = self.first_move[self.ply];

        // Pawns
        let mut left_pawn_captures;
        let mut right_pawn_captures;

        match side {
            Side::White => {
                left_pawn_captures = BitBoard(
                    self.board.bit_pieces[side as usize][Piece::Pawn as usize].0
                        & ((self.board.bit_units[side.opponent() as usize].0 & self.not_h_file.0)
                            >> 7),
                );
                right_pawn_captures = BitBoard(
                    self.board.bit_pieces[side as usize][Piece::Pawn as usize].0
                        & ((self.board.bit_units[side.opponent() as usize].0 & self.not_a_file.0)
                            >> 9),
                );
            }
            Side::Black => {
                left_pawn_captures = BitBoard(
                    self.board.bit_pieces[side as usize][Piece::Pawn as usize].0
                        & ((self.board.bit_units[side.opponent() as usize].0 & self.not_h_file.0)
                            << 9),
                );
                right_pawn_captures = BitBoard(
                    self.board.bit_pieces[side as usize][Piece::Pawn as usize].0
                        & ((self.board.bit_units[side.opponent() as usize].0 & self.not_a_file.0)
                            << 7),
                );
            }
        }

        while left_pawn_captures.0 != 0 {
            let square_from = left_pawn_captures.next_bit();
            let mut victim = self.bit_pawn_left_captures[side as usize][square_from as usize];

            self.add_capture(
                square_from
                    .try_into()
                    .expect("Failed to convert square_from to Square"),
                victim
                    .try_into()
                    .expect("Failed to convert victim to Square"),
                PAWN_CAPTURE_SCORE[self.board.value[victim.next_bit() as usize] as usize] as isize,
                &mut move_count,
            );
        }

        while right_pawn_captures.0 != 0 {
            let square_from = right_pawn_captures.next_bit();
            let mut victim = self.bit_pawn_right_captures[side as usize][square_from as usize];

            self.add_capture(
                square_from
                    .try_into()
                    .expect("Failed to convert square_from to Square"),
                victim
                    .try_into()
                    .expect("Failed to convert victim to Square"),
                PAWN_CAPTURE_SCORE[self.board.value[victim.next_bit() as usize] as usize] as isize,
                &mut move_count,
            );
        }

        // Knights
        let mut knights = BitBoard(self.board.bit_pieces[side as usize][Piece::Knight as usize].0);

        while knights.0 != 0 {
            let square_from = knights.next_bit();

            let mut knight_captures = BitBoard(
                self.bit_knight_moves[square_from as usize].0
                    & self.board.bit_units[side.opponent() as usize].0,
            );

            while knight_captures.0 != 0 {
                let square_to = knight_captures.next_bit();

                self.add_capture(
                    square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square"),
                    square_to
                        .try_into()
                        .expect("Failed to convert square_to to Square"),
                    KNIGHT_CAPTURE_SCORE[self.board.value[square_to as usize] as usize] as isize,
                    &mut move_count,
                );
            }
        }

        // Bishops, rooks, queens
        for (piece, bit_moves, capture_score) in [
            (Piece::Bishop, self.bit_bishop_moves, BISHOP_CAPTURE_SCORE),
            (Piece::Rook, self.bit_rook_moves, ROOK_CAPTURE_SCORE),
            (Piece::Queen, self.bit_queen_moves, QUEEN_CAPTURE_SCORE),
        ] {
            let mut pieces = BitBoard(self.board.bit_pieces[side as usize][piece as usize].0);

            while pieces.0 != 0 {
                let attacking_square = pieces.next_bit();

                let mut possible_captures = BitBoard(
                    bit_moves[attacking_square as usize].0
                        & self.board.bit_units[side.opponent() as usize].0,
                );

                while possible_captures.0 != 0 {
                    let square_to = possible_captures.next_bit();

                    if (self.bit_between[attacking_square as usize][square_to as usize].0
                        & self.board.bit_all.0)
                        == 0
                    {
                        self.add_capture(
                            attacking_square
                                .try_into()
                                .expect("Failed to convert square_from to Square"),
                            square_to
                                .try_into()
                                .expect("Failed to convert square_to to Square"),
                            capture_score[self.board.value[square_to as usize] as usize] as isize,
                            &mut move_count,
                        );
                    }

                    possible_captures.0 &=
                        self.bit_after[attacking_square as usize][square_to as usize].0;
                }
            }
        }

        // King
        let king_square = self.board.bit_pieces[side as usize][Piece::King as usize].next_bit();

        self.generate_king_captures(side, king_square, &mut move_count);

        self.first_move[self.ply + 1] = move_count;
    }

    pub fn make_move(&mut self, from: Square, to: Square) -> bool {
        // Check for castling
        if (to as i32 - from as i32).abs() == 2 && self.board.value[from as usize] == Piece::King {
            if self.is_square_attacked_by_side(self.side.opponent(), from) {
                return false;
            }

            if to == Square::G1 {
                if self.is_square_attacked_by_side(self.side.opponent(), Square::F1) {
                    return false;
                }

                self.board
                    .update_piece(self.side, Piece::Rook, Square::H1, Square::F1);
            } else if to == Square::C1 {
                if self.is_square_attacked_by_side(self.side.opponent(), Square::D1) {
                    return false;
                }

                self.board
                    .update_piece(self.side, Piece::Rook, Square::A1, Square::D1);
            } else if to == Square::G8 {
                if self.is_square_attacked_by_side(self.side.opponent(), Square::F8) {
                    return false;
                }

                self.board
                    .update_piece(self.side, Piece::Rook, Square::H8, Square::F8);
            } else if to == Square::C8 {
                if self.is_square_attacked_by_side(self.side.opponent(), Square::D8) {
                    return false;
                }

                self.board
                    .update_piece(self.side, Piece::Rook, Square::A8, Square::D8);
            }
        }

        let mut game = self.game_list[self.ply_from_start_of_game].unwrap_or(Game::new());

        game.from = from;
        game.to = to;
        game.capture = self.board.value[to as usize];
        game.fifty = self.fifty;
        game.castle = self.castle;
        game.hash = self.board.hash.current_key;
        game.lock = self.board.hash.current_lock;

        // Update the castle permissions
        self.castle &= CASTLE_MASK[from as usize] & CASTLE_MASK[to as usize];

        self.ply += 1;
        self.ply_from_start_of_game += 1;
        self.fifty += 1;

        if self.board.value[from as usize] == Piece::Pawn {
            self.fifty = 0;

            // Handle en passant
            if self.board.value[to as usize] == Piece::Empty
                && COLUMN[from as usize] != COLUMN[to as usize]
            {
                self.board.remove_piece(
                    self.side.opponent(),
                    Piece::Pawn,
                    (to as i32 + REVERSE_SQUARE[self.side as usize])
                        .try_into()
                        .expect("Failed to convert square to Square"),
                );
            }
        }

        // Handle regular (non-en passant) captures
        if self.board.value[to as usize] != Piece::Empty {
            self.fifty = 0;

            self.board
                .remove_piece(self.side.opponent(), self.board.value[to as usize], to);
        }

        // Handle promotions
        if self.board.value[from as usize] == Piece::Pawn && [0, 7].contains(&ROW[to as usize]) {
            self.board.remove_piece(self.side, Piece::Pawn, from);
            self.board.add_piece(self.side, Piece::Queen, to);

            game.promote = Some(Piece::Queen);
        } else {
            self.board
                .update_piece(self.side, self.board.value[from as usize], from, to);

            game.promote = None;
        }

        let original_side = self.side;

        self.side = self.side.opponent();
        self.other_side = self.other_side.opponent();

        // Update the game list entry
        self.game_list[self.ply_from_start_of_game] = Some(game);

        let king_square = self.board.bit_pieces[original_side as usize][Piece::King as usize]
            .next_bit()
            .try_into()
            .expect("Failed to convert square to Square");

        if self.is_square_attacked_by_side(original_side.opponent(), king_square) {
            self.take_back_move();
            return false;
        }

        true
    }

    fn take_back_move(&mut self) {
        self.side = self.side.opponent();
        self.other_side = self.other_side.opponent();
        self.ply -= 1;
        self.ply_from_start_of_game -= 1;

        let game = self.game_list[self.ply_from_start_of_game].expect("No game to take back");

        let from = game.from;
        let to = game.to;
        let castle_permissions = game.castle;
        let fifty = game.fifty;

        // En passant
        if self.board.value[to as usize] == Piece::Pawn
            && game.capture == Piece::Empty
            && COLUMN[from as usize] != COLUMN[to as usize]
        {
            self.board.add_piece(
                self.side.opponent(),
                Piece::Pawn,
                (to as i32 + REVERSE_SQUARE[self.side as usize])
                    .try_into()
                    .expect("Failed to convert square to Square"),
            );
        }

        // Promotion
        if game.promote.is_some() {
            self.board.add_piece(self.side, Piece::Pawn, from);
            self.board.remove_piece(self.side, Piece::Queen, to);
        } else {
            // Regular undo of a non-promotion move
            self.board
                .update_piece(self.side, self.board.value[to as usize], to, from);
        }

        // Replace captured piece
        if game.capture != Piece::Empty {
            self.board.add_piece(self.side.opponent(), game.capture, to);
        }

        // Castling
        if (to as i32 - from as i32).abs() == 2 && self.board.value[from as usize] == Piece::King {
            if to == Square::G1 {
                self.board
                    .update_piece(self.side, Piece::Rook, Square::F1, Square::H1);
            } else if to == Square::C1 {
                self.board
                    .update_piece(self.side, Piece::Rook, Square::D1, Square::A1);
            } else if to == Square::G8 {
                self.board
                    .update_piece(self.side, Piece::Rook, Square::F8, Square::H8);
            } else if to == Square::C8 {
                self.board
                    .update_piece(self.side, Piece::Rook, Square::D8, Square::A8);
            }
        }
    }

    fn make_recapture(&mut self, from: Square, to: Square) -> bool {
        let mut game = self.game_list[self.ply_from_start_of_game].unwrap_or(Game::new());

        game.from = from;
        game.to = to;
        game.capture = self.board.value[to as usize];

        // Update the game list entry
        self.game_list[self.ply_from_start_of_game] = Some(game);

        self.ply += 1;
        self.ply_from_start_of_game += 1;

        self.board
            .remove_piece(self.side.opponent(), self.board.value[to as usize], to);
        self.board
            .update_piece(self.side, self.board.value[from as usize], from, to);

        let original_side = self.side;
        let original_opponent_side = self.side.opponent();

        self.side = self.side.opponent();
        self.other_side = self.other_side.opponent();

        // Undo upon check
        if self.is_square_attacked_by_side(
            original_opponent_side,
            self.board.bit_pieces[original_side as usize][Piece::King as usize].into(),
        ) {
            self.take_back_recapture();
            return false;
        }

        true
    }

    fn take_back_recapture(&mut self) {
        let original_side = self.side;
        let original_opponent_side = self.side.opponent();

        self.side = self.side.opponent();
        self.other_side = self.other_side.opponent();

        self.ply -= 1;
        self.ply_from_start_of_game -= 1;

        let game = self.game_list[self.ply_from_start_of_game].expect("No game to unmake");

        let from = game.from;
        let to = game.to;

        self.board.update_piece(
            original_opponent_side,
            self.board.value[to as usize],
            to,
            from,
        );
        self.board.add_piece(original_side, game.capture, to);
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

        let (square_score, king_endgame_score, passed_pawns_score) = Self::get_score_tables();

        let mut first_move = [-1; MAX_PLY];
        first_move[0] = 0;

        let (bit_queen_moves, bit_rook_moves, bit_bishop_moves) =
            Self::get_queen_rook_bishop_moves();

        Self {
            // Dynamic
            move_list: [None; MOVE_STACK],
            first_move,
            game_list: [None; GAME_STACK],
            fifty: 0,
            nodes: 0,
            ply: 0,
            ply_from_start_of_game: 0,
            board: Board::new(),
            history_table: [[0; NUM_SQUARES]; NUM_SQUARES],
            pawn_material_score: [0; NUM_SIDES],
            piece_material_score: [0; NUM_SIDES],
            castle: 0b1111, // All castling rights available
            turn: Side::White,
            // Static
            side: Side::White,
            other_side: Side::Black,
            square_score,
            king_endgame_score,
            passed_pawns_score,
            bit_between: Self::get_bit_between(),
            bit_after: Self::get_bit_after(),
            bit_pawn_all_captures,
            bit_pawn_left_captures,
            bit_pawn_right_captures,
            bit_pawn_defends,
            bit_pawn_moves: [[BitBoard(0); NUM_SQUARES]; NUM_SIDES],
            bit_knight_moves: Self::get_knight_moves(),
            bit_bishop_moves,
            bit_rook_moves,
            bit_queen_moves,
            bit_king_moves: Self::get_king_moves(),
            mask_passed,
            mask_path,
            mask,     // TODO: Are these actually needed?
            not_mask, // TODO: Are these actually needed?
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
