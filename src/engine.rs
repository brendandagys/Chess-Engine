use std::{panic, sync::Arc};

use crate::{
    constants::{
        DEFAULT_MAX_DEPTH, DEFAULT_PLAYER_INCREMENT_MS, DEFAULT_PLAYER_TIME_REMAINING_MS,
        INFINITY_SCORE, MATE_THRESHOLD, MAX_PLY, NUM_SIDES, NUM_SQUARES,
    },
    polyglot::PolyglotBook,
    position::Position,
    time::TimeManager,
    types::{Difficulty, MoveData, Piece, Side, Square},
};

pub struct Engine {
    pub difficulty: Option<Difficulty>,
    pub position: Position,
    pub search_settings: SearchSettings,
    pub computer_side: Option<Side>,
    history_table: [[[isize; NUM_SQUARES]; NUM_SQUARES]; NUM_SIDES], // [color][from][to] = score
    pub book: Option<PolyglotBook>,
}

pub struct SearchSettings {
    pub wtime: u64,
    pub btime: u64,
    pub winc: u64,
    pub binc: u64,
    pub movetime: Option<u64>,
    pub max_depth: u16,
    pub max_nodes: Option<usize>,
}

pub struct SearchResult {
    pub best_move_from: Option<Square>,
    pub best_move_to: Option<Square>,
    pub best_move_promote: Option<Piece>,
    pub evaluation: i32,
    pub depth: u16,
    pub nodes: usize,
    pub qnodes: usize,
    pub time_ms: u64,
    pub principal_variation: Vec<MoveData>, // Principal variation: list of (from, to, promote)
    pub from_book: bool,
}

impl Default for Engine {
    fn default() -> Self {
        Engine::new(None, None, None, None, None, None, None, None, None)
    }
}

impl Engine {
    pub fn new(
        wtime: Option<u64>,
        btime: Option<u64>,
        winc: Option<u64>,
        binc: Option<u64>,
        movetime: Option<u64>,
        max_depth: Option<u16>,
        max_nodes: Option<usize>,
        book_path: Option<&str>,
        difficulty: Option<Difficulty>,
    ) -> Self {
        let wtime = wtime.unwrap_or(DEFAULT_PLAYER_TIME_REMAINING_MS);
        let btime = btime.unwrap_or(DEFAULT_PLAYER_TIME_REMAINING_MS);
        let winc = winc.unwrap_or(DEFAULT_PLAYER_INCREMENT_MS);
        let binc = binc.unwrap_or(DEFAULT_PLAYER_INCREMENT_MS);

        let max_depth = difficulty
            .map(|d| d.max_depth() as u16)
            .or(max_depth)
            .unwrap_or(DEFAULT_MAX_DEPTH);

        let mut engine = Engine {
            position: Position::new(TimeManager::new(wtime, btime, winc, binc, movetime, true)),
            search_settings: SearchSettings {
                wtime,
                btime,
                winc,
                binc,
                movetime,
                max_depth,
                max_nodes,
            },
            computer_side: None,
            history_table: [[[0; NUM_SQUARES]; NUM_SQUARES]; NUM_SIDES],
            book: None,
            difficulty,
        };

        if let Some(book_path) = book_path {
            if let Err(e) = engine.load_opening_book(book_path) {
                panic!("{}", e);
            }
        }

        engine.generate_moves();
        engine
    }

