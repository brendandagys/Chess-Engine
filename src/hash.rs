use crate::{
    constants::NUM_HASH_SLOTS,
    polyglot::POLYGLOT,
    types::{Move, Piece, Side, Square},
};

/// An entry in the transposition table storing the best move for a position
#[derive(Clone, Copy)]
pub struct HashEntry {
    /// The hash key for collision detection (instead of separate "lock")
    pub hash_key: u64,
    /// Best move found for this position
    pub best_move: Option<Move>,
    /// Search depth at which this entry was stored
    pub depth: u8,
    /// Score/evaluation for this position
    pub score: i32,
}

impl Default for HashEntry {
    fn default() -> Self {
        Self {
            hash_key: 0,
            best_move: None,
            depth: 0,
            score: 0,
        }
    }
}

/// Transposition table for storing positions and their best moves
pub struct HashTable {
    entries: Vec<HashEntry>,
}

impl HashTable {
    pub fn new() -> Self {
        Self {
            entries: vec![HashEntry::default(); NUM_HASH_SLOTS],
        }
    }

    /// Get an entry at the given index
    fn get(&self, index: usize) -> &HashEntry {
        &self.entries[index]
    }

    /// Get a mutable entry at the given index
    fn get_mut(&mut self, index: usize) -> &mut HashEntry {
        &mut self.entries[index]
    }
}

/// Zobrist hash manager for incremental position hashing
pub struct Hash {
    /// Current Zobrist hash key for the position
    pub current_key: u64,
    /// Transposition table
    hash_table: HashTable,
}

impl Hash {
    pub fn new() -> Self {
        Self {
            current_key: 0,
            hash_table: HashTable::new(),
        }
    }

    /// Store a move in the hash table for the current position
    pub fn store_move(&mut self, move_: Move, depth: u8, score: i32) {
        let index = (self.current_key as usize) % NUM_HASH_SLOTS;
        let entry = self.hash_table.get_mut(index);

        if entry.hash_key != self.current_key || depth >= entry.depth {
            entry.hash_key = self.current_key;
            entry.best_move = Some(move_);
            entry.depth = depth;
            entry.score = score;
        }
    }

    /// Look up the hash entry for the current position, if available
    pub fn probe(&self) -> Option<&HashEntry> {
        let index = (self.current_key as usize) % NUM_HASH_SLOTS;
        let entry = self.hash_table.get(index);

        // Verify this is the same position (collision detection) and has a move stored
        if entry.hash_key == self.current_key && entry.best_move.is_some() {
            Some(entry)
        } else {
            None
        }
    }

    /// Update hash for a piece on a square. Call this twice per move (from/to).
    pub fn toggle_piece(&mut self, side: Side, piece: Piece, square: Square) {
        if piece == Piece::Empty {
            return;
        }

        let piece_index = match (side, piece) {
            (Side::Black, Piece::Pawn) => 0,
            (Side::White, Piece::Pawn) => 1,
            (Side::Black, Piece::Knight) => 2,
            (Side::White, Piece::Knight) => 3,
            (Side::Black, Piece::Bishop) => 4,
            (Side::White, Piece::Bishop) => 5,
            (Side::Black, Piece::Rook) => 6,
            (Side::White, Piece::Rook) => 7,
            (Side::Black, Piece::Queen) => 8,
            (Side::White, Piece::Queen) => 9,
            (Side::Black, Piece::King) => 10,
            (Side::White, Piece::King) => 11,
            _ => return,
        };

        self.current_key ^=
            POLYGLOT[64 * piece_index + 8 * square.rank() as usize + square.file() as usize];
    }

    /// Toggle side-to-move in the hash
    pub fn toggle_side_to_move(&mut self) {
        self.current_key ^= POLYGLOT[780];
    }

    /// Update castle rights in the hash when they change
    pub fn update_castle_rights(&mut self, old_castle: u8, new_castle: u8) {
        if old_castle == new_castle {
            return;
        }

        if old_castle & 1 != new_castle & 1 {
            self.current_key ^= POLYGLOT[768]; // White short
        }
        if old_castle & 2 != new_castle & 2 {
            self.current_key ^= POLYGLOT[769]; // White long
        }
        if old_castle & 4 != new_castle & 4 {
            self.current_key ^= POLYGLOT[770]; // Black short
        }
        if old_castle & 8 != new_castle & 8 {
            self.current_key ^= POLYGLOT[771]; // Black long
        }
    }

    /// Update en passant file in the hash when it changes
    pub fn update_en_passant(&mut self, old_file: Option<u8>, new_file: Option<u8>) {
        // Remove old en passant if it existed
        if let Some(f) = old_file {
            if f < 8 {
                self.current_key ^= POLYGLOT[772 + f as usize];
            }
        }

        // Add new en passant if it exists
        if let Some(f) = new_file {
            if f < 8 {
                self.current_key ^= POLYGLOT[772 + f as usize];
            }
        }
    }
}
