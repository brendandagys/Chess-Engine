/// NOTE: 1 ply = one move by a single player

pub const NUM_SQUARES: usize = 64;
pub const NUM_PIECE_TYPES: usize = 6;
pub const NUM_SIDES: usize = 2;
pub const NUM_RANKS: usize = 8;
pub const NUM_FILES: usize = 8;

/// Maximum search depth (in full moves). Used to size arrays with per-depth information.
pub const DEFAULT_MAX_DEPTH: u16 = 5;

/// Hard limit for maximum search depth (in ply). Used to size arrays with per-ply information.
pub const MAX_PLY: usize = 64;

// Time
pub const DEFAULT_PLAYER_TIME_REMAINING_MS: u64 = 300_000; // 5 minutes
pub const DEFAULT_PLAYER_INCREMENT_MS: u64 = 0;
pub const DEFAULT_FIXED_TIME: bool = false;
pub const DEFAULT_MOVETIME_MS: u64 = 1000; // Value for fixed-time mode

pub const SOFT_TO_HARD_LIMIT_RATIO: f64 = 0.75; // Hard limit is 1/30 of time remaining, plus increment

pub const MAX_HISTORY_SCORE: isize = 10000;

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

// Hash table configuration

// You could use a power-of-2 size for faster modulo (bitwise AND instead of %)
// const NUM_HASH_SLOTS: usize = 1 << 22; // 4,194,304 (2^22)
// let index = (self.current_key as usize) & (NUM_HASH_SLOTS - 1); // Faster than %
pub const NUM_HASH_SLOTS: usize = 5_000_000;

#[rustfmt::skip]
pub const INIT_COLOR: [u8; NUM_SQUARES] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1
];

#[rustfmt::skip]
pub const INIT_BOARD: [u8; NUM_SQUARES] = [
    3, 1, 2, 4, 5, 2, 1, 3,
    0, 0, 0, 0, 0, 0, 0, 0,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6,
    0, 0, 0, 0, 0, 0, 0, 0,
    3, 1, 2, 4, 5, 2, 1, 3
];

#[rustfmt::skip]
pub const ROW: [u8; NUM_SQUARES] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1,
    2, 2, 2, 2, 2, 2, 2, 2,
    3, 3, 3, 3, 3, 3, 3, 3,
    4, 4, 4, 4, 4, 4, 4, 4,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 6, 6, 6, 6, 6, 6, 6,
    7, 7, 7, 7, 7, 7, 7, 7
];

#[rustfmt::skip]
pub const COLUMN: [u8; NUM_SQUARES] = [
    0, 1, 2, 3, 4, 5, 6, 7,
    0, 1, 2, 3, 4, 5, 6, 7,
    0, 1, 2, 3, 4, 5, 6, 7,
    0, 1, 2, 3, 4, 5, 6, 7,
    0, 1, 2, 3, 4, 5, 6, 7,
    0, 1, 2, 3, 4, 5, 6, 7,
    0, 1, 2, 3, 4, 5, 6, 7,
    0, 1, 2, 3, 4, 5, 6, 7
];

#[rustfmt::skip]
pub const NORTH_WEST_DIAGONAL: [u8; NUM_SQUARES] = [
    14, 13, 12, 11, 10, 9, 8, 7,
	  13, 12, 11, 10,  9, 8, 7, 6,
	  12, 11, 10,  9,  8, 7, 6, 5,
	  11, 10,  9,  8,  7, 6, 5, 4,
	  10,  9,  8,  7,  6, 5, 4, 3,
	   9,  8,  7,  6,  5, 4, 3, 2,
	   8,  7,  6,  5,  4, 3, 2, 1,
	   7,  6,  5,  4,  3, 2, 1, 0
];

