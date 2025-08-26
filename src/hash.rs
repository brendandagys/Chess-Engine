use crate::{
    constants::{MAX_HASH, NUM_SIDES},
    types::{Move, Piece, Side, Square},
    zobrist_hash::{ZOBRIST_HASH_TABLE, ZOBRIST_LOCK_TABLE, initialize_zobrist_hash_tables},
};

#[derive(Clone, Copy)]
pub struct HashPosition {
    pub hash_lock: u64,
    pub from: Square, // Best move found for position
    pub to: Square,   // Best move found for position
}

impl Default for HashPosition {
    fn default() -> Self {
        Self {
            hash_lock: 0,
            from: Square::A1,
            to: Square::A1,
        }
    }
}

pub struct HashTable {
    positions: Vec<HashPosition>,
}

impl HashTable {
    pub fn new() -> Self {
        Self {
            positions: vec![HashPosition::default(); MAX_HASH],
        }
    }
}

pub struct Hash {
    current_key: u64,
    current_lock: u64,
    collisions: u64,
    hash_tables: [HashTable; NUM_SIDES],
}

impl Hash {
    pub fn new() -> Self {
        initialize_zobrist_hash_tables();

        Self {
            current_key: 0,
            current_lock: 0,
            collisions: 0,
            hash_tables: [HashTable::new(), HashTable::new()],
        }
    }

    /// Add an entry to the hash table, possibly overwriting
    pub fn update_position(&mut self, side: usize, move_: Move) {
        let index = (self.current_key as usize) % MAX_HASH;
        let entry = &mut self.hash_tables[side].positions[index];

        entry.hash_lock = self.current_lock;
        entry.from = move_.from;
        entry.to = move_.to;
    }

    /// Update the current key and lock. Called when pieces are moved on the board.
    pub fn update_position_hash_key(&mut self, side: Side, piece: Piece, square: Square) {
        if let (Some(hash_table), Some(lock_table)) =
            (ZOBRIST_HASH_TABLE.get(), ZOBRIST_LOCK_TABLE.get())
        {
            self.current_key ^= hash_table[side as usize][piece as usize][square as usize];
            self.current_lock ^= lock_table[side as usize][piece as usize][square as usize];
        }
    }

    pub fn lookup(&mut self, side: usize) -> Option<(Square, Square)> {
        if let Some(table) = self.hash_tables.get(side) {
            let key = (self.current_key as usize) % MAX_HASH;
            let entry = table.positions[key];

            if entry.hash_lock != self.current_lock {
                self.collisions += 1;
                return None;
            }

            return Some((entry.from, entry.to));
        }

        None
    }
}
