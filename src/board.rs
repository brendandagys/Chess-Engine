use crate::{
    constants::{
        BISHOP_SCORE, COLUMN, FLIPPED_BOARD_SQUARE, INIT_BOARD, INIT_COLOR, KING_ENDGAME_SCORE,
        KING_SCORE, KNIGHT_SCORE, NORTH_EAST_DIAGONAL, NORTH_WEST_DIAGONAL, NUM_PIECE_TYPES,
        NUM_SIDES, NUM_SQUARES, PASSED_SCORE, PAWN_SCORE, QUEEN_SCORE, ROOK_SCORE, ROW,
    },
    hash::Hash,
    types::{BitBoard, Board, MoveData, Piece, Side, Square},
};

impl Board {
    pub fn new() -> Self {
        let mut board = [Piece::Empty; NUM_SQUARES];

        let mut bit_pieces = [[BitBoard(0); NUM_PIECE_TYPES]; NUM_SIDES];
        let mut bit_units = [BitBoard(0); NUM_SIDES];
        let mut bit_all = BitBoard(0);

        let mut hash = Hash::new();

        hash.toggle_side_to_move();
        hash.update_castle_rights(0, 0b1111);

        for square in Square::iter() {
            let piece = Piece::try_from(INIT_BOARD[square as usize]).unwrap();
            let side_val = INIT_COLOR[square as usize];

            if piece != Piece::Empty && side_val < 2 {
                let side = Side::try_from(side_val).unwrap();
                board[square as usize] = piece;
                hash.toggle_piece(side, piece, square);
                bit_pieces[side as usize][piece as usize].set_bit(square);
                bit_units[side as usize].set_bit(square);
                bit_all.set_bit(square);
            }
        }

        Self {
            value: board,
            bit_pieces,
            bit_units,
            bit_all,
            hash,
        }
    }

    pub fn empty() -> Self {
        Self {
            value: [Piece::Empty; NUM_SQUARES],
            bit_pieces: [[BitBoard(0); NUM_PIECE_TYPES]; NUM_SIDES],
            bit_units: [BitBoard(0); NUM_SIDES],
            bit_all: BitBoard(0),
            hash: Hash::new(),
        }
    }

    pub fn add_piece(&mut self, side: Side, piece: Piece, square: Square) {
        // TODO: Are these checks needed?
        if piece == Piece::Empty {
            return;
        }

        self.value[square as usize] = piece;
        self.hash.toggle_piece(side, piece, square);
        self.bit_pieces[side as usize][piece as usize].set_bit(square);
        self.bit_units[side as usize].set_bit(square);
        self.bit_all.set_bit(square);
    }

    pub fn remove_piece(&mut self, side: Side, piece: Piece, square: Square) {
        if piece == Piece::Empty {
            return;
        }

        self.hash.toggle_piece(side, piece, square);
        self.value[square as usize] = Piece::Empty;
        self.bit_pieces[side as usize][piece as usize].clear_bit(square);
        self.bit_units[side as usize].clear_bit(square);
        self.bit_all.clear_bit(square);
    }

    pub fn update_piece(&mut self, side: Side, piece: Piece, from: Square, to: Square) {
        if piece == Piece::Empty {
            return;
        }

        self.bit_units[side as usize].clear_bit(from);
        self.bit_units[side as usize].set_bit(to);

        self.bit_all.clear_bit(from);
        self.bit_all.set_bit(to);

        self.hash.toggle_piece(side, piece, from);
        self.hash.toggle_piece(side, piece, to);

        self.value[from as usize] = Piece::Empty;
        self.value[to as usize] = piece;

        self.bit_pieces[side as usize][piece as usize].clear_bit(from);
        self.bit_pieces[side as usize][piece as usize].set_bit(to);
    }

