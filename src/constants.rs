/// NOTE: 1 ply = one move by a single player

pub const NUM_SQUARES: usize = 64;
pub const NUM_PIECE_TYPES: usize = 6;
pub const NUM_SIDES: usize = 2;
pub const NUM_RANKS: usize = 8;
pub const NUM_FILES: usize = 8;

pub const MAX_PLY: usize = 64;

/// Maximum size for total moves in the move list.
/// With 40 moves/position on average, allows for 50-ply depth.
pub const MOVE_STACK: usize = 4000;

/// Stores moves for the entire game.
/// Stores information about a move so it can be taken back.
pub const GAME_STACK: usize = 2000;

/// Added to move score so that the move from the hash table is searched first.
pub const HASH_SCORE: i32 = 100_000_000;

/// Added to move score so that captures are search right after the hash table.
pub const CAPTURE_SCORE: i32 = 10_000_000;
