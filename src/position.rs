use crate::{
    constants::{
        BISHOP_CAPTURE_SCORE, BISHOP_SCORE, CAPTURE_SCORE, CASTLE_MASK, COLUMN,
        FLIPPED_BOARD_SQUARE, GAME_STACK, HASH_SCORE, ISOLATED_PAWN_SCORE, KING_CAPTURE_SCORE,
        KING_ENDGAME_SCORE, KING_SCORE, KINGSIDE_DEFENSE, KNIGHT_CAPTURE_SCORE, KNIGHT_SCORE,
        MAX_PLY, MOVE_STACK, NORTH_EAST_DIAGONAL, NORTH_WEST_DIAGONAL, NUM_PIECE_TYPES, NUM_SIDES,
        NUM_SQUARES, PASSED_SCORE, PAWN_CAPTURE_SCORE, PAWN_SCORE, QUEEN_CAPTURE_SCORE,
        QUEEN_SCORE, QUEENSIDE_DEFENSE, REVERSE_SQUARE, ROOK_CAPTURE_SCORE, ROOK_SCORE, ROW,
    },
    types::{BitBoard, Board, Game, Move, Piece, Side, Square},
    utils::get_time,
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
    stop_search: bool,
    best_move_from: Option<Square>, // Found from the search/hash
    best_move_to: Option<Square>,   // Found from the search/hash
    hash_from: Option<Square>,
    hash_to: Option<Square>,
    start_time: u64,
    stop_time: u64,
    max_time: u8,
    fixed_time: bool,
    max_depth: u16,
    fixed_depth: bool,
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
            let attacking_piece = b1.next_bit_mut();

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

    /// Returns the square of the lowest attacker of the given side
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

        let b1 = BitBoard(
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
                let attacking_piece = b1.next_bit_mut();

                if (self.bit_between[attacking_piece as usize][square as usize].0
                    & self.board.bit_all.0)
                    == 0
                {
                    return Some(BitBoard(attacking_piece.into()).into());
                }
            }
        }

        let b1 = BitBoard(
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
            let square_to = king_captures.next_bit_mut();

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
            let square_from = left_pawn_captures.next_bit_mut();
            let victim = self.bit_pawn_left_captures[side as usize][square_from as usize];

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
            let square_from = right_pawn_captures.next_bit_mut();
            let victim = self.bit_pawn_right_captures[side as usize][square_from as usize];

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
            let square_from = unblocked_pawns.next_bit_mut();
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
            let square_from = knights.next_bit_mut();

            let mut knight_captures = BitBoard(
                self.bit_knight_moves[square_from as usize].0
                    & self.board.bit_units[side.opponent() as usize].0,
            );

            while knight_captures.0 != 0 {
                let square_to = knight_captures.next_bit_mut();

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
                let square_to = knight_moves.next_bit_mut();

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
                    let square_from = pieces.next_bit_mut();
                    let mut possible_moves = BitBoard(bit_moves[square_from as usize].0);

                    // Remove squares blocked by friendly units and squares after them
                    let mut moves_to_self_occupied_squares =
                        BitBoard(possible_moves.0 & self.board.bit_units[side as usize].0);

                    while moves_to_self_occupied_squares.0 != 0 {
                        let square_to = moves_to_self_occupied_squares.next_bit_mut();

                        moves_to_self_occupied_squares.0 &=
                            self.bit_after[square_from as usize][square_to as usize].0;

                        possible_moves.0 &=
                            self.bit_after[square_from as usize][square_to as usize].0;
                    }

                    let mut possible_captures = BitBoard(
                        possible_moves.0 & self.board.bit_units[side.opponent() as usize].0,
                    );

                    while possible_captures.0 != 0 {
                        let square_to = possible_captures.next_bit_mut();

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
                        let square_to = possible_moves.next_bit_mut();

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
                let square_to = king_moves.next_bit_mut();

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
            let square_from = left_pawn_captures.next_bit_mut();
            let victim = self.bit_pawn_left_captures[side as usize][square_from as usize];

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
            let square_from = right_pawn_captures.next_bit_mut();
            let victim = self.bit_pawn_right_captures[side as usize][square_from as usize];

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
            let square_from = knights.next_bit_mut();

            let mut knight_captures = BitBoard(
                self.bit_knight_moves[square_from as usize].0
                    & self.board.bit_units[side.opponent() as usize].0,
            );

            while knight_captures.0 != 0 {
                let square_to = knight_captures.next_bit_mut();

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
                let attacking_square = pieces.next_bit_mut();

                let mut possible_captures = BitBoard(
                    bit_moves[attacking_square as usize].0
                        & self.board.bit_units[side.opponent() as usize].0,
                );

                while possible_captures.0 != 0 {
                    let square_to = possible_captures.next_bit_mut();

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

    /// Make a move and return success state.
    /// If unsuccessful, the move will be undone.
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

    fn evaluate_pawn(
        &self,
        side: Side,
        square: Square,
        kingside_pawns: &mut i32,
        queenside_pawns: &mut i32,
    ) -> i32 {
        let mut score: i32 = 0;

        if (self.mask_passed[side as usize][square as usize].0
            & self.board.bit_pieces[side.opponent() as usize][Piece::Pawn as usize].0)
            == 0
            && self.mask_path[side as usize][square as usize].0
                & self.board.bit_pieces[side as usize][Piece::Pawn as usize].0
                == 0
        {
            score += self.passed_pawns_score[side as usize][square as usize];
        }

        if self.mask_isolated[square as usize].0
            & self.board.bit_pieces[side as usize][Piece::Pawn as usize].0
            == 0
        {
            score += ISOLATED_PAWN_SCORE // Is negative
        }

        *kingside_pawns += KINGSIDE_DEFENSE[side as usize][square as usize];
        *queenside_pawns += QUEENSIDE_DEFENSE[side as usize][square as usize];

        score
    }

    fn evaluate_rook(&self, side: Side, square: Square) -> i32 {
        if self.mask_column[square as usize].0
            & self.board.bit_pieces[side as usize][Piece::Pawn as usize].0
            == 0
        {
            if self.mask_column[square as usize].0
                & self.board.bit_pieces[side.opponent() as usize][Piece::Pawn as usize].0
                == 0
            {
                return 20;
            }

            return 10;
        }

        0
    }

    /// Adds a score for each unit on the board.
    /// Optionally adds a score for king position if opponent has a queen.
    /// Returns side-to-move's score minus opponent's score.
    fn evaluate_position(&self) -> i32 {
        let mut score = [0, 0];

        let mut queenside_pawns = [0, 0];
        let mut kingside_pawns = [0, 0];

        for side in Side::iter() {
            // Pawns
            let mut pawns = self.board.bit_pieces[side as usize][Piece::Pawn as usize];

            while pawns.0 != 0 {
                let pawn_square = pawns.next_bit_mut();

                score[side as usize] += self.square_score[side as usize][Piece::Pawn as usize]
                    [pawn_square as usize]
                    + self.evaluate_pawn(
                        side,
                        pawn_square
                            .try_into()
                            .expect("Failed to convert pawn u8 to Square"),
                        &mut kingside_pawns[side as usize],
                        &mut queenside_pawns[side as usize],
                    );
            }

            // Knights
            let mut knights = self.board.bit_pieces[side as usize][Piece::Knight as usize];

            while knights.0 != 0 {
                let knight_square = knights.next_bit_mut();

                score[side as usize] += self.square_score[side as usize][Piece::Knight as usize]
                    [knight_square as usize];
            }

            // Bishops
            let mut bishops = self.board.bit_pieces[side as usize][Piece::Bishop as usize];

            while bishops.0 != 0 {
                let bishop_square = bishops.next_bit_mut();

                score[side as usize] += self.square_score[side as usize][Piece::Bishop as usize]
                    [bishop_square as usize];
            }

            // Rooks
            let mut rooks = self.board.bit_pieces[side as usize][Piece::Rook as usize];

            while rooks.0 != 0 {
                let rook_square = rooks.next_bit_mut();

                score[side as usize] += self.square_score[side as usize][Piece::Rook as usize]
                    [rook_square as usize]
                    + self.evaluate_rook(
                        side,
                        rook_square
                            .try_into()
                            .expect("Failed to convert rook u8 to Square"),
                    );
            }

            // Queens (can be multiple after promotions)
            let mut queens = self.board.bit_pieces[side as usize][Piece::Queen as usize];

            while queens.0 != 0 {
                let queen_square = queens.next_bit_mut();

                score[side as usize] +=
                    self.square_score[side as usize][Piece::Queen as usize][queen_square as usize];
            }

            // King
            let king_square = self.board.bit_pieces[side as usize][Piece::King as usize].next_bit();

            if self.board.bit_pieces[side.opponent() as usize][Piece::Queen as usize].0 == 0 {
                score[side as usize] += self.king_endgame_score[side as usize][king_square as usize]
            } else {
                if self.board.bit_pieces[side as usize][Piece::King as usize].0
                    & self.mask_kingside.0
                    != 0
                {
                    score[side as usize] += kingside_pawns[king_square as usize]
                } else if self.board.bit_pieces[side as usize][Piece::King as usize].0
                    & self.mask_queenside.0
                    != 0
                {
                    score[side as usize] += queenside_pawns[king_square as usize]
                }
            }
        }

        score[0] - score[1]
    }

    fn set_hash_move(&mut self) {
        for i in self.first_move[self.ply]..self.first_move[self.ply + 1] {
            if let Some(ref mut move_) = self.move_list[i as usize] {
                if move_.from == self.best_move_from.unwrap()
                    && move_.to == self.best_move_to.unwrap()
                {
                    move_.score = HASH_SCORE as isize;
                    return;
                }
            }
        }
    }

    fn display_principal_variation(&mut self, depth: u16) {
        self.best_move_from = self.hash_from;
        self.best_move_to = self.hash_to;

        for _ in 0..depth {
            if !self
                .board
                .hash
                .lookup(self.side, &mut self.hash_from, &mut self.hash_to)
            {
                break;
            }

            print!(" ");
            Position::display_move(self.hash_from.unwrap(), self.hash_to.unwrap());
            self.make_move(self.hash_from.unwrap(), self.hash_to.unwrap());
        }

        while self.ply > 0 {
            self.take_back_move();
        }
    }

    fn recapture_search(&mut self, mut from: Square, to: Square) -> i32 {
        let mut score = [0; 12];
        let mut capture_count = 0;
        let mut transaction_count = 0;

        // Even indexes contain opponent's captured piece values
        score[0] = self.board.value[to as usize].value();
        score[1] = self.board.value[from as usize].value();

        let mut total_score = 0;

        while capture_count < 10 {
            if !self.make_recapture(from, to) {
                break;
            }

            capture_count += 1;
            transaction_count += 1;
            self.nodes += 1;

            let lowest_value_attacking_square =
                // `make_recapture()` will toggle the side
                self.get_square_of_lowest_value_attacker_of_square(self.side, to);

            match lowest_value_attacking_square {
                Some(square) => {
                    score[capture_count + 1] = self.board.value[square as usize].value();

                    // Stop if capturing piece value is more that that of the captured piece + next attacker
                    if score[capture_count] > score[capture_count - 1] + score[capture_count + 1] {
                        capture_count -= 1;
                        break;
                    }

                    from = square;
                }
                None => {
                    break;
                }
            }
        }

        // Pruning
        while capture_count > 1 {
            if score[capture_count - 1] >= score[capture_count - 2] {
                capture_count -= 2;
            } else {
                break;
            }
        }

        for i in 0..capture_count {
            total_score += if i % 2 == 0 { score[i] } else { -score[i] };
        }

        while transaction_count > 0 {
            self.take_back_recapture();
            transaction_count -= 1;
        }

        total_score
    }

    fn capture_search(&mut self, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        let score = self.evaluate_position();

        if score > alpha {
            if score >= beta {
                return beta;
            }

            alpha = score;
        } else if score + Piece::Queen.value() < alpha {
            return alpha;
        }

        let mut score = 0;
        let mut best_move_index = 0;
        let mut best_score = 0;

        self.generate_captures(self.side);

        for move_index in self.first_move[self.ply]..self.first_move[self.ply + 1] {
            self.sort(move_index);

            let from = self.move_list[move_index as usize].unwrap().from;
            let to = self.move_list[move_index as usize].unwrap().to;

            if score + self.board.value[to as usize].value() < alpha {
                continue;
            }

            score = self.recapture_search(from, to);

            if score > best_score {
                best_score = score;
                best_move_index = move_index;
            }
        }

        if best_score > 0 {
            score += best_score;
        }

        if score > alpha {
            if score >= beta {
                if best_score > 0 {
                    self.board.hash.update_position_best_move(
                        self.side,
                        self.move_list[best_move_index as usize].unwrap(),
                    );
                }

                return beta;
            }

            return score;
        }

        alpha
    }

    fn check_if_time_is_exhausted(&mut self) {
        if (get_time() >= self.stop_time || (self.max_time < 50 && self.ply > 1))
            && !self.fixed_depth
            && self.ply > 1
        {
            self.stop_search = true;
            // longjmp(env, 0);
        }
    }

    fn sort(&mut self, from_index: isize) {
        let mut best_score = self.move_list[from_index as usize].unwrap().score;
        let mut best_score_index = from_index;

        for i in from_index + 1..self.first_move[self.ply + 1] {
            if self.move_list[i as usize].unwrap().score > best_score {
                best_score = self.move_list[i as usize].unwrap().score;
                best_score_index = i;
            }
        }

        let move_ = self.move_list[from_index as usize];
        self.move_list[from_index as usize] = self.move_list[best_score_index as usize];
        self.move_list[best_score_index as usize] = move_;
    }

    /// Search backward for an identical position (repetition).
    /// Positions are identical if the key and lock are the same.
    fn search_backward_for_identical_position(&self) -> bool {
        let mut cur = self.ply_from_start_of_game.saturating_sub(4);
        let end = self
            .ply_from_start_of_game
            .saturating_sub(self.fifty as usize);

        while cur >= end {
            if self.game_list[cur].unwrap().hash == self.board.hash.current_key
                && self.game_list[cur].unwrap().lock == self.board.hash.current_lock
            {
                return true;
            }

            cur -= 2;
        }

        false
    }

    /// Main part of the search.
    /// Alpha is the player's best score found so far.
    /// Beta is the opponent's best score found so far.
    fn search(&mut self, mut alpha: i32, beta: i32, depth: u16) -> i32 {
        // Stop if the position is a repeat
        if self.ply > 0 && self.search_backward_for_identical_position() {
            return 0;
        }

        // If depth has run out, the capture search is performed
        if depth == 0 {
            return self.capture_search(alpha, beta);
        }

        self.nodes += 1;

        // Check the time every 4,096 positions (efficient bitwise AND operation)
        if self.nodes & 4096 == 0 {
            self.check_if_time_is_exhausted();
        }

        if self.ply > MAX_PLY - 2 {
            return self.evaluate_position();
        }

        let mut best_move: Option<Move> = None;
        let mut best_score = -10001;

        let in_check = self.is_square_attacked_by_side(
            self.side.opponent(),
            self.board.bit_pieces[self.side as usize][Piece::King as usize]
                .next_bit()
                .try_into()
                .expect("Failed to convert square to Square"),
        );

        self.generate_moves_and_captures(self.side);

        // If the position is in the hash table, look up its best move
        if self
            .board
            .hash
            .lookup(self.side, &mut self.hash_from, &mut self.hash_to)
        {
            self.set_hash_move();
        }

        let mut legal_moves_count = 0;

        // Loop through the moves in order of their score
        for move_index in self.first_move[self.ply]..self.first_move[self.ply + 1] {
            self.sort(move_index);

            // Skip invalid moves (i.e. pinned pieces)
            if !self.make_move(
                self.move_list[move_index as usize].unwrap().from,
                self.move_list[move_index as usize].unwrap().to,
            ) {
                continue;
            }

            legal_moves_count += 1;

            let next_depth = match self.is_square_attacked_by_side(
                self.side.opponent(),
                self.board.bit_pieces[self.side as usize][Piece::King as usize]
                    .next_bit()
                    .try_into()
                    .expect("Failed to convert square to Square"),
            ) {
                true => depth,
                false => {
                    if self.move_list[move_index as usize].unwrap().score > CAPTURE_SCORE as isize
                        || legal_moves_count == 1
                        || in_check
                    {
                        depth - 1
                    } else if self.move_list[move_index as usize].unwrap().score > 0 {
                        depth - 2
                    } else {
                        depth - 3
                    }
                }
            };

            // Get score for opponent's next move
            let score = -self.search(-beta, -alpha, next_depth);

            self.take_back_move();

            if score > best_score {
                best_score = score;
                best_move = self.move_list[move_index as usize];
            }

            if score > alpha {
                // Beta cutoff - score is too good; opponent won't allow this position
                if score >= beta {
                    let move_ = self.move_list[move_index as usize].unwrap();

                    // Check if it's a "quiet" move
                    // These moves need a history to distinguish between seemingly similar moves
                    if move_.to.as_bit() & self.board.bit_all.0 == 0 {
                        // Add to history table
                        self.history_table[move_.from as usize][move_.to as usize] +=
                            depth as isize;
                    }

                    self.board.hash.update_position_best_move(self.side, move_);
                    return beta;
                }

                alpha = score;
            }
        }

        // Either checkmate or stalemate
        if legal_moves_count == 0 {
            match self.is_square_attacked_by_side(
                self.side.opponent(),
                self.board.bit_pieces[self.side as usize][Piece::King as usize]
                    .next_bit()
                    .try_into()
                    .expect("Failed to convert square to Square"),
            ) {
                true => return -10000 + self.ply as i32, // TODO: Improve safety, though likely irrelevant
                false => return 0,
            }
        }

        // Draw by 50-move rule
        if self.fifty >= 100 {
            return 0;
        }

        self.board
            .hash
            .update_position_best_move(self.side, best_move.unwrap());

        alpha
    }

    /// Launch the search.
    /// Iterates until maximum depth is reached or the allotted time runs out.
    pub fn think(&mut self) {
        self.stop_search = false;

        // setjmp(env);

        if self.stop_search {
            while self.ply > 0 {
                self.take_back_move();
                return;
            }
        }

        if !self.fixed_time {
            // Halve allotted time for check-evasion or recapture moves
            if self.is_square_attacked_by_side(
                self.side.opponent(),
                self.board.bit_pieces[self.side as usize][Piece::King as usize]
                    .next_bit()
                    .try_into()
                    .expect("Failed to convert square to Square"),
            ) {
                self.max_time /= 2;
            }
        }

        self.start_time = get_time();
        self.stop_time = self.start_time + self.max_time as u64;

        self.ply = 0;
        self.nodes = 0;

        self.new_position();

        self.history_table = [[0; NUM_SQUARES]; NUM_SQUARES];

        println!("PLY      NODES  SCORE  PV");

        for depth in 1..=self.max_depth {
            if !self.fixed_depth && self.max_depth > 1 {
                if self.fixed_time {
                    if get_time() >= self.start_time + self.max_time as u64 {
                        self.stop_search = true;
                        return;
                    }
                } else if get_time() >= self.start_time + self.max_time as u64 / 4 {
                    self.stop_search = true;
                    return;
                }
            }

            let score = self.search(-10000, 10000, depth);

            print!(
                "{:>3} {:>8} {:>6} {}",
                depth,
                score,
                (get_time() - self.start_time) / 10,
                self.nodes,
            );

            if self
                .board
                .hash
                .lookup(self.side, &mut self.hash_from, &mut self.hash_to)
            {
                self.display_principal_variation(depth);
            } else {
                self.best_move_from = None;
                self.best_move_to = None;
            }

            println!();
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            if score > 9000 || score < -9000 {
                break;
            }
        }
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
            stop_search: false,
            best_move_from: None,
            best_move_to: None,
            hash_from: None,
            hash_to: None,
            start_time: 0,
            stop_time: 0,
            max_time: 0,
            fixed_time: false,
            max_depth: 0,
            fixed_depth: false,
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
