use std::panic;

use crate::{
    constants::{
        DEFAULT_MAX_DEPTH, DEFAULT_PLAYER_INCREMENT_MS, DEFAULT_PLAYER_TIME_REMAINING_MS,
        NUM_SIDES, NUM_SQUARES,
    },
    position::Position,
    time::TimeManager,
    types::{Piece, Side, Square},
    zobrist_hash::initialize_zobrist_hash_tables,
};

pub struct Engine {
    pub position: Position,
    pub search_settings: SearchSettings,
    pub computer_side: Option<Side>,
    history_table: [[[isize; NUM_SQUARES]; NUM_SQUARES]; NUM_SIDES], // [color][from][to] = score
}

pub struct SearchSettings {
    pub wtime: u64,
    pub btime: u64,
    pub winc: u64,
    pub binc: u64,
    pub movetime: Option<u64>,
    pub depth: u16,
}

pub struct SearchResult {
    pub best_move: String,
    pub ponder_move: Option<String>,
    pub evaluation: i32,
    pub depth: u16,
    pub nodes: u64,
    pub pv: Vec<String>,
    pub time_ms: u64,
}

impl Default for Engine {
    fn default() -> Self {
        Engine::new(None, None, None, None, None, None)
    }
}

impl Engine {
    pub fn new(
        wtime: Option<u64>,
        btime: Option<u64>,
        winc: Option<u64>,
        binc: Option<u64>,
        movetime: Option<u64>,
        depth: Option<u16>,
    ) -> Self {
        initialize_zobrist_hash_tables();

        let wtime = wtime.unwrap_or(DEFAULT_PLAYER_TIME_REMAINING_MS);
        let btime = btime.unwrap_or(DEFAULT_PLAYER_TIME_REMAINING_MS);
        let winc = winc.unwrap_or(DEFAULT_PLAYER_INCREMENT_MS);
        let binc = binc.unwrap_or(DEFAULT_PLAYER_INCREMENT_MS);
        let depth = depth.unwrap_or(DEFAULT_MAX_DEPTH);

        let mut engine = Engine {
            position: Position::new(TimeManager::new(wtime, btime, winc, binc, movetime, true)),
            search_settings: SearchSettings {
                wtime,
                btime,
                winc,
                binc,
                depth,
                movetime,
            },
            computer_side: None,
            history_table: [[[0; NUM_SQUARES]; NUM_SQUARES]; NUM_SIDES],
        };

        engine.generate_moves();
        engine
    }

    pub fn new_game(&mut self) {
        self.position = Position::new(TimeManager::new(
            self.search_settings.wtime,
            self.search_settings.btime,
            self.search_settings.winc,
            self.search_settings.binc,
            self.search_settings.movetime,
            true,
        ));

        self.computer_side = None;
    }

    pub fn generate_moves(&mut self) {
        self.position
            .generate_moves_and_captures(self.position.side, |_, _, _| 0);
    }

    /// Core iterative deepening search logic. Returns final depth reached and score achieved.
    pub fn think<F>(&mut self, mut on_depth_complete: Option<F>) -> (u16, i32)
    where
        F: FnMut(u16, i32, &mut Position),
    {
        // Handle panics from the hard time check
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
                if *msg == "TimeExhausted" {
                    return;
                }
            }

            default_hook(panic_info);
        }));

        self.position.time_manager = TimeManager::new(
            self.search_settings.wtime,
            self.search_settings.btime,
            self.search_settings.winc,
            self.search_settings.binc,
            self.search_settings.movetime,
            self.position.side == Side::White,
        );

        self.position.nodes = 0;

        // Reset history table at the start of the search
        self.history_table = [[[0; NUM_SQUARES]; NUM_SQUARES]; NUM_SIDES];

        let mut final_depth = 0;
        let mut final_score = 0;

        // Iterative deepening: search depth 1, 2, 3, ... maximum
        for depth in 1..=self.search_settings.depth {
            // Soft time limit (avoid starting a depth that won't finish)
            if self.search_settings.depth > 1 && depth > 1 {
                if self.position.time_manager.is_soft_limit_reached() {
                    break;
                }
            }

            self.position.ply = 0;
            self.position.first_move[0] = 0;

            // Perform the search at this depth
            let score = match panic::catch_unwind(panic::AssertUnwindSafe(|| {
                self.position
                    .search(-10000, 10000, depth, &mut self.history_table)
            })) {
                Ok(score) => score,
                Err(panic_payload) => {
                    if let Some(msg) = panic_payload.downcast_ref::<&str>() {
                        // Handle time exhaustion panic
                        if *msg == "TimeExhausted" {
                            // Ensure we've unwound all moves
                            while self.position.ply > 0 {
                                self.position.take_back_move();
                            }
                            break;
                        }
                    }

                    // Re-throw any other panics
                    panic::resume_unwind(panic_payload);
                }
            };

            while self.position.ply > 0 {
                self.position.take_back_move();
            }

            final_depth = depth;
            final_score = score;

            // Callback for depth-specific output or processing
            if let Some(ref mut callback) = on_depth_complete {
                callback(depth, score, &mut self.position);
            }

            if score > 9000 || score < -9000 {
                break;
            }
        }

        // Set hash_from and hash_to for retrieval by caller from best move
        if let (Some(from), Some(to)) = (self.position.best_move_from, self.position.best_move_to) {
            self.position.hash_from = Some(from);
            self.position.hash_to = Some(to);
        }

        (final_depth, final_score)
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

    /// Parse a UCI move string (e.g. "e2e4", "e7e8q") and return the from/to squares and promotion piece
    pub fn move_from_uci_string(move_str: &str) -> Result<(Square, Square, Option<Piece>), String> {
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

        Ok((from, to, promote))
    }

    /// Parse a move in algebraic notation (e2e4) and return the index in the move list
    pub fn parse_move_string(&mut self, move_str: &str) -> Option<usize> {
        if move_str.len() < 4 {
            return None;
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
            return None;
        }

        let from_file = (chars[0] as u8 - b'a') as usize;
        let from_rank = (chars[1] as u8 - b'1') as usize;
        let to_file = (chars[2] as u8 - b'a') as usize;
        let to_rank = (chars[3] as u8 - b'1') as usize;

        let from_square = from_rank * 8 + from_file;
        let to_square = to_rank * 8 + to_file;

        // Find matching move in move list
        for i in self.position.first_move[self.position.ply]
            ..self.position.first_move[self.position.ply + 1]
        {
            if let Some(mv) = self.position.move_list[i as usize] {
                if mv.from as usize == from_square && mv.to as usize == to_square {
                    return Some(i as usize);
                }
            }
        }

        None
    }

    pub fn display_legal_moves(&self) {
        for i in self.position.first_move[self.position.ply]
            ..self.position.first_move[self.position.ply + 1]
        {
            if let Some(mv) = self.position.move_list[i as usize] {
                print!(
                    "{} ",
                    Engine::move_to_uci_string(mv.from, mv.to, mv.promote, false)
                );
                if (i + 1) % 8 == 0 {
                    println!();
                }
            }
        }
        println!();
    }
}