    pub fn load_opening_book(&mut self, book_path: &str) -> Result<(), String> {
        match PolyglotBook::load(book_path) {
            Ok(book) => {
                self.book = Some(book);
                Ok(())
            }
            Err(e) => Err(format!("Failed to load opening book: {}", e)),
        }
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

    /// Core iterative deepening search logic. Returns SearchResult with best move and evaluation.
    pub fn think<F>(&mut self, mut on_depth_complete: Option<F>) -> SearchResult
    where
        F: FnMut(u16, i32, &mut Position),
    {
        // Don't print anything when known panics occur
        let default_hook: Arc<dyn Fn(&panic::PanicHookInfo<'_>) + Send + Sync + 'static> =
            Arc::from(panic::take_hook());

        panic::set_hook({
            let hook = Arc::clone(&default_hook);
            Box::new(move |panic_info: &panic::PanicHookInfo<'_>| {
                if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
                    if matches!(*msg, "TimeExhausted" | "NodeLimitReached") {
                        return;
                    }
                }

                hook(panic_info);
            }) as Box<dyn Fn(&panic::PanicHookInfo<'_>) + Send + Sync + 'static>
        });

        if let Some(book) = &self.book {
            if let Some(book_entry) = book.get_move_from_book(self.position.board.hash.current_key)
            {
                let book_move = book_entry.decode_move();

                if let Some(ref mut callback) = on_depth_complete {
                    self.position
                        .make_move(book_move.from, book_move.to, book_move.promote);
                    let score = -self.position.evaluate();
                    self.position.take_back_move();
                    callback(0, score, &mut self.position);
                }

                return SearchResult {
                    best_move_from: Some(book_move.from),
                    best_move_to: Some(book_move.to),
                    best_move_promote: book_move.promote,
                    evaluation: 0,
                    depth: 0,
                    nodes: 0,
                    qnodes: 0,
                    time_ms: 0,
                    principal_variation: vec![book_move],
                    from_book: true,
                };
            }
        }

        self.position.time_manager = TimeManager::new(
            self.search_settings.wtime,
            self.search_settings.btime,
            self.search_settings.winc,
            self.search_settings.binc,
            self.search_settings.movetime,
            self.position.side == Side::White,
        );

        // Reset all search statistics
        self.position.nodes = 0;
        self.position.qnodes = 0;
        self.position.max_depth_reached = 0;
        self.position.hash_hits = 0;
        self.position.hash_stores = 0;
        self.position.beta_cutoffs = 0;

        // Reset history table at the start of the search
        self.history_table = [[[0; NUM_SQUARES]; NUM_SQUARES]; NUM_SIDES];

        let mut final_depth = 0;
        let mut final_score = 0;

        // Save the principal variation from the last completed iteration
        let mut saved_pv_length = 0;
        let mut saved_pv = [None; MAX_PLY];

        // Iterative deepening: search depth 1, 2, 3, ... maximum
        for depth in 1..=self.search_settings.max_depth {
            // Soft time limit (avoid starting a depth that won't finish)
            if self.search_settings.max_depth > 1 && depth > 1 {
                if self.position.time_manager.is_soft_limit_reached() {
                    break;
                }
            }

            self.position.ply = 0;
            self.position.first_move[0] = 0;

            // Perform the search at this depth
            let score = match panic::catch_unwind(panic::AssertUnwindSafe(|| {
                self.position.search(
                    -INFINITY_SCORE,
                    INFINITY_SCORE,
                    depth,
                    &mut self.history_table,
                    self.search_settings.max_nodes,
                )
            })) {
                Ok(score) => score,
                Err(panic_payload) => {
                    if let Some(panic_message) = panic_payload.downcast_ref::<&str>() {
                        if matches!(*panic_message, "TimeExhausted" | "NodeLimitReached") {
                            while self.position.ply > 0 {
                                self.position.take_back_move();
                            }

                            // Restore PV from the last completed iteration
                            self.position.pv_length[0] = saved_pv_length;
                            for i in 0..saved_pv_length {
                                self.position.pv_table[0][i] = saved_pv[i];
                            }

                            break;
                        }
                    }

                    panic::resume_unwind(panic_payload);
                }
            };

            while self.position.ply > 0 {
                self.position.take_back_move();
            }

            final_depth = depth;
            final_score = score;

            // Save the PV from this completed iteration
            saved_pv_length = self.position.pv_length[0];
            for i in 0..saved_pv_length {
                saved_pv[i] = self.position.pv_table[0][i];
            }

            if let Some(ref mut callback) = on_depth_complete {
                callback(depth, score, &mut self.position);
            }

            if score > MATE_THRESHOLD || score < -MATE_THRESHOLD {
                break;
            }
        }

        // Collect principal variation from position
        let mut principal_variation = Vec::new();

        for i in 0..self.position.pv_length[0] {
            if let Some(move_) = self.position.pv_table[0][i] {
                principal_variation.push(MoveData {
                    from: move_.from,
                    to: move_.to,
                    promote: move_.promote,
                });
            }
        }

        SearchResult {
            best_move_from: principal_variation[0].from.into(),
            best_move_to: principal_variation[0].to.into(),
            best_move_promote: principal_variation[0].promote,
            evaluation: final_score,
            depth: final_depth,
            nodes: self.position.nodes,
            qnodes: self.position.qnodes,
            time_ms: self.position.time_manager.elapsed().as_millis() as u64,
            principal_variation,
            from_book: false,
        }
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
        let mut moves = Vec::new();

        for i in self.position.first_move[self.position.ply]
            ..self.position.first_move[self.position.ply + 1]
        {
            if let Some(mv) = self.position.move_list[i as usize] {
                moves.push(Engine::move_to_uci_string(
                    mv.from, mv.to, mv.promote, false,
                ));
            }
        }

        moves.sort();

        for (index, move_str) in moves.iter().enumerate() {
            print!("{} ", move_str);
            if (index + 1) % 8 == 0 {
                println!();
            }
        }
        println!();
    }
}