    /// Parse a UCI move string (e.g. "e2e4", "e7e8q") and return the from/to squares and promotion piece
    pub fn move_from_uci_string(move_str: &str) -> Result<MoveData, String> {
        if move_str.len() < 4 || move_str.len() > 5 {
            return Err(format!("Invalid move string length: {}", move_str));
        }

        let chars: Vec<char> = move_str.chars().collect();

        if chars[0] < 'a'
            || chars[0] > 'h'
            || chars[1] < '1'
            || chars[1] > '8'
            || chars[2] < 'a'
            || chars[2] > 'h'
            || chars[3] < '1'
            || chars[3] > '8'
        {
            return Err(format!("Invalid move format: {}", move_str));
        }

        let from_file = (chars[0] as u8 - b'a') as usize;
        let from_rank = (chars[1] as u8 - b'1') as usize;
        let to_file = (chars[2] as u8 - b'a') as usize;
        let to_rank = (chars[3] as u8 - b'1') as usize;

        let from_square = from_rank * 8 + from_file;
        let to_square = to_rank * 8 + to_file;

        let from = Square::try_from(from_square as u8)
            .map_err(|e| format!("Invalid from square: {}", e))?;
        let to =
            Square::try_from(to_square as u8).map_err(|e| format!("Invalid to square: {}", e))?;

        let promote = if chars.len() == 5 {
            match chars[4] {
                'q' => Some(Piece::Queen),
                'r' => Some(Piece::Rook),
                'b' => Some(Piece::Bishop),
                'n' => Some(Piece::Knight),
                _ => return Err(format!("Invalid promotion piece: {}", chars[4])),
            }
        } else {
            None
        };

        Ok(MoveData { from, to, promote })
    }

    /// Convert a move to UCI format (e.g., "e2e4", "e7e8q")
    pub fn move_to_uci_string(
        from: Square,
        to: Square,
        promote: Option<Piece>,
        pretty: bool,
    ) -> String {
        let from_file = (from as usize % 8) as u8 + b'a';
        let from_rank = (from as usize / 8) as u8 + b'1';
        let to_file = (to as usize % 8) as u8 + b'a';
        let to_rank = (to as usize / 8) as u8 + b'1';

        let mut result = format!(
            "{}{}{}{}{}",
            from_file as char,
            from_rank as char,
            if pretty { " -> " } else { "" },
            to_file as char,
            to_rank as char,
        );

        if let Some(piece) = promote {
            result.push(match piece {
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Rook => 'r',
                _ => 'q',
            });
        }

        result
    }

    pub fn get_ranks() -> [[u8; NUM_SQUARES]; NUM_SIDES] {
        let mut ranks = [[0; NUM_SQUARES]; NUM_SIDES];

        for square in 0..NUM_SQUARES {
            ranks[Side::White as usize][square] = ROW[square];
            ranks[Side::Black as usize][square] = 7 - ROW[square];
        }

        ranks
    }

