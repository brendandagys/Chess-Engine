use crate::{
    constants::{
        BISHOP_CAPTURE_SCORE, BISHOP_SCORE, CAPTURE_SCORE, CASTLE_MASK, COLUMN,
        DEFAULT_MAX_QUIESCENCE_DEPTH, FLIPPED_BOARD_SQUARE, GAME_STACK, HASH_SCORE, INFINITY_SCORE,
        ISOLATED_PAWN_SCORE, KING_CAPTURE_SCORE, KING_ENDGAME_SCORE, KING_SCORE, KINGSIDE_DEFENSE,
        KNIGHT_CAPTURE_SCORE, KNIGHT_SCORE, MATE_SCORE, MAX_HISTORY_SCORE, MAX_PLY, MOVE_STACK,
        NORTH_EAST_DIAGONAL, NORTH_WEST_DIAGONAL, NUM_PIECE_TYPES, NUM_SIDES, NUM_SQUARES,
        PASSED_SCORE, PAWN_CAPTURE_SCORE, PAWN_SCORE, QUEEN_CAPTURE_SCORE, QUEEN_SCORE,
        QUEENSIDE_DEFENSE, REVERSE_SQUARE, ROOK_CAPTURE_SCORE, ROOK_SCORE, ROW,
    },
    time::TimeManager,
    types::{BitBoard, Board, Game, GameResult, Move, Piece, Side, Square},
};

pub struct Position {
    // DYNAMIC
    pub move_list: [Option<Move>; MOVE_STACK],
    pub first_move: [isize; MAX_PLY], // First move location for each ply in the move list (ply 1: 0, ply 2: first_move[1])
    pub game_list: [Option<Game>; GAME_STACK], // Indexes by `ply_from_start_of_game`
    pub fifty: u8,                    // Ply since last capture or pawn move (0-100) [50-move rule]
    pub nodes: usize, // Total nodes (position in search tree) searched since start of turn
    pub qnodes: usize, // Quiescence nodes searched
    pub seldepth: usize, // Maximum selective search depth reached (including quiescence)
    pub hash_hits: usize, // Number of transposition table hits
    pub hash_stores: usize, // Number of positions stored in hash table
    pub beta_cutoffs: usize, // Number of beta cutoffs (fail-highs)
    pub ply: usize, // How many half-moves deep in current search tree; resets each search ("move" = both players take a turn)
    pub ply_from_start_of_game: usize, // Total half-moves from start of game (take-backs, fifty-move rule)
    pub board: Board,
    pub pawn_engine_score: [usize; NUM_SIDES],
    pub piece_engine_score: [usize; NUM_SIDES],
    pub material_score: [usize; NUM_SIDES],
    pub castle: u8,                     // Castle permissions
    pub best_move_from: Option<Square>, // Found from the search/hash
    pub best_move_to: Option<Square>,   // Found from the search/hash
    pub hash_from: Option<Square>,
    pub hash_to: Option<Square>,
    pub time_manager: TimeManager,
    // STATIC
    pub side: Side,
    square_score: [[[i32; NUM_SQUARES]; NUM_PIECE_TYPES]; NUM_SIDES],
    king_endgame_score: [[i32; NUM_SQUARES]; NUM_SIDES],
    passed_pawns_score: [[i32; NUM_SQUARES]; NUM_SIDES], // Score for 7th rank is built into `square_score`
    bit_between: [[BitBoard; NUM_SQUARES]; NUM_SQUARES], // &'ed with `bit_all`. 0-result means nothing blocking the line
    bit_after: [[BitBoard; NUM_SQUARES]; NUM_SQUARES], // Square and those after it in vector are 0
    bit_pawn_left_captures: [[BitBoard; NUM_SQUARES]; NUM_SIDES],
    bit_pawn_right_captures: [[BitBoard; NUM_SQUARES]; NUM_SIDES],
    bit_pawn_defends: [[BitBoard; NUM_SQUARES]; NUM_SIDES],
    bit_knight_moves: [BitBoard; NUM_SQUARES],
    bit_bishop_moves: [BitBoard; NUM_SQUARES],
    bit_rook_moves: [BitBoard; NUM_SQUARES],
    bit_queen_moves: [BitBoard; NUM_SQUARES],
    bit_king_moves: [BitBoard; NUM_SQUARES],
    mask_passed: [[BitBoard; NUM_SQUARES]; NUM_SIDES],
    mask_path: [[BitBoard; NUM_SQUARES]; NUM_SIDES],
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

impl Default for Position {
    fn default() -> Self {
        Self::new(TimeManager::default())
    }
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

    fn add_move(&mut self, from: Square, to: Square, history_score: isize, move_count: &mut isize) {
        let move_ = Move {
            from,
            to,
            promote: None,
            score: history_score,
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

    fn add_pawn_promotion_moves(
        &mut self,
        from: Square,
        to: Square,
        history_score: isize,
        move_count: &mut isize,
    ) {
        // Add moves for all four promotion pieces: Queen, Rook, Bishop, Knight
        for promote_piece in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
            let move_ = Move {
                from,
                to,
                promote: Some(promote_piece),
                score: history_score,
            };
            self.move_list[*move_count as usize] = Some(move_);
            *move_count += 1;
        }
    }

    fn add_pawn_promotion_captures(
        &mut self,
        from: Square,
        to: Square,
        base_score: isize,
        move_count: &mut isize,
    ) {
        // Add capture moves for all four promotion pieces: Queen, Rook, Bishop, Knight
        for promote_piece in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
            let move_ = Move {
                from,
                to,
                promote: Some(promote_piece),
                score: base_score + CAPTURE_SCORE as isize,
            };
            self.move_list[*move_count as usize] = Some(move_);
            *move_count += 1;
        }
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

    /// Material scores are set by `set_material()`, called after FEN loading.
    /// The board pieces are already correctly set up from FEN loading or `Position::new()`,
    /// so we don't need to re-add them here (doing so would corrupt the hash by double-toggling).
    pub fn set_material_scores(&mut self) {
        self.piece_engine_score = [0; NUM_SIDES];
        self.pawn_engine_score = [0; NUM_SIDES];
        self.material_score = [0; NUM_SIDES];

        // Recalculate material scores from current board state
        for square in Square::iter() {
            let piece = self.board.value[square as usize];

            if piece != Piece::Empty {
                let side_idx = if self.board.bit_units[Side::White as usize].is_bit_set(square) {
                    Side::White as usize
                } else {
                    Side::Black as usize
                };

                if piece == Piece::Pawn {
                    self.pawn_engine_score[side_idx] += piece.value() as usize;
                } else {
                    self.piece_engine_score[side_idx] += piece.value() as usize;
                }

                self.material_score[side_idx] += piece.traditional_value() as usize;
            }
        }
    }

    /// Displays the chess board with colored squares and pieces
    ///
    /// # Arguments
    /// * `flip` - If true, displays from black's perspective (a1 at top-right)
    ///            If false, displays from white's perspective (a1 at bottom-left)
    pub fn display_board(&self, flip: bool) {
        // Board color pattern for display (0 = dark, 1 = light)
        const BOARD_COLOR: [u8; 64] = [
            1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0,
            1, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1,
            0, 1, 0, 1, 0, 1,
        ];

        // Character representation of pieces
        const PIECE_CHAR: [char; 7] = ['P', 'N', 'B', 'R', 'Q', 'K', ' '];

        // Terminal color codes for better display
        let reset = "\x1b[0m";
        let dark_square = "\x1b[48;5;94m"; // Dark brown background
        let light_square = "\x1b[48;5;223m"; // Light brown background
        let white_piece_color = "\x1b[97m"; // White text for white pieces
        let black_piece_color = "\x1b[96m"; // Cyan text for black pieces

        println!();

        for rank in 0..8 {
            let display_rank = if !flip { 7 - rank } else { rank };

            // Print rank number at the start of each row
            print!("{} ", display_rank + 1);

            for file in 0..8 {
                let display_file = if !flip { file } else { 7 - file };
                let i = (display_rank * 8 + display_file) as usize;

                let piece = self.board.value[i];
                let is_white_piece = self.board.bit_units[Side::White as usize]
                    .is_bit_set(Square::try_from(i as u8).unwrap());

                // Set background color based on square color
                let bg_color = if BOARD_COLOR[i] == 0 {
                    light_square
                } else {
                    dark_square
                };

                // Set text color based on piece color
                let text_color = if is_white_piece {
                    white_piece_color
                } else {
                    black_piece_color
                };

                print!("{}{}", bg_color, text_color);

                match piece {
                    Piece::Empty => print!("   "),
                    _ => {
                        let piece_char = PIECE_CHAR[piece as usize];
                        if is_white_piece {
                            print!(" {} ", piece_char);
                        } else {
                            print!(" {} ", piece_char.to_lowercase());
                        }
                    }
                }

                print!("{}", reset);
            }

            // Line break at the end of each rank
            println!();
        }

        if !flip {
            println!("   a  b  c  d  e  f  g  h");
        } else {
            println!("   h  g  f  e  d  c  b  a");
        }

        // Display material scores
        let white_engine_score = self.pawn_engine_score[Side::White as usize]
            + self.piece_engine_score[Side::White as usize];
        let black_engine_score = self.pawn_engine_score[Side::Black as usize]
            + self.piece_engine_score[Side::Black as usize];

        let diff = self.material_score[Side::White as usize]
            .abs_diff(self.material_score[Side::Black as usize]);

        let (white_advantage, black_advantage) = match self.material_score[Side::White as usize]
            .cmp(&self.material_score[Side::Black as usize])
        {
            std::cmp::Ordering::Greater => (diff, 0),
            std::cmp::Ordering::Less => (0, diff),
            std::cmp::Ordering::Equal => (0, 0),
        };

        println!(
            "\nMaterial: W - {}{} | B - {}{}",
            white_engine_score,
            if white_advantage > 0 {
                format!(" (+{})", white_advantage)
            } else {
                "".to_string()
            },
            black_engine_score,
            if black_advantage > 0 {
                format!(" (+{})", black_advantage)
            } else {
                "".to_string()
            }
        );
    }

    pub fn display_move(from: Square, to: Square) {
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
            return Some(Square::try_from(b1.next_bit()).ok()?);
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
                    return Some(Square::try_from(attacking_piece).ok()?);
                }
            }
        }