#[rustfmt::skip]
pub const NORTH_EAST_DIAGONAL: [u8; NUM_SQUARES] = [
    7, 8, 9, 10, 11, 12, 13, 14,
	  6, 7, 8,  9, 10, 11, 12, 13,
	  5, 6, 7,  8,  9, 10, 11, 12,
	  4, 5, 6,  7,  8,  9, 10, 11,
	  3, 4, 5,  6,  7,  8,  9, 10,
	  2, 3, 4,  5,  6,  7,  8,  9,
	  1, 2, 3,  4,  5,  6,  7,  8,
	  0, 1, 2,  3,  4,  5,  6,  7
];

#[rustfmt::skip]
pub const FLIPPED_BOARD_SQUARE: [u8; NUM_SQUARES] = [
    56, 57, 58, 59, 60, 61, 62, 63,
    48, 49, 50, 51, 52, 53, 54, 55,
    40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39,
    24, 25, 26, 27, 28, 29, 30, 31,
    16, 17, 18, 19, 20, 21, 22, 23,
     8,  9, 10, 11, 12, 13, 14, 15,
     0,  1,  2,  3,  4,  5,  6,  7
];

/// A1 - H8
#[rustfmt::skip]
pub const PAWN_SCORE: [i32; NUM_SQUARES] = [
	    0,   0,   0,   0,   0,   0,   0,   0,
	    0,   2,   4, -12, -12,   4,   2,   0,
	    0,   2,   4,   4,   4,   4,   2,   0,
	    0,   2,   4,   8,   8,   4,   2,   0,
	    0,   2,   4,   8,   8,   4,   2,   0,
	    4,   8,  10,  16,  16,  10,   8,   4,
	  100, 100, 100, 100, 100, 100, 100, 100,
	    0,   0,   0,   0,   0,   0,   0,   0
];

#[rustfmt::skip]
pub const KNIGHT_SCORE: [i32; NUM_SQUARES] = [
	   -30, -20, -10, -8, -8, -10, -20,  -30,
	   -16,  -6,  -2,  0,  0,  -2,  -6,  -16,
	    -8,  -2,   4,  6,  6,   4,  -2,   -8,
	    -5,   0,   6,  8,  8,   6,   0,   -5,
	    -5,   0,   6,  8,  8,   6,   0,   -5,
	   -10,  -2,   4,  6,  6,   4,  -2,  -10,
	   -20, -10,  -2,  0,  0,  -2, -10,  -20,
	  -150, -20, -10, -5, -5, -10, -20, -150
];

#[rustfmt::skip]
pub const BISHOP_SCORE: [i32; NUM_SQUARES] = [
	  -10, -10, -12, -10, -10, -12, -10, -10,
	    0,   4,   4,   4,   4,   4,   4,   0,
	    2,   4,   6,   6,   6,   6,   4,   2,
	    2,   4,   6,   8,   8,   6,   4,   2,
	    2,   4,   6,   8,   8,   6,   4,   2,
	    2,   4,   6,   6,   6,   6,   4,   2,
	  -10,   4,   4,   4,   4,   4,   4, -10,
	  -10, -10, -10, -10, -10, -10, -10, -10
];

#[rustfmt::skip]
pub const ROOK_SCORE: [i32; NUM_SQUARES] = [
	   4,  4,  4,  6,  6,  4,  4,  4,
	   0,  0,  0,  0,  0,  0,  0,  0,
	   0,  0,  0,  0,  0,  0,  0,  0,
	   0,  0,  0,  0,  0,  0,  0,  0,
	   0,  0,  0,  0,  0,  0,  0,  0,
	   0,  0,  0,  0,  0,  0,  0,  0,
	  20, 20, 20, 20, 20, 20, 20, 20,
	  10, 10, 10, 10, 10, 10, 10, 10
];

