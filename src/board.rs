use crate::{
    constants::{INIT_BOARD, INIT_COLOR, NUM_PIECE_TYPES, NUM_SIDES, NUM_SQUARES},
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
}