        let b1 = BitBoard(
            self.bit_king_moves[square as usize].0
                & self.board.bit_pieces[side as usize][Piece::King as usize].0,
        );

        if b1.0 != 0 {
            return Some(Square::try_from(b1.next_bit()).ok()?);
        }

        None
    }

    fn generate_en_passant_moves(&mut self, side: Side, move_count: &mut isize) {
        if self.ply_from_start_of_game == 0 {
            return;
        }

        let last_game_entry = self.game_list[self.ply_from_start_of_game];

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
                    let our_pawn_square = (last_square_opponent_moved_to as i32 - 1) as usize;
                    self.add_capture(
                        (last_square_opponent_moved_to as i32 - 1)
                            .try_into()
                            .expect("Failed to convert square index to Square"),
                        self.pawn_right_index[side as usize][our_pawn_square]
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
                    let our_pawn_square = (last_square_opponent_moved_to as i32 + 1) as usize;
                    self.add_capture(
                        (last_square_opponent_moved_to as i32 + 1)
                            .try_into()
                            .expect("Failed to convert square index to Square"),
                        self.pawn_left_index[side as usize][our_pawn_square]
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
                    self.add_move(Square::E1, Square::G1, 0, move_count);
                }
                // Queenside
                if self.castle & 2 != 0
                    && (self.bit_between[Square::A1 as usize][Square::E1 as usize].0
                        & self.board.bit_all.0)
                        == 0
                {
                    self.add_move(Square::E1, Square::C1, 0, move_count);
                }
            }
            Side::Black => {
                // Kingside
                if self.castle & 4 != 0
                    && (self.bit_between[Square::E8 as usize][Square::H8 as usize].0
                        & self.board.bit_all.0)
                        == 0
                {
                    self.add_move(Square::E8, Square::G8, 0, move_count);
                }
                // Queenside
                if self.castle & 8 != 0
                    && (self.bit_between[Square::E8 as usize][Square::A8 as usize].0
                        & self.board.bit_all.0)
                        == 0
                {
                    self.add_move(Square::E8, Square::C8, 0, move_count);
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

    pub fn generate_moves_and_captures<F>(&mut self, side: Side, get_history_score: F)
    where
        F: Fn(Side, Square, Square) -> isize,
    {
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
            let square_to = victim
                .try_into()
                .expect("Failed to convert victim to Square");

            let base_score =
                PAWN_CAPTURE_SCORE[self.board.value[victim.next_bit() as usize] as usize] as isize;

            // Check if this is a promotion
            if self.ranks[side as usize][square_from as usize] == 6 {
                self.add_pawn_promotion_captures(
                    square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square"),
                    square_to,
                    base_score,
                    &mut move_count,
                );
            } else {
                self.add_capture(
                    square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square"),
                    square_to,
                    base_score,
                    &mut move_count,
                );
            }
        }

        while right_pawn_captures.0 != 0 {
            let square_from = right_pawn_captures.next_bit_mut();
            let victim = self.bit_pawn_right_captures[side as usize][square_from as usize];
            let square_to = victim
                .try_into()
                .expect("Failed to convert victim to Square");

            let base_score =
                PAWN_CAPTURE_SCORE[self.board.value[victim.next_bit() as usize] as usize] as isize;

            // Check if this is a promotion
            if self.ranks[side as usize][square_from as usize] == 6 {
                self.add_pawn_promotion_captures(
                    square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square"),
                    square_to,
                    base_score,
                    &mut move_count,
                );
            } else {
                self.add_capture(
                    square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square"),
                    square_to,
                    base_score,
                    &mut move_count,
                );
            }
        }

        while unblocked_pawns.0 != 0 {
            let square_from = unblocked_pawns.next_bit_mut();
            let to = self.pawn_plus_index[side as usize][square_from as usize];

            // Only add the move if the destination square is valid
            if to >= 0 && to <= 63 {
                let square_to = to
                    .try_into()
                    .expect("Failed to convert pawn plus index to Square");

                // Check if this is a promotion
                if self.ranks[side as usize][square_from as usize] == 6 {
                    let square_from = square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square");

                    self.add_pawn_promotion_moves(
                        square_from,
                        square_to,
                        get_history_score(side, square_from, square_to),
                        &mut move_count,
                    );
                } else {
                    let square_from: Square = square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square");

                    self.add_move(
                        square_from,
                        square_to,
                        get_history_score(side, square_from, square_to),
                        &mut move_count,
                    );

                    // Check double jump validity
                    if self.ranks[side as usize][square_from as usize] == 1
                        && self.board.value
                            [self.pawn_double_index[side as usize][square_from as usize] as usize]
                            == Piece::Empty
                    {
                        let square_double_jump_to: Square = self.pawn_double_index[side as usize]
                            [square_from as usize]
                            .try_into()
                            .expect("Failed to convert pawn double index to Square");

                        self.add_move(
                            square_from,
                            square_double_jump_to,
                            get_history_score(side, square_from, square_double_jump_to),
                            &mut move_count,
                        );
                    }
                }
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

                // TODO: remove
                let captured_piece = self.board.value[square_to as usize];
                if captured_piece != Piece::Empty
                    && (captured_piece as usize) < KNIGHT_CAPTURE_SCORE.len()
                {
                    self.add_capture(
                        square_from
                            .try_into()
                            .expect("Failed to convert square_from to Square"),
                        square_to
                            .try_into()
                            .expect("Failed to convert square_to to Square"),
                        KNIGHT_CAPTURE_SCORE[self.board.value[square_to as usize] as usize]
                            as isize,
                        &mut move_count,
                    );
                }
            }

            let mut knight_moves =
                BitBoard(self.bit_knight_moves[square_from as usize].0 & !self.board.bit_all.0);

            while knight_moves.0 != 0 {
                let square_from: Square = square_from
                    .try_into()
                    .expect("Failed to convert square_from to Square");

                let square_to: Square = knight_moves
                    .next_bit_mut()
                    .try_into()
                    .expect("Failed to convert square_to to Square");

                let history_score = get_history_score(side, square_from, square_to);
                self.add_move(square_from, square_to, history_score, &mut move_count);
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
                let square_from = pieces.next_bit_mut();
                let mut possible_moves = BitBoard(bit_moves[square_from as usize].0);

                // Remove squares blocked by friendly units and squares after them
                let mut moves_to_self_occupied_squares =
                    BitBoard(possible_moves.0 & self.board.bit_units[side as usize].0);

                while moves_to_self_occupied_squares.0 != 0 {
                    let square_to = moves_to_self_occupied_squares.next_bit_mut();

                    moves_to_self_occupied_squares.0 &=
                        self.bit_after[square_from as usize][square_to as usize].0;

                    possible_moves.0 &= self.bit_after[square_from as usize][square_to as usize].0;
                }

                let mut possible_captures =
                    BitBoard(possible_moves.0 & self.board.bit_units[side.opponent() as usize].0);

                while possible_captures.0 != 0 {
                    let square_to = possible_captures.next_bit_mut();

                    if (self.bit_between[square_from as usize][square_to as usize].0
                        & self.board.bit_all.0)
                        == 0
                    {
                        // TODO: remove
                        let captured_piece = self.board.value[square_to as usize];
                        // Skip if empty (shouldn't happen, but safeguard against board corruption)
                        if captured_piece != Piece::Empty
                            && (captured_piece as usize) < capture_score.len()
                        {
                            self.add_capture(
                                square_from
                                    .try_into()
                                    .expect("Failed to convert square_from to Square"),
                                square_to
                                    .try_into()
                                    .expect("Failed to convert square_to to Square"),
                                capture_score[captured_piece as usize] as isize,
                                &mut move_count,
                            );
                        }
                    }

                    possible_captures.0 &=
                        self.bit_after[square_from as usize][square_to as usize].0;

                    possible_moves.0 &= self.bit_after[square_from as usize][square_to as usize].0;
                }

                while possible_moves.0 != 0 {
                    let square_to = possible_moves.next_bit_mut();

                    let square_from: Square = square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square");

                    let square_to: Square = square_to
                        .try_into()
                        .expect("Failed to convert square_to to Square");

                    self.add_move(
                        square_from,
                        square_to,
                        get_history_score(side, square_from, square_to),
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
            let square_from: Square = king_square
                .try_into()
                .expect("Failed to convert king_square to Square");

            let square_to: Square = king_moves
                .next_bit_mut()
                .try_into()
                .expect("Failed to convert square_to to Square");

            self.add_move(
                square_from,
                square_to,
                get_history_score(side, square_from, square_to),
                &mut move_count,
            );
        }

        self.first_move[self.ply + 1] = move_count;
    }

    pub fn generate_captures(&mut self, side: Side) {
        let mut move_count = self.first_move[self.ply];

        self.generate_en_passant_moves(side, &mut move_count);

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
            let square_to = victim
                .try_into()
                .expect("Failed to convert victim to Square");

            let base_score =
                PAWN_CAPTURE_SCORE[self.board.value[victim.next_bit() as usize] as usize] as isize;

            // Check if this is a promotion
            if self.ranks[side as usize][square_from as usize] == 6 {
                self.add_pawn_promotion_captures(
                    square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square"),
                    square_to,
                    base_score,
                    &mut move_count,
                );
            } else {
                self.add_capture(
                    square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square"),
                    square_to,
                    base_score,
                    &mut move_count,
                );
            }
        }

        while right_pawn_captures.0 != 0 {
            let square_from = right_pawn_captures.next_bit_mut();
            let victim = self.bit_pawn_right_captures[side as usize][square_from as usize];
            let square_to = victim
                .try_into()
                .expect("Failed to convert victim to Square");

            let base_score =
                PAWN_CAPTURE_SCORE[self.board.value[victim.next_bit() as usize] as usize] as isize;

            // Check if this is a promotion
            if self.ranks[side as usize][square_from as usize] == 6 {
                self.add_pawn_promotion_captures(
                    square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square"),
                    square_to,
                    base_score,
                    &mut move_count,
                );
            } else {
                self.add_capture(
                    square_from
                        .try_into()
                        .expect("Failed to convert square_from to Square"),
                    square_to,
                    base_score,
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
                        // TODO: remove
                        let captured_piece = self.board.value[square_to as usize];
                        // Skip if empty (shouldn't happen, but safeguard against board corruption)
                        if captured_piece != Piece::Empty
                            && (captured_piece as usize) < capture_score.len()
                        {
                            self.add_capture(
                                attacking_square
                                    .try_into()
                                    .expect("Failed to convert square_from to Square"),
                                square_to
                                    .try_into()
                                    .expect("Failed to convert square_to to Square"),
                                capture_score[captured_piece as usize] as isize,
                                &mut move_count,
                            );
                        }
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

    /// Returns the number of times the current position has occurred **previously** in the game
    pub fn repetitions(&self) -> usize {
        // game_list[i] stores the hash of the position BEFORE the move that led to ply i
        // So game_list[i].hash is the position at ply i-1
        // When we're at ply N, we want to check positions at ply N-2, N-4, etc. (same side to move)
        // Those hashes are stored in game_list[N-1], game_list[N-3], etc.

        if self.ply_from_start_of_game < 2 {
            return 0;
        }

        let current_hash = self.board.hash.current_key;
        let min_ply = self
            .ply_from_start_of_game
            .saturating_sub(self.fifty as usize);

        // Iterate backwards by 2 (checking same side to move)
        // Start at N-1 (representing position at ply N-2), stop when reaching min_ply
        (0..=(self.ply_from_start_of_game - 1) / 2)
            .map(|k| self.ply_from_start_of_game - 1 - (2 * k))
            .take_while(|&i| i > 0 && i - 1 >= min_ply)
            .filter_map(|i| self.game_list[i])
            .filter(|game| game.hash == current_hash)
            .count()
    }

    /// Checks if the position has insufficient material to checkmate
    ///
    /// Returns true if:
    /// - King vs King
    /// - King + Bishop vs King
    /// - King + Knight vs King
    /// - King + Bishop vs King + Bishop (same color bishops)
    ///
    /// Note: K+B vs K+N and K+N vs K+N are NOT insufficient material
    fn has_insufficient_material(&self) -> bool {
        // If there are any pawns, there's sufficient material (pawn can promote)
        if self.pawn_engine_score[Side::White as usize] != 0
            || self.pawn_engine_score[Side::Black as usize] != 0
        {
            return false;
        }

        // Count pieces for each side (excluding kings)
        let white_queens = self.board.bit_pieces[Side::White as usize][Piece::Queen as usize]
            .0
            .count_ones();
        let white_rooks = self.board.bit_pieces[Side::White as usize][Piece::Rook as usize]
            .0
            .count_ones();
        let white_bishops = self.board.bit_pieces[Side::White as usize][Piece::Bishop as usize]
            .0
            .count_ones();
        let white_knights = self.board.bit_pieces[Side::White as usize][Piece::Knight as usize]
            .0
            .count_ones();

        let black_queens = self.board.bit_pieces[Side::Black as usize][Piece::Queen as usize]
            .0
            .count_ones();
        let black_rooks = self.board.bit_pieces[Side::Black as usize][Piece::Rook as usize]
            .0
            .count_ones();
        let black_bishops = self.board.bit_pieces[Side::Black as usize][Piece::Bishop as usize]
            .0
            .count_ones();
        let black_knights = self.board.bit_pieces[Side::Black as usize][Piece::Knight as usize]
            .0
            .count_ones();

        // If either side has a queen or rook, there's sufficient material
        if white_queens > 0 || white_rooks > 0 || black_queens > 0 || black_rooks > 0 {
            return false;
        }

        // K vs K
        if white_bishops == 0 && white_knights == 0 && black_bishops == 0 && black_knights == 0 {
            return true;
        }

        // K+B vs K or K vs K+B
        if (white_bishops == 1 && white_knights == 0 && black_bishops == 0 && black_knights == 0)
            || (black_bishops == 1
                && black_knights == 0
                && white_bishops == 0
                && white_knights == 0)
        {
            return true;
        }

        // K+N vs K or K vs K+N
        if (white_knights == 1 && white_bishops == 0 && black_bishops == 0 && black_knights == 0)
            || (black_knights == 1
                && black_bishops == 0
                && white_bishops == 0
                && white_knights == 0)
        {
            return true;
        }

        // K+B vs K+B (same color bishops)
        // Need to check if bishops are on same colored squares
        if white_bishops == 1 && white_knights == 0 && black_bishops == 1 && black_knights == 0 {
            let white_bishop_square =
                self.board.bit_pieces[Side::White as usize][Piece::Bishop as usize].next_bit();
            let black_bishop_square =
                self.board.bit_pieces[Side::Black as usize][Piece::Bishop as usize].next_bit();

            // Bishops are on same color if (rank + file) has same parity
            let white_color = (white_bishop_square / 8 + white_bishop_square % 8) % 2;
            let black_color = (black_bishop_square / 8 + black_bishop_square % 8) % 2;

            if white_color == black_color {
                return true;
            }
        }

        false
    }

    /// Checks the current game state and returns the result
    pub fn check_game_result(&mut self) -> GameResult {
        if self.repetitions() >= 2 {
            return GameResult::DrawByRepetition;
        }

        if self.fifty >= 100 {
            return GameResult::DrawByFiftyMoveRule;
        }

        if self.has_insufficient_material() {
            return GameResult::DrawByInsufficientMaterial;
        }

        self.generate_moves_and_captures(self.side, |_, _, _| 0);

        // Check for legal moves for the player-now-to-move
        let mut has_legal_moves = false;
        for i in self.first_move[self.ply]..self.first_move[1 + self.ply] as isize {
            if let Some(mv) = self.move_list[i as usize] {
                if self.make_move(mv.from, mv.to, mv.promote) {
                    self.take_back_move();
                    has_legal_moves = true;
                    break;
                }
            }
        }

        if !has_legal_moves {
            // Check if king is in check
            let king_square =
                self.board.bit_pieces[self.side as usize][Piece::King as usize].next_bit();

            if self.is_square_attacked_by_side(
                self.side.opponent(),
                Square::try_from(king_square).unwrap(),
            ) {
                return GameResult::Checkmate(self.side.opponent());
            } else {
                return GameResult::Stalemate;
            }
        }

        GameResult::InProgress
    }

    /// Get the en passant file (0-7 for files A-H) from the last move, if available
    fn get_en_passant_file(&self) -> Option<u8> {
        if self.ply_from_start_of_game == 0 {
            return None;
        }

        if let Some(last_game) = self.game_list[self.ply_from_start_of_game] {
            let from = last_game.from;
            let to = last_game.to;

            // Check if a pawn just made a double-push
            if self.board.value[to as usize] == Piece::Pawn && (from as i32 - to as i32).abs() == 16
            {
                return Some(COLUMN[to as usize]);
            }
        }

        None
    }

    /// TODO: Should this return something else (e.g., Result type)?
    /// Make a move with optional promotion piece and return success state.
    /// If unsuccessful, the move will be undone.
    pub fn make_move(&mut self, from: Square, to: Square, promote: Option<Piece>) -> bool {
        // Check for castling
        if (to as i32 - from as i32).abs() == 2 && self.board.value[from as usize] == Piece::King {
            // Cannot castle out of check
            if self.is_square_attacked_by_side(self.side.opponent(), from) {
                return false;
            }

            if to == Square::G1 {
                // Cannot castle through check or into check
                if self.is_square_attacked_by_side(self.side.opponent(), Square::F1)
                    || self.is_square_attacked_by_side(self.side.opponent(), Square::G1)
                {
                    return false;
                }

                self.board
                    .update_piece(self.side, Piece::Rook, Square::H1, Square::F1);
            } else if to == Square::C1 {
                // Cannot castle through check or into check
                if self.is_square_attacked_by_side(self.side.opponent(), Square::D1)
                    || self.is_square_attacked_by_side(self.side.opponent(), Square::C1)
                {
                    return false;
                }

                self.board
                    .update_piece(self.side, Piece::Rook, Square::A1, Square::D1);
            } else if to == Square::G8 {
                // Cannot castle through check or into check
                if self.is_square_attacked_by_side(self.side.opponent(), Square::F8)
                    || self.is_square_attacked_by_side(self.side.opponent(), Square::G8)
                {
                    return false;
                }

                self.board
                    .update_piece(self.side, Piece::Rook, Square::H8, Square::F8);
            } else if to == Square::C8 {
                // Cannot castle through check or into check
                if self.is_square_attacked_by_side(self.side.opponent(), Square::D8)
                    || self.is_square_attacked_by_side(self.side.opponent(), Square::C8)
                {
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

        // Store old en passant file for hash update
        let old_en_passant_file = self.get_en_passant_file();
        game.en_passant_file = old_en_passant_file;

        // Update the castle permissions
        let old_castle = self.castle;
        self.castle &= CASTLE_MASK[from as usize] & CASTLE_MASK[to as usize];

        if old_castle != self.castle {
            self.board
                .hash
                .update_castle_rights(old_castle, self.castle);
        }

        self.ply += 1;
        self.ply_from_start_of_game += 1;
        self.fifty += 1;

        if self.board.value[from as usize] == Piece::Pawn {
            self.fifty = 0;

            // Handle en passant (diagonal pawn move to empty square, but NOT on promotion rank)
            if self.board.value[to as usize] == Piece::Empty
                && COLUMN[from as usize] != COLUMN[to as usize]
                && ![0, 7].contains(&ROW[to as usize])
            // Not on promotion rank
            {
                let en_passant_target = to as i32 + REVERSE_SQUARE[self.side as usize];
                self.board.remove_piece(
                    self.side.opponent(),
                    Piece::Pawn,
                    en_passant_target
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
            let promotion_piece = promote.unwrap_or(Piece::Queen);
            self.board.remove_piece(self.side, Piece::Pawn, from);
            self.board.add_piece(self.side, promotion_piece, to);

            game.promote = Some(promotion_piece);
        } else {
            self.board
                .update_piece(self.side, self.board.value[from as usize], from, to);

            game.promote = None;
        }

        self.set_material_scores();

        let original_side = self.side;

        self.side = self.side.opponent();

        self.board.hash.toggle_side_to_move();

        self.game_list[self.ply_from_start_of_game] = Some(game);

        // Determine new en passant file after this move
        let new_en_passant_file = if self.board.value[to as usize] == Piece::Pawn
            && (from as i32 - to as i32).abs() == 16
        {
            Some(COLUMN[to as usize])
        } else {
            None
        };

        self.board
            .hash
            .update_en_passant(old_en_passant_file, new_en_passant_file);

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

    pub fn take_back_move(&mut self) {
        let game = self.game_list[self.ply_from_start_of_game].expect("No game to take back");

        let current_en_passant_file = self.get_en_passant_file();

        self.side = self.side.opponent();

        self.board.hash.toggle_side_to_move();

        // 2 undo calls from CLI may cause underflow since ply is 1 after it moves piece
        self.ply = self.ply.saturating_sub(1);

        self.ply_from_start_of_game -= 1;

        let from = game.from;
        let to = game.to;

        let old_castle = self.castle;
        self.castle = game.castle;

        if old_castle != self.castle {
            self.board
                .hash
                .update_castle_rights(old_castle, self.castle);
        }

        self.fifty = game.fifty;

        self.board
            .hash
            .update_en_passant(current_en_passant_file, game.en_passant_file);

        // En passant
        if self.board.value[to as usize] == Piece::Pawn
            && game.capture == Piece::Empty
            && COLUMN[from as usize] != COLUMN[to as usize]
        {
            let en_passant_target = to as i32 + REVERSE_SQUARE[self.side as usize];
            // Validate the target square is in range
            if en_passant_target >= 0 && en_passant_target < 64 {
                self.board.add_piece(
                    self.side.opponent(),
                    Piece::Pawn,
                    Square::try_from(en_passant_target as u8).unwrap(),
                );
            }
        }

        // Promotion
        if let Some(promotion_piece) = game.promote {
            self.board.add_piece(self.side, Piece::Pawn, from);
            self.board.remove_piece(self.side, promotion_piece, to);
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

        self.set_material_scores();
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
    pub fn evaluate(&self) -> i32 {
        let mut score = [0, 0];

        let mut queenside_pawns = [0, 0];
        let mut kingside_pawns = [0, 0];

        for side in Side::iter() {
            // Add material score
            score[side as usize] += self.material_score[side as usize] as i32;

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
                    score[side as usize] += kingside_pawns[side as usize]
                } else if self.board.bit_pieces[side as usize][Piece::King as usize].0
                    & self.mask_queenside.0
                    != 0
                {
                    score[side as usize] += queenside_pawns[side as usize]
                }
            }
        }

        // Return evaluation from side-to-move's perspective
        // Negamax requires positive scores to mean the side-to-move is winning
        let total = score[Side::White as usize] - score[Side::Black as usize];

        match self.side {
            Side::White => total,
            Side::Black => -total,
        }
    }

    pub fn parse_square(input: &str) -> Option<usize> {
        if input.len() != 2 {
            return None;
        }

        let chars: Vec<char> = input.chars().collect();

        if chars[0] < 'a' || chars[0] > 'h' || chars[1] < '1' || chars[1] > '8' {
            return None;
        }

        let file = (chars[0] as u8 - b'a') as usize;
        let rank = (chars[1] as u8 - b'1') as usize;

        Some(rank * 8 + file)
    }

    #[allow(dead_code)]
    fn set_hash_move(&mut self) {
        for i in self.first_move[self.ply]..self.first_move[self.ply + 1] {
            if let Some(ref mut move_) = self.move_list[i as usize] {
                if let (Some(hash_from), Some(hash_to)) = (self.hash_from, self.hash_to) {
                    if move_.from == hash_from && move_.to == hash_to {
                        move_.score = HASH_SCORE as isize;
                        return;
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    fn display_principal_variation(&mut self, depth: u16) {
        self.best_move_from = self.hash_from;
        self.best_move_to = self.hash_to;

        for _ in 0..depth {
            if let Some(entry) = self.board.hash.probe() {
                if let Some(move_) = entry.best_move {
                    self.hash_from = Some(move_.from);
                    self.hash_to = Some(move_.to);
                } else {
                    break;
                }
            } else {
                break;
            }

            print!(" ");
            Position::display_move(self.hash_from.unwrap(), self.hash_to.unwrap());
            // NOTE: Hash table doesn't currently store promotion piece, so we use None which defaults to Queen
            if !self.make_move(self.hash_from.unwrap(), self.hash_to.unwrap(), None) {
                // Move failed (shouldn't happen with hash moves, but be safe)
                break;
            }
        }

        while self.ply > 0 {
            self.take_back_move();
        }
    }

    /// Quiescence search extends the regular search by only examining
    /// capturing moves until the position becomes "quiet" (no more captures).
    /// This avoids the horizon effect where the engine stops searching
    /// in the middle of a tactical sequence. This function is recursive.
    ///
    /// # Arguments
    /// * `alpha` - The best score the maximizing player can guarantee
    /// * `beta` - The best score the minimizing player can guarantee
    ///
    /// # Returns
    /// The best score achievable from this position with optimal play
    /// from both sides, bounded by the alpha-beta window.
    ///
    /// # Algorithm
    /// 1. Evaluate the current position (stand pat)
    /// 2. Check for beta cutoff (position too good)
    /// 3. Update alpha if stand pat improves it
    /// 4. Generate and search all capture moves recursively until the position is quiet (base case)
    /// 5. Apply alpha-beta pruning to reduce search space
    fn quiescence_search(&mut self, mut alpha: i32, beta: i32, depth: u16) -> i32 {
        self.nodes += 1;
        self.qnodes += 1;

        // Track selective depth
        if self.ply > self.seldepth {
            self.seldepth = self.ply;
        }

        if depth == 0 {
            return self.evaluate();
        }

        if self.nodes & 1023 == 0 {
            self.check_if_time_is_exhausted();
        }

        let mut best_score = self.evaluate();

        if best_score >= beta {
            return best_score;
        }

        if best_score > alpha {
            alpha = best_score;
        }

        self.generate_captures(self.side);
        let move_list_start = self.first_move[self.ply] as usize;
        let move_list_end = self.first_move[self.ply + 1] as usize;

        // Search all captures
        for move_index in move_list_start..move_list_end {
            self.sort(move_index as isize);

            let current_move = self.move_list[move_index].unwrap();

            // Try to make the move
            if !self.make_move(current_move.from, current_move.to, current_move.promote) {
                // Move is illegal (leaves king in check)
                continue;
            }

            let score = -self.quiescence_search(-beta, -alpha, depth - 1);

            self.take_back_move();

            // Fail-high cutoff
            if score >= beta {
                return score;
            }

            if score > best_score {
                best_score = score;
            }

            if score > alpha {
                alpha = score;
            }
        }

        best_score
    }

    fn check_if_time_is_exhausted(&mut self) {
        if self.time_manager.is_hard_limit_reached() {
            self.time_manager.stopped = true;
            // TODO: Remove panic
            panic!("TimeExhausted"); // This is like longjmp - jumps out immediately
        }
    }

    /// Incrementally sort the move list by selecting the best move from the
    /// unsorted portion and swapping it to the front, utilizing selection sort.
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
    /// Positions are identical if the key is the same.
    fn search_backward_for_identical_position(&self) -> bool {
        let mut cur = self.ply_from_start_of_game.saturating_sub(4);
        let end = self
            .ply_from_start_of_game
            .saturating_sub(self.fifty as usize);

        while cur >= end {
            if let Some(game) = self.game_list[cur] {
                if game.hash == self.board.hash.current_key {
                    return true;
                }
            }

            if cur < 2 {
                break;
            }
            cur -= 2;
        }

        false
    }

    fn update_history_table(
        &self,
        history_table: &mut [[[isize; NUM_SQUARES]; NUM_SQUARES]; NUM_SIDES],
        depth: u16,
        move_: Move,
    ) {
        let depth_sq = (depth as isize) * (depth as isize);
        let current = history_table[self.side as usize][move_.from as usize][move_.to as usize];

        // Use saturating approach to update history table
        let bonus = depth_sq - current * depth_sq.abs() / MAX_HISTORY_SCORE;

        history_table[self.side as usize][move_.from as usize][move_.to as usize] += bonus;
    }

    fn on_beta_cutoff(
        &mut self,
        current_move: Move,
        score: i32,
        depth: u16,
        history_table: &mut [[[isize; NUM_SQUARES]; NUM_SQUARES]; NUM_SIDES],
    ) -> i32 {
        self.beta_cutoffs += 1;

        let is_capture = current_move.to.as_bit() & self.board.bit_all.0 != 0;
        let is_promotion = current_move.promote.is_some();

        // Only update history table for quiet moves
        if !is_capture && !is_promotion {
            self.update_history_table(history_table, depth, current_move);
        }

        // Fail-soft (return the score that caused the cutoff instead of beta)
        return score;
    }

    /// Principal Variation Search with alpha-beta pruning.
    /// PVS is an enhancement of alpha-beta that uses null-window searches for better pruning.
    ///
    /// Alpha is the lower bound (player's best guaranteed score).
    /// Beta is the upper bound (opponent's best guaranteed score).
    pub fn search(
        &mut self,
        mut alpha: i32,
        beta: i32,
        mut depth: u16,
        history_table: &mut [[[isize; NUM_SQUARES]; NUM_SQUARES]; NUM_SIDES],
    ) -> i32 {
        self.nodes += 1;

        // Track selective depth for main search too
        if self.ply > self.seldepth {
            self.seldepth = self.ply;
        }

        // Check for draw by repetition
        if self.ply > 0 && self.search_backward_for_identical_position() {
            return 0;
        }

        // TODO: Should we check for other draw conditions here?

        if depth == 0 {
            return self.quiescence_search(alpha, beta, DEFAULT_MAX_QUIESCENCE_DEPTH);
        }

        if self.nodes & 1023 == 0 {
            self.check_if_time_is_exhausted();
        }

        // TODO: Transposition table integration (see CODEBASE_ANALYSIS.md for details)
        // The hash table infrastructure exists in hash.rs but needs to be integrated here.
        //
        // Step 1: Probe the transposition table ONCE at the start of search (not at ply 0)
        //   let hash_entry = if self.ply > 0 {
        //       self.board.hash.probe()
        //   } else {
        //       None
        //   };
        //
        // Step 2: Check if we can use the cached result for early cutoff
        //   if let Some(entry) = hash_entry {
        //       if entry.depth >= depth as u8 {
        //           // Adjust mate scores for current ply distance
        //           let adjusted_score = if entry.score > MATE_THRESHOLD {
        //               entry.score - self.ply as i32
        //           } else if entry.score < -MATE_THRESHOLD {
        //               entry.score + self.ply as i32
        //           } else {
        //               entry.score
        //           };
        //
        //           match entry.node_type {
        //               NodeType::Exact => {
        //                   self.board.hash.stats.exact_hits += 1;
        //                   return adjusted_score;
        //               }
        //               NodeType::LowerBound => {
        //                   if adjusted_score >= beta {
        //                       self.board.hash.stats.cutoffs += 1;
        //                       return adjusted_score;
        //                   }
        //               }
        //               NodeType::UpperBound => {
        //                   if adjusted_score <= alpha {
        //                       self.board.hash.stats.fail_lows += 1;
        //                       return adjusted_score;
        //                   }
        //               }
        //           }
        //       }
        //   }
        //
        // Step 3: Extract hash move for move ordering
        //   let hash_move: Option<Move> = hash_entry.and_then(|e| e.best_move);
        //
        // Step 4: After move generation, prioritize hash move
        //   if let Some(hm) = hash_move {
        //       for i in move_list_start..move_list_end {
        //           if let Some(mv) = self.move_list[i] {
        //               if mv.from == hm.from && mv.to == hm.to {
        //                   self.move_list[i].as_mut().unwrap().score = HASH_SCORE as isize;
        //                   break;
        //               }
        //           }
        //       }
        //   }
        //
        // Step 5: At end of search, store results in transposition table
        //   // Determine node type
        //   let node_type = if best_score >= beta {
        //       NodeType::LowerBound  // Cut-node (fail-high)
        //   } else if best_score > original_alpha {
        //       NodeType::Exact  // PV-node
        //   } else {
        //       NodeType::UpperBound  // All-node (fail-low)
        //   };
        //
        //   // Adjust mate scores for storage (relative to root, not current ply)
        //   let stored_score = if best_score > MATE_THRESHOLD {
        //       best_score + self.ply as i32
        //   } else if best_score < -MATE_THRESHOLD {
        //       best_score - self.ply as i32
        //   } else {
        //       best_score
        //   };
        //
        //   // Store in hash table
        //   if let Some(mv) = best_move {
        //       self.board.hash.store_move(mv, depth as u8, stored_score, node_type);
        //   } else if self.ply > 0 {
        //       self.board.hash.store_without_move(depth as u8, stored_score, node_type);
        //   }
        //
        // Note: The hash table probe and store methods already exist in hash.rs and work correctly.
        // This integration would provide significant search speedup (2-5x) by avoiding re-computation
        // of previously seen positions.

        // Check if we're currently in check
        let king_square = self.board.bit_pieces[self.side as usize][Piece::King as usize]
            .next_bit()
            .try_into()
            .expect("Failed to convert square to Square");

        let in_check = self.is_square_attacked_by_side(self.side.opponent(), king_square);

        if in_check {
            depth += 1; // Extend search depth if in check
        }

        let mut best_score = -INFINITY_SCORE;
        let mut best_move: Option<Move> = None;

        // Generate all legal moves
        self.generate_moves_and_captures(self.side, |side, from, to| {
            history_table[side as usize][from as usize][to as usize]
        });

        let move_list_start = self.first_move[self.ply] as usize;
        let move_list_end = self.first_move[self.ply + 1] as usize;

        let mut legal_moves_count = 0;
        let mut is_pv_node = true; // Track if this is still a PV node

        for move_index in move_list_start..move_list_end {
            // Pick the best remaining move (selection sort)
            self.sort(move_index as isize);

            let current_move = self.move_list[move_index].unwrap();

            // Try to make the move
            if !self.make_move(current_move.from, current_move.to, current_move.promote) {
                // Move is illegal (leaves king in check)
                continue;
            }

            legal_moves_count += 1;

            let score: i32;

            if is_pv_node {
                // First move: search with full window (standard alpha-beta)
                score = -self.search(-beta, -alpha, depth - 1, history_table);
                is_pv_node = false;
            } else {
                // Non-PV moves: search with null/"zero" window (key optimization of PVS/NegaScout)
                score = -self.search(-(alpha + 1), -alpha, depth - 1, history_table);

                if score > alpha && score < beta {
                    // Re-search with full window to get exact score
                    let re_search_score = -self.search(-beta, -alpha, depth - 1, history_table);
                    self.take_back_move();

                    if re_search_score >= beta {
                        return self.on_beta_cutoff(
                            current_move,
                            re_search_score,
                            depth,
                            history_table,
                        );
                    }

                    if re_search_score > best_score {
                        best_score = re_search_score;
                        best_move = Some(current_move);

                        if best_score > alpha {
                            alpha = best_score;
                        }
                    }

                    continue; // Already took back the move
                }
            }

            self.take_back_move();

            if score >= beta {
                return self.on_beta_cutoff(current_move, score, depth, history_table);
            }

            if score > best_score {
                best_score = score;
                best_move = Some(current_move);

                if best_score > alpha {
                    alpha = best_score;
                }
            }
        }

        // Check for checkmate or stalemate
        if legal_moves_count == 0 {
            if in_check {
                // Checkmate - return negative score, prefer shorter mates
                return -MATE_SCORE + self.ply as i32;
            } else {
                // Stalemate
                return 0;
            }
        }

        // Check for draw by 50-move rule
        if self.fifty >= 100 {
            return 0;
        }

        // Store best move at root for retrieval later
        if self.ply == 0 {
            if let Some(mv) = best_move {
                self.best_move_from = Some(mv.from);
                self.best_move_to = Some(mv.to);
            }
        }

        best_score
    }

    pub fn new(time_manager: TimeManager) -> Self {
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
            bit_pawn_defends,
            pawn_plus_index,
            pawn_double_index,
            not_a_file,
            not_h_file,
        ) = Self::get_pawn_masks();

        let (square_score, king_endgame_score, passed_pawns_score) = Self::get_score_tables();

        let mut first_move = [-1; MAX_PLY];
        first_move[0] = 0;

        let (bit_queen_moves, bit_rook_moves, bit_bishop_moves) =
            Self::get_queen_rook_bishop_moves();

        let mut position = Self {
            // Dynamic
            move_list: [None; MOVE_STACK],
            first_move,
            game_list: [None; GAME_STACK],
            fifty: 0,
            nodes: 0,
            qnodes: 0,
            seldepth: 0,
            hash_hits: 0,
            hash_stores: 0,
            beta_cutoffs: 0,
            ply: 0,
            ply_from_start_of_game: 0,
            board: Board::new(),
            pawn_engine_score: [0; NUM_SIDES],
            piece_engine_score: [0; NUM_SIDES],
            material_score: [0; NUM_SIDES],
            castle: 0b1111, // All castling rights available
            best_move_from: None,
            best_move_to: None,
            hash_from: None,
            hash_to: None,
            time_manager,
            side: Side::White,
            // Static
            square_score,
            king_endgame_score,
            passed_pawns_score,
            bit_between: Self::get_bit_between(),
            bit_after: Self::get_bit_after(),
            bit_pawn_left_captures,
            bit_pawn_right_captures,
            bit_pawn_defends,
            bit_knight_moves: Self::get_knight_moves(),
            bit_bishop_moves,
            bit_rook_moves,
            bit_queen_moves,
            bit_king_moves: Self::get_king_moves(),
            mask_passed,
            mask_path,
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
        };

        position.generate_moves_and_captures(position.side, |_, _, _| 0);
        position.set_material_scores();
        position
    }

    /// Load a position from a FEN (Forsyth-Edwards Notation) string.
    ///
    /// Supports all six FEN fields:
    /// 1. Piece placement (from white's perspective, rank 8 to rank 1)
    /// 2. Active color ("w" or "b")
    /// 3. Castling availability (KQkq or "-")
    /// 4. En passant target square (e.g., "e3" or "-")
    /// 5. Halfmove clock (number of halfmoves since last capture or pawn advance)
    /// 6. Fullmove number (starts at 1, increments after Black's move)
    ///
    /// # Arguments
    /// * `fen` - A string slice containing the FEN notation
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err` with error message on failure
    ///
    /// # Example
    /// ```ignore
    /// let mut position = Position::new();
    /// // Starting position
    /// position.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    /// // Position after 1.e4
    /// position.from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();
    /// ```
    pub fn from_fen(fen: &str) -> Result<Position, Box<dyn std::error::Error>> {
        let mut position = Position::default();
        position.board = Board::empty();

        let parts: Vec<&str> = fen.split(' ').collect();
        if parts.is_empty() {
            return Err("Invalid FEN string".into());
        }

        let board_part = parts[0];
        let ranks: Vec<&str> = board_part.split('/').collect();

        if ranks.len() != 8 {
            return Err("Invalid FEN board: must have 8 ranks".into());
        }

        // Process each rank from 8 to 1 (FEN starts at rank 8)
        for (rank_idx, rank_str) in ranks.iter().enumerate() {
            let rank = 7 - rank_idx; // Convert to 0-based rank (rank 8 → 7, rank 1 → 0)
            let mut file = 0usize;

            for ch in rank_str.chars() {
                if ch.is_ascii_digit() {
                    let empty_squares = ch.to_digit(10).unwrap() as usize;
                    file += empty_squares;
                    continue;
                }

                if file >= 8 {
                    return Err(
                        format!("Invalid FEN: too many squares in rank {}", rank + 1).into(),
                    );
                }

                let square_idx = rank * 8 + file;
                let square = Square::try_from(square_idx as u8)?;

                let (piece, side) = match ch {
                    'K' => (Piece::King, Side::White),
                    'Q' => (Piece::Queen, Side::White),
                    'R' => (Piece::Rook, Side::White),
                    'B' => (Piece::Bishop, Side::White),
                    'N' => (Piece::Knight, Side::White),
                    'P' => (Piece::Pawn, Side::White),
                    'k' => (Piece::King, Side::Black),
                    'q' => (Piece::Queen, Side::Black),
                    'r' => (Piece::Rook, Side::Black),
                    'b' => (Piece::Bishop, Side::Black),
                    'n' => (Piece::Knight, Side::Black),
                    'p' => (Piece::Pawn, Side::Black),
                    _ => return Err(format!("Invalid piece character: {}", ch).into()),
                };

                position.board.add_piece(side, piece, square);
                file += 1;
            }

            if file != 8 {
                return Err(format!(
                    "Invalid FEN: rank {} has {} squares instead of 8",
                    rank + 1,
                    file
                )
                .into());
            }
        }

        // Parse side to move
        if parts.len() > 1 {
            match parts[1] {
                "w" => {
                    position.side = Side::White;
                }
                "b" => {
                    position.side = Side::Black;
                }
                _ => {}
            }
        }

        // Parse castling rights
        if parts.len() > 2 {
            position.castle = 0;
            for ch in parts[2].chars() {
                match ch {
                    'K' => {
                        if position.board.bit_pieces[Side::White as usize][Piece::King as usize]
                            .is_bit_set(Square::E1)
                        {
                            position.castle |= 1;
                        }
                    }
                    'Q' => {
                        if position.board.bit_pieces[Side::White as usize][Piece::King as usize]
                            .is_bit_set(Square::E1)
                        {
                            position.castle |= 2;
                        }
                    }
                    'k' => {
                        if position.board.bit_pieces[Side::Black as usize][Piece::King as usize]
                            .is_bit_set(Square::E8)
                        {
                            position.castle |= 4;
                        }
                    }
                    'q' => {
                        if position.board.bit_pieces[Side::Black as usize][Piece::King as usize]
                            .is_bit_set(Square::E8)
                        {
                            position.castle |= 8;
                        }
                    }
                    '-' => {}
                    _ => {}
                }
            }
        }

        // Parse en passant target square (field 4)
        // Store the info but defer setting game_list until after ply_from_start_of_game is set
        let ep_game_entry: Option<(Square, Square)> = if parts.len() > 3 {
            let ep_square_str = parts[3];

            if ep_square_str != "-" {
                // Validate en passant square format (e.g., "e3", "d6")
                if ep_square_str.len() == 2 {
                    let file = ep_square_str.chars().next().unwrap();
                    let rank = ep_square_str.chars().nth(1).unwrap();

                    if !('a'..='h').contains(&file) || !('1'..='8').contains(&rank) {
                        return Err(format!("Invalid en passant square: {}", ep_square_str).into());
                    }

                    // En passant square should only be on rank 3 (for white) or rank 6 (for black)
                    if (position.side == Side::White && rank != '6')
                        || (position.side == Side::Black && rank != '3')
                    {
                        return Err(format!(
                            "Invalid en passant square {} for side to move",
                            ep_square_str
                        )
                        .into());
                    }

                    // Calculate the pawn's actual square and where it moved from
                    // The en passant square is where the pawn would be captured, not where it moved to
                    let file_index = (file as u8 - b'a') as usize;

                    // If white to move, black pawn moved (ep_square is rank 6, pawn is on rank 5)
                    // If black to move, white pawn moved (ep_square is rank 3, pawn is on rank 4)
                    let (pawn_to, pawn_from) = if position.side == Side::White {
                        // Black pawn moved from rank 7 to rank 5, ep_square is rank 6
                        let pawn_square = file_index + 4 * 8; // rank 5 (0-indexed: rank 4)
                        let from_square = file_index + 6 * 8; // rank 7 (0-indexed: rank 6)
                        (
                            Square::try_from(pawn_square as u8)
                                .map_err(|e| format!("Invalid pawn square: {}", e))?,
                            Square::try_from(from_square as u8)
                                .map_err(|e| format!("Invalid from square: {}", e))?,
                        )
                    } else {
                        // White pawn moved from rank 2 to rank 4, ep_square is rank 3
                        let pawn_square = file_index + 3 * 8; // rank 4 (0-indexed: rank 3)
                        let from_square = file_index + 1 * 8; // rank 2 (0-indexed: rank 1)
                        (
                            Square::try_from(pawn_square as u8)
                                .map_err(|e| format!("Invalid pawn square: {}", e))?,
                            Square::try_from(from_square as u8)
                                .map_err(|e| format!("Invalid from square: {}", e))?,
                        )
                    };

                    Some((pawn_from, pawn_to))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Parse halfmove clock (field 5) - number of halfmoves since last capture or pawn advance
        if parts.len() > 4 {
            match parts[4].parse::<u8>() {
                Ok(halfmove) => {
                    position.fifty = halfmove;
                }
                Err(_) => {
                    return Err(format!("Invalid halfmove clock: {}", parts[4]).into());
                }
            }
        } else {
            position.fifty = 0;
        }

        // Parse fullmove number (field 6) - increments after Black's move
        // We convert this to ply_from_start_of_game (halfmoves)
        if parts.len() > 5 {
            match parts[5].parse::<usize>() {
                Ok(fullmove) => {
                    if fullmove < 1 {
                        return Err("Fullmove number must be at least 1".into());
                    }
                    // Convert fullmove to ply: (fullmove - 1) * 2 + (0 if white to move, 1 if black)
                    position.ply_from_start_of_game =
                        (fullmove - 1) * 2 + if position.side == Side::Black { 1 } else { 0 };
                }
                Err(_) => {
                    return Err(format!("Invalid fullmove number: {}", parts[5]).into());
                }
            }
        } else {
            // Default to move 1 if not specified
            position.ply_from_start_of_game = if position.side == Side::Black { 1 } else { 0 };
        }

        // Now that ply_from_start_of_game is set, we can create the synthetic game_list entry for en passant
        if let Some((pawn_from, pawn_to)) = ep_game_entry {
            position.game_list[position.ply_from_start_of_game] = Some(Game {
                from: pawn_from,
                to: pawn_to,
                promote: None,
                capture: Piece::Empty, // TODO: Is this why guards were needed for search tests?
                fifty: position.fifty,
                castle: position.castle,
                hash: position.board.hash.current_key,
                en_passant_file: Some(COLUMN[pawn_to as usize]),
            });
        }

        // Initialize hash with castle rights
        position.board.hash.update_castle_rights(0, position.castle);

        // Initialize hash with side-to-move (if Black to move, toggle the hash)
        if position.side == Side::Black {
            position.board.hash.toggle_side_to_move();
        }

        // Initialize hash with en passant file if present
        if let Some((_, pawn_to)) = ep_game_entry {
            position
                .board
                .hash
                .update_en_passant(None, Some(COLUMN[pawn_to as usize]));
        }

        Ok(position)
    }

    /// Generate and display a FEN (Forsyth-Edwards Notation) string from the current position.
    ///
    /// Creates a complete FEN string with all six fields:
    /// 1. Piece placement (from white's perspective, rank 8 to rank 1)
    /// 2. Active color ("w" or "b")
    /// 3. Castling availability (KQkq or "-")
    /// 4. En passant target square (e.g., "e3" or "-")
    /// 5. Halfmove clock (number of halfmoves since last capture or pawn advance)
    /// 6. Fullmove number (starts at 1, increments after Black's move)
    ///
    /// The FEN string is printed to stdout.
    ///
    /// # Example
    /// ```ignore
    /// let mut position = Position::new();
    /// position.display_fen();
    /// // Prints: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// ```
    pub fn display_fen(&self) {
        let fen = self.to_fen();
        println!("{}", fen);
    }

    /// Generate a FEN (Forsyth-Edwards Notation) string from the current position.
    ///
    /// Creates a complete FEN string with all six fields:
    /// 1. Piece placement (from white's perspective, rank 8 to rank 1)
    /// 2. Active color ("w" or "b")
    /// 3. Castling availability (KQkq or "-")
    /// 4. En passant target square (e.g., "e3" or "-")
    /// 5. Halfmove clock (number of halfmoves since last capture or pawn advance)
    /// 6. Fullmove number (starts at 1, increments after Black's move)
    ///
    /// # Returns
    /// A String containing the FEN notation
    ///
    /// # Example
    /// ```ignore
    /// let mut position = Position::new();
    /// let fen = position.to_fen();
    /// assert_eq!(fen, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    /// ```
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // 1. Piece placement (from rank 8 to rank 1)
        for rank in (0..8).rev() {
            let mut empty_count = 0;

            for file in 0..8 {
                let square_idx = rank * 8 + file;
                let square = Square::try_from(square_idx as u8).unwrap();
                let piece = self.board.value[square as usize];

                if piece == Piece::Empty {
                    empty_count += 1;
                } else {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }

                    let is_white = self.board.bit_units[Side::White as usize].is_bit_set(square);

                    let piece_char = match piece {
                        Piece::King => {
                            if is_white {
                                'K'
                            } else {
                                'k'
                            }
                        }
                        Piece::Queen => {
                            if is_white {
                                'Q'
                            } else {
                                'q'
                            }
                        }
                        Piece::Rook => {
                            if is_white {
                                'R'
                            } else {
                                'r'
                            }
                        }
                        Piece::Bishop => {
                            if is_white {
                                'B'
                            } else {
                                'b'
                            }
                        }
                        Piece::Knight => {
                            if is_white {
                                'N'
                            } else {
                                'n'
                            }
                        }
                        Piece::Pawn => {
                            if is_white {
                                'P'
                            } else {
                                'p'
                            }
                        }
                        Piece::Empty => unreachable!(),
                    };
                    fen.push(piece_char);
                }
            }

            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }

            if rank > 0 {
                fen.push('/');
            }
        }

        // 2. Active color
        fen.push(' ');
        fen.push(if self.side == Side::White { 'w' } else { 'b' });

        // 3. Castling availability
        fen.push(' ');
        let mut castle_str = String::new();
        if self.castle & 1 != 0 {
            castle_str.push('K');
        }
        if self.castle & 2 != 0 {
            castle_str.push('Q');
        }
        if self.castle & 4 != 0 {
            castle_str.push('k');
        }
        if self.castle & 8 != 0 {
            castle_str.push('q');
        }
        fen.push_str(if castle_str.is_empty() {
            "-"
        } else {
            &castle_str
        });

        // 4. En passant target square
        fen.push(' ');
        if self.ply_from_start_of_game > 0 {
            if let Some(game) = self.game_list[self.ply_from_start_of_game] {
                if let Some(ep_file) = game.en_passant_file {
                    // Calculate the en passant target square
                    let ep_rank = if self.side == Side::White { 5 } else { 2 };
                    fen.push((b'a' + ep_file) as char);
                    fen.push((b'1' + ep_rank) as char);
                } else {
                    fen.push('-');
                }
            } else {
                fen.push('-');
            }
        } else {
            fen.push('-');
        }

        // 5. Halfmove clock
        fen.push(' ');
        fen.push_str(&self.fifty.to_string());

        // 6. Fullmove number
        fen.push(' ');
        let fullmove = (self.ply_from_start_of_game / 2) + 1;
        fen.push_str(&fullmove.to_string());

        fen
    }
}