#[rustfmt::skip]
pub const QUEEN_SCORE: [i32; NUM_SQUARES] = [
	  -10, -10, -6, -4, -4, -6, -10, -10,
	  -10,   2,  2,  2,  2,  2,   2, -10,
	    2,   2,  2,  3,  3,  2,   2,   2,
	    2,   2,  3,  4,  4,  3,   2,   2,
	    2,   2,  3,  4,  4,  3,   2,   2,
	    2,   2,  2,  3,  3,  2,   2,   2,
	  -10,   2,  2,  2,  2,  2,   2, -10,
	  -10, -10,  2,  2,  2,  2, -10, -10
];

#[rustfmt::skip]
pub const KING_SCORE: [i32; NUM_SQUARES] = [
	   20,  20,  20, -40,  10, -60,  20,  20,     
	   15,  20, -25, -30, -30, -45,  20,  15,   
	  -48, -48, -48, -48, -48, -48, -48, -48,
	  -48, -48, -48, -48, -48, -48, -48, -48,
	  -48, -48, -48, -48, -48, -48, -48, -48,
	  -48, -48, -48, -48, -48, -48, -48, -48,
	  -48, -48, -48, -48, -48, -48, -48, -48,
	  -48, -48, -48, -48, -48, -48, -48, -48
];

#[rustfmt::skip]
pub const KING_ENDGAME_SCORE: [i32; NUM_SQUARES] = [
	   0,  8, 16, 18, 18, 16,  8,  0,
	   8, 16, 24, 32, 32, 24, 16,  8,
	  16, 24, 32, 40, 40, 32, 24, 16,
	  25, 32, 40, 48, 48, 40, 32, 25,
	  25, 32, 40, 48, 48, 40, 32, 25,
	  16, 24, 32, 40, 40, 32, 24, 16,
	   8, 16, 24, 32, 32, 24, 16,  8,
	   0,  8, 16, 18, 18, 16,  8,  0
];

#[rustfmt::skip]
pub const PASSED_SCORE: [i32; NUM_SQUARES] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     8,  8,  8,  8,  8,  8,  8,  8,
     8,  8,  8,  8,  8,  8,  8,  8,
    15, 15, 15, 15, 15, 15, 15, 15,
    30, 30, 30, 30, 30, 30, 30, 30,
    60, 60, 60, 60, 60, 60, 60, 60,
     0,  0,  0,  0,  0,  0,  0,  0, // 7th rank; always passed; handled by `pawn_score`
     0,  0,  0,  0,  0,  0,  0,  0
];

/// Used to determine the castling permissions after a move.
/// We logical-AND the castle bits with the CASTLE_MASK bits for
/// both of the move's squares.
///
/// If castle is 1 (white can castle kingside), and we play a move
/// where the rook on h1 gets captured, we AND castle with
/// CASTLE_MASK[63] (1&14).
///
/// Castle becomes 0 and white can't castle kingside anymore.
/// 
/// 0001 white kingside  (14: 1110)
/// 0010 white queenside (13: 1101)
/// 0100 black kingside  (11: 1011)
/// 1000 black queenside (7: 0111)
///
/// 12: 1100
///  3: 0011
/// 15: 1111
#[rustfmt::skip]
pub const CASTLE_MASK: [u8; NUM_SQUARES] = [
	  13, 15, 15, 15, 12, 15, 15, 14,
	  15, 15, 15, 15, 15, 15, 15, 15,
	  15, 15, 15, 15, 15, 15, 15, 15,
	  15, 15, 15, 15, 15, 15, 15, 15,
	  15, 15, 15, 15, 15, 15, 15, 15,
	  15, 15, 15, 15, 15, 15, 15, 15,
	  15, 15, 15, 15, 15, 15, 15, 15,
	   7, 15, 15, 15,  3, 15, 15, 11
];

#[rustfmt::skip]
pub const LSB_64_TABLE: [u8; 64] = [
    63, 30,  3, 32, 59, 14, 11, 33,
    60, 24, 50,  9, 55, 19, 21, 34,
    61, 29,  2, 53, 51, 23, 41, 18,
    56, 28,  1, 43, 46, 27,  0, 35,
    62, 31, 58,  4,  5, 49, 54,  6,
    15, 52, 12, 40,  7, 42, 45, 16,
    25, 57, 48, 13, 10, 39,  8, 44,
    20, 47, 38, 22, 17, 37, 36, 26
];