    pub fn get_pawn_masks() -> (
        [[BitBoard; NUM_SQUARES]; NUM_SIDES],
        [BitBoard; NUM_SQUARES],
        [[BitBoard; NUM_SQUARES]; NUM_SIDES],
        [BitBoard; NUM_SQUARES],
        [[i32; NUM_SQUARES]; NUM_SIDES],
        [[i32; NUM_SQUARES]; NUM_SIDES],
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

        let white = Side::White as usize;
        let black = Side::Black as usize;

        for square in Square::iter() {
            let square_ = square as usize;
            for square_2 in Square::iter() {
                let square_2_ = square_2 as usize;
                // Passed pawns
                if COLUMN[square_].abs_diff(COLUMN[square_2_]) < 2 {
                    if ROW[square_] < ROW[square_2_] && ROW[square_2_] < 7 {
                        mask_passed[white][square_].set_bit(square_2);
                    }

                    if ROW[square_] > ROW[square_2_] && ROW[square_2_] > 0 {
                        mask_passed[black][square_].set_bit(square_2);
                    }
                }

                // Isolated pawns
                if COLUMN[square_].abs_diff(COLUMN[square_2_]) == 1 {
                    mask_isolated[square_].set_bit(square_2);
                }

                // Pawn paths
                if COLUMN[square_] == COLUMN[square_2_] {
                    if ROW[square_] < ROW[square_2_] {
                        mask_path[white][square_].set_bit(square_2);
                    }

                    if ROW[square_] > ROW[square_2_] {
                        mask_path[black][square_].set_bit(square_2);
                    }
                }

                // Column mask
                if COLUMN[square_] == COLUMN[square_2_] {
                    mask_column[square_].set_bit(square_2);
                }
            }

            // Pawn left
            if COLUMN[square_] > 0 {
                if ROW[square_] < 7 {
                    pawn_left_index[white][square_] = square as i32 + 7;

                    let pawn_left_index_casted = pawn_left_index[white][square_]
                        .try_into()
                        .expect("Failed to cast pawn left index from i32 to Square");

                    bit_pawn_all_captures[white][square_].set_bit(pawn_left_index_casted);
                    bit_pawn_left_captures[white][square_].set_bit(pawn_left_index_casted);
                }

                if ROW[square_] > 0 {
                    pawn_left_index[black][square_] = square as i32 - 9;

                    let pawn_left_index_casted = pawn_left_index[black][square_]
                        .try_into()
                        .expect("Failed to cast pawn left index from i32 to Square");

                    bit_pawn_all_captures[black][square_].set_bit(pawn_left_index_casted);
                    bit_pawn_left_captures[black][square_].set_bit(pawn_left_index_casted);
                }
            }

            // Pawn right
            if COLUMN[square_] < 7 {
                if ROW[square_] < 7 {
                    pawn_right_index[white][square_] = square as i32 + 9;

                    let pawn_right_index_casted = pawn_right_index[white][square_]
                        .try_into()
                        .expect("Failed to cast pawn right index from i32 to Square");

                    bit_pawn_all_captures[white][square_].set_bit(pawn_right_index_casted);
                    bit_pawn_right_captures[white][square_].set_bit(pawn_right_index_casted);
                }

                if ROW[square_] > 0 {
                    pawn_right_index[black][square_] = square as i32 - 7;

                    let pawn_right_index_casted = pawn_right_index[black][square_]
                        .try_into()
                        .expect("Failed to cast pawn right index from i32 to Square");

                    bit_pawn_all_captures[black][square_].set_bit(pawn_right_index_casted);
                    bit_pawn_right_captures[black][square_].set_bit(pawn_right_index_casted);
                }
            }

            // Pawn defends - pawns that defend this square
            bit_pawn_defends[white][square_] = bit_pawn_all_captures[black][square_];

            bit_pawn_defends[black][square_] = bit_pawn_all_captures[white][square_];

            // Pawn movements
            if ROW[square_] < 7 {
                pawn_plus_index[white][square_] = square as i32 + 8;
            }
            if ROW[square_] < 6 {
                pawn_double_index[white][square_] = square as i32 + 16;
            }

            if ROW[square_] > 0 {
                pawn_plus_index[black][square_] = square as i32 - 8;
            }
            if ROW[square_] > 1 {
                pawn_double_index[black][square_] = square as i32 - 16;
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
            bit_pawn_defends,
            pawn_plus_index,
            pawn_double_index,
            not_a_file,
            not_h_file,
        )
    }

    pub fn get_bit_between() -> [[BitBoard; NUM_SQUARES]; NUM_SQUARES] {
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

    /// Return the square at the border of the board when going
    /// in the direction of `plus` from the given square.
    fn get_edge(square: Square, plus: i32) -> Square {
        let mut square_index = square as i32;

        loop {
            square_index += plus;

            if square_index < 0 || square_index >= 64 {
                square_index -= plus; // Step back to last valid square
                break;
            }

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

    pub fn get_bit_after() -> [[BitBoard; NUM_SQUARES]; NUM_SQUARES] {
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
                        (Board::get_edge(square, -7), Board::get_edge(square, 7)),
                        (-7, 7),
                    );
                }

                // Northeast diagonal
                if NORTH_EAST_DIAGONAL[square as usize] == NORTH_EAST_DIAGONAL[square_2 as usize] {
                    compute_after_and_set_bitboard(
                        bitboard,
                        square,
                        square_2,
                        (Board::get_edge(square, -9), Board::get_edge(square, 9)),
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

    pub fn get_knight_moves() -> [BitBoard; NUM_SQUARES] {
        let mut bit_knight_moves = [BitBoard(0); NUM_SQUARES];

        for square in Square::iter() {
            let square_ = square as usize;

            if ROW[square as usize] < 6 && COLUMN[square as usize] < 7 {
                bit_knight_moves[square_].set_bit((square as i32 + 17).try_into().unwrap());
            }
            if ROW[square_] < 7 && COLUMN[square_] < 6 {
                bit_knight_moves[square_].set_bit((square as i32 + 10).try_into().unwrap());
            }
            if ROW[square_] < 6 && COLUMN[square_] > 0 {
                bit_knight_moves[square_].set_bit((square as i32 + 15).try_into().unwrap());
            }
            if ROW[square_] < 7 && COLUMN[square_] > 1 {
                bit_knight_moves[square_].set_bit((square as i32 + 6).try_into().unwrap());
            }
            if ROW[square_] > 1 && COLUMN[square_] < 7 {
                bit_knight_moves[square_].set_bit((square as i32 - 15).try_into().unwrap());
            }
            if ROW[square_] > 0 && COLUMN[square_] < 6 {
                bit_knight_moves[square_].set_bit((square as i32 - 6).try_into().unwrap());
            }
            if ROW[square_] > 1 && COLUMN[square_] > 0 {
                bit_knight_moves[square_].set_bit((square as i32 - 17).try_into().unwrap());
            }
            if ROW[square_] > 0 && COLUMN[square_] > 1 {
                bit_knight_moves[square_].set_bit((square as i32 - 10).try_into().unwrap());
            }
        }

        bit_knight_moves
    }

    pub fn get_king_moves() -> [BitBoard; NUM_SQUARES] {
        let mut bit_king_moves = [BitBoard(0); NUM_SQUARES];

        for square in Square::iter() {
            let square_ = square as usize;

            if COLUMN[square_] > 0 {
                bit_king_moves[square_].set_bit((square as i32 - 1).try_into().unwrap());
            }
            if COLUMN[square_] < 7 {
                bit_king_moves[square_].set_bit((square as i32 + 1).try_into().unwrap());
            }
            if ROW[square_] > 0 {
                bit_king_moves[square_].set_bit((square as i32 - 8).try_into().unwrap());
            }
            if ROW[square_] < 7 {
                bit_king_moves[square_].set_bit((square as i32 + 8).try_into().unwrap());
            }
            if COLUMN[square_] < 7 && ROW[square_] < 7 {
                bit_king_moves[square_].set_bit((square as i32 + 9).try_into().unwrap());
            }
            if COLUMN[square_] > 0 && ROW[square_] < 7 {
                bit_king_moves[square_].set_bit((square as i32 + 7).try_into().unwrap());
            }
            if COLUMN[square_] > 0 && ROW[square_] > 0 {
                bit_king_moves[square_].set_bit((square as i32 - 9).try_into().unwrap());
            }
            if COLUMN[square_] < 7 && ROW[square_] > 0 {
                bit_king_moves[square_].set_bit((square as i32 - 7).try_into().unwrap());
            }
        }

        bit_king_moves
    }

    pub fn get_queenside_and_kingside_masks() -> (BitBoard, BitBoard) {
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

    pub fn get_queen_rook_bishop_moves() -> (
        [BitBoard; NUM_SQUARES],
        [BitBoard; NUM_SQUARES],
        [BitBoard; NUM_SQUARES],
    ) {
        let mut bit_queen_moves = [BitBoard(0); NUM_SQUARES];
        let mut bit_rook_moves = [BitBoard(0); NUM_SQUARES];
        let mut bit_bishop_moves = [BitBoard(0); NUM_SQUARES];

        for square in Square::iter() {
            let square_ = square as usize;

            for square_2 in Square::iter() {
                let square_2_ = square_2 as usize;

                if square != square_2 {
                    if NORTH_WEST_DIAGONAL[square_] == NORTH_WEST_DIAGONAL[square_2_]
                        || NORTH_EAST_DIAGONAL[square_] == NORTH_EAST_DIAGONAL[square_2_]
                    {
                        bit_queen_moves[square_].set_bit(square_2);
                        bit_bishop_moves[square_].set_bit(square_2);
                    }

                    if ROW[square_] == ROW[square_2_] || COLUMN[square_] == COLUMN[square_2_] {
                        bit_queen_moves[square_].set_bit(square_2);
                        bit_rook_moves[square_].set_bit(square_2);
                    }
                }
            }
        }

        (bit_queen_moves, bit_rook_moves, bit_bishop_moves)
    }

    pub fn get_score_tables() -> (
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
}
