use crate::{
    constants::NUM_HASH_SLOTS,
    types::{Move, Piece, Side, Square},
    zobrist_hash::{
        ZOBRIST_CASTLE_HASH, ZOBRIST_EN_PASSANT_HASH, ZOBRIST_HASH_TABLE,
        ZOBRIST_SIDE_TO_MOVE_HASH, initialize_zobrist_hash_tables,
    },
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
        initialize_zobrist_hash_tables();

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

        if let Some(hash_table) = ZOBRIST_HASH_TABLE.get() {
            self.current_key ^= hash_table[side as usize][piece as usize][square as usize];
        }
    }

    /// Toggle side-to-move in the hash. Call this when switching turns.
    pub fn toggle_side_to_move(&mut self) {
        if let Some(&hash_key) = ZOBRIST_SIDE_TO_MOVE_HASH.get() {
            self.current_key ^= hash_key;
        }
    }

    /// Update castle rights in the hash when they change
    pub fn update_castle_rights(&mut self, old_castle: u8, new_castle: u8) {
        if old_castle == new_castle {
            return;
        }

        if let Some(hash_table) = ZOBRIST_CASTLE_HASH.get() {
            self.current_key ^= hash_table[old_castle as usize];
            self.current_key ^= hash_table[new_castle as usize];
        }
    }

    /// Update en passant file in the hash when it changes
    pub fn update_en_passant(&mut self, old_file: Option<u8>, new_file: Option<u8>) {
        if old_file == new_file {
            return;
        }

        if let Some(hash_table) = ZOBRIST_EN_PASSANT_HASH.get() {
            if let Some(file) = old_file {
                self.current_key ^= hash_table[file as usize];
            }
            if let Some(file) = new_file {
                self.current_key ^= hash_table[file as usize];
            }
        }
    }
}