#[rustfmt::skip]
pub const QUEENSIDE_DEFENSE: [[i32; NUM_SQUARES]; NUM_SIDES] = [
  [
  	0,  0, 0, 0, 0, 0, 0, 0,
	  8, 10, 8, 0, 0, 0, 0, 0,
	  8,  6, 8, 0, 0, 0, 0, 0,
	  0,  0, 0, 0, 0, 0, 0, 0,
	  0,  0, 0, 0, 0, 0, 0, 0,
	  0,  0, 0, 0, 0, 0, 0, 0,
	  0,  0, 0, 0, 0, 0, 0, 0,
	  0,  0, 0, 0, 0, 0, 0, 0
  ],
  [
  	0,  0, 0, 0, 0, 0, 0, 0,
	  0,  0, 0, 0, 0, 0, 0, 0,
	  0,  0, 0, 0, 0, 0, 0, 0,
	  0,  0, 0, 0, 0, 0, 0, 0,
	  0,  0, 0, 0, 0, 0, 0, 0,
	  8,  6, 8, 0, 0, 0, 0, 0,
	  8, 10, 8, 0, 0, 0, 0, 0,
	  0,  0, 0, 0, 0, 0, 0, 0
  ]
];

#[rustfmt::skip]
pub const KINGSIDE_DEFENSE: [[i32; NUM_SQUARES]; NUM_SIDES] = [
  [
  	0, 0, 0, 0, 0, 0,  0, 0,
	  0, 0, 0, 0, 0, 8, 10, 8,
	  0, 0, 0, 0, 0, 8,  6, 8,
	  0, 0, 0, 0, 0, 0,  0, 0,
	  0, 0, 0, 0, 0, 0,  0, 0,
	  0, 0, 0, 0, 0, 0,  0, 0,
	  8, 6, 8, 0, 0, 8,  8, 8,
	  0, 0, 0, 0, 0, 0,  0, 0
  ],
  [
  	0, 0, 0, 0, 0, 0,  0, 0,
	  0, 0, 0, 0, 0, 0,  0, 0,
	  0, 0, 0, 0, 0, 0,  0, 0,
	  0, 0, 0, 0, 0, 0,  0, 0,
	  0, 0, 0, 0, 0, 0,  0, 0,
	  0, 0, 0, 0, 0, 8,  6, 8,
	  0, 0, 0, 0, 0, 8, 10, 8,
	  0, 0, 0, 0, 0, 0,  0, 0
  ]
];

// TODO: remove last element (for Empty piece)
pub const PAWN_CAPTURE_SCORE: [i32; 7] = [0, 10, 20, 30, 40, 0, 0]; // Last element is for Empty piece
pub const KNIGHT_CAPTURE_SCORE: [i32; 7] = [-3, 7, 17, 27, 37, 0, 0]; // Last element is for Empty piece
pub const BISHOP_CAPTURE_SCORE: [i32; 7] = [-3, 7, 17, 27, 37, 0, 0]; // Last element is for Empty piece
pub const ROOK_CAPTURE_SCORE: [i32; 7] = [-5, 5, 15, 25, 35, 0, 0]; // Last element is for Empty piece
pub const QUEEN_CAPTURE_SCORE: [i32; 7] = [-9, 1, 11, 21, 31, 0, 0]; // Last element is for Empty piece
pub const KING_CAPTURE_SCORE: [i32; 7] = [0, 10, 20, 30, 40, 0, 0]; // Last element is for Empty piece

pub const REVERSE_SQUARE: [i32; NUM_SIDES] = [-8, 8];

pub const ISOLATED_PAWN_SCORE: i32 = -20;
