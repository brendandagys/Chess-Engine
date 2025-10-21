use chess_engine::constants::{MAX_DEPTH, MAX_SEARCH_DURATION_MS};
use chess_engine::position::Position;
use chess_engine::types::{Piece, Side, Square};
use chess_engine::utils::get_time;
use chess_engine::zobrist_hash::initialize_zobrist_hash_tables;
use std::io::{self, Write};

struct ChessEngine {
    position: Position,
    computer_side: Option<Side>,
    fixed_time: bool,
    fixed_depth: bool,
    max_search_duration_ms: u32,
    max_depth: u16,
    flip: bool,
    turn: u32,
    display_disabled: bool,
}

impl ChessEngine {
    fn new() -> Self {
        initialize_zobrist_hash_tables();
        let mut position = Position::new();
        position.set_material_scores();

        Self {
            position,
            computer_side: None,
            fixed_time: false,
            fixed_depth: false,
            max_search_duration_ms: MAX_SEARCH_DURATION_MS,
            max_depth: MAX_DEPTH,
            flip: false,
            turn: 0,
            display_disabled: false,
        }
    }

    fn new_game(&mut self) {
        self.position = Position::new();
        self.position
            .generate_moves_and_captures(self.position.side);
    }

    fn show_help(&self) {
        println!("======================= INFORMATION ======================");
        println!("h or help - Displays help on the commands");
        println!("d or D    - Displays board and toggles display setting");
        println!("moves     - Displays of list of possible moves");
        println!("fen       - Displays a FEN string for the current position");
        println!("f         - Flips the board");
        println!("q or quit - Quits the program");
        println!("================= CONTROLLING THE ENGINE =================");
        println!("go        - Starts the engine from the current position");
        println!("new       - Starts a new game");
        println!("p or play - The computer plays a move");
        println!("off       - Turns the computer player off");
        println!("switch    - Switches sides");
        println!("undo      - Takes back the last move");
        println!("===================== CONFIGURATION ======================");
        println!("fen <FEN> - Loads a FEN string");
        println!("sd        - Sets the search depth");
        println!("st        - Sets the time limit per move in seconds");
    }

    fn display_board(&self) {
        if self.display_disabled {
            return;
        }

        self.position.display_board(self.flip);
    }

    /// Parse a move in long algebraic notation (e.g., e2e4)
    /// and return the index in the move list.
    fn parse_move_string(&self, move_str: &str) -> Option<usize> {
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
        for i in 0..self.position.first_move[1] as usize {
            if let Some(mv) = self.position.move_list[i] {
                if mv.from as usize == from_square && mv.to as usize == to_square {
                    return Some(i);
                }
            }
        }

        None
    }

    fn move_string(from: Square, to: Square, promote: Option<Piece>) -> String {
        let from_file = (from as usize % 8) as u8 + b'a';
        let from_rank = (from as usize / 8) as u8 + b'1';
        let to_file = (to as usize % 8) as u8 + b'a';
        let to_rank = (to as usize / 8) as u8 + b'1';

        let mut result = format!(
            "\x1b[32m{}{} -> {}{}\x1b[0m",
            from_file as char, from_rank as char, to_file as char, to_rank as char
        );

        if let Some(piece) = promote {
            let promote_char = match piece {
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Rook => 'r',
                _ => 'q',
            };
            result.push(promote_char);
        }

        result
    }

    fn print_result(&mut self) {
        self.position.set_material_scores();
        self.position
            .generate_moves_and_captures(self.position.side);

        let mut has_legal_moves = false;
        for i in 0..self.position.first_move[1] as usize {
            if let Some(mv) = self.position.move_list[i] {
                if self
                    .position
                    .make_move_with_promotion(mv.from, mv.to, mv.promote)
                {
                    self.position.take_back_move();
                    has_legal_moves = true;
                    break;
                }
            }
        }

        // Check for stalemate with insufficient material
        if self.position.pawn_material_score[0] == 0
            && self.position.pawn_material_score[1] == 0
            && self.position.piece_material_score[0] <= 300
            && self.position.piece_material_score[1] <= 300
        {
            println!("1/2-1/2 {{Stalemate}}");
            self.new_game();
            self.computer_side = None;
            return;
        }

        if !has_legal_moves {
            self.position
                .generate_moves_and_captures(self.position.side);
            self.display_board();
            println!("GAME OVER ");

            let king_square = self.position.board.bit_pieces[self.position.side as usize]
                [Piece::King as usize]
                .next_bit();

            if self.position.is_square_attacked_by_side(
                self.position.side.opponent(),
                Square::try_from(king_square).unwrap(),
            ) {
                if self.position.side == Side::White {
                    println!("0-1 {{Black mates}}");
                } else {
                    println!("1-0 {{White mates}}");
                }
            } else {
                println!("1/2-1/2 {{Stalemate}}");
            }

            self.new_game();
            self.computer_side = None;
        } else if self.position.reps() >= 3 {
            println!("1/2-1/2 {{Draw by repetition}}");
            self.new_game();
            self.computer_side = None;
        } else if self.position.fifty >= 100 {
            println!("1/2-1/2 {{Draw by fifty move rule}}");
            self.new_game();
            self.computer_side = None;
        }
    }

    fn run_main_loop(&mut self) {
        self.display_board();

        loop {
            // Display current turn info
            println!("\n-------------------------------");
            println!(
                "*   Ply: {} | To move: {:?}   *",
                self.position.ply_from_start_of_game + 1,
                self.position.side
            );
            println!("-------------------------------");

            // Computer's turn
            if Some(self.position.side) == self.computer_side {
                println!("\nComputer is thinking...");

                // Set search parameters
                self.position.max_depth = self.max_depth;
                self.position.max_search_duration_ms = self.max_search_duration_ms;
                self.position.fixed_time = self.fixed_time;
                self.position.fixed_depth = self.fixed_depth;

                self.position.think();

                let (hash_from, hash_to) = if let (Some(from), Some(to)) =
                    (self.position.hash_from, self.position.hash_to)
                {
                    (from, to)
                } else {
                    // TODO: What is the purpose of this branch?
                    println!("(No legal moves)");
                    self.computer_side = None;
                    self.display_board();
                    self.position
                        .generate_moves_and_captures(self.position.side);
                    continue;
                };

                self.position.make_move(hash_from, hash_to);
                self.position.set_material_scores();

                let elapsed_time = get_time() - self.position.start_time;
                print!("\nTime: {} ms", elapsed_time);

                let nps = if elapsed_time > 0 {
                    (self.position.nodes as f64 / elapsed_time as f64) * 1000.0
                } else {
                    0.0
                };

                print!(" | Nodes/s: {}\n", nps as u64);

                println!(
                    "\nComputer plays: {}",
                    Self::move_string(hash_from, hash_to, None)
                );

                self.position.ply = 0;
                self.position.first_move[0] = 0;
                self.position
                    .generate_moves_and_captures(self.position.side);
                self.print_result();
                self.turn += 1;
                self.display_board();
                continue;
            }

            // Show available moves
            self.position.ply = 0;
            self.position.first_move[0] = 0;
            self.position
                .generate_moves_and_captures(self.position.side);

            print!("\nFrom square OR command > ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => return, // EOF
                Ok(_) => {}
                Err(_) => return,
            }

            let command = input.trim();

            // COMMANDS WITHOUT PARAMETERS
            match command {
                "d" => {
                    self.position.display_board(self.flip);
                    continue;
                }
                "D" => {
                    self.display_disabled = !self.display_disabled;

                    if self.display_disabled {
                        println!("\nBoard display disabled");
                    } else {
                        println!("\nBoard display enabled");
                        self.display_board();
                    }
                    continue;
                }
                "f" | "F" => {
                    self.flip = !self.flip;
                    self.display_board();
                    continue;
                }
                "go" => {
                    self.handle_go_command();
                    continue;
                }
                "h" | "help" => {
                    self.show_help();
                    continue;
                }
                "fen" => {
                    println!("\n{}", self.position.to_fen());
                    continue;
                }
                "moves" => {
                    println!("\nLegal moves:");
                    let move_count = self.position.first_move[1];
                    for i in 0..move_count as usize {
                        if let Some(mv) = self.position.move_list[i] {
                            print!("{} ", Self::move_string(mv.from, mv.to, mv.promote));
                            if (i + 1) % 8 == 0 {
                                println!();
                            }
                        }
                    }
                    println!();
                    continue;
                }
                "new" => {
                    self.new_game();
                    self.computer_side = None;
                    self.turn = 1;
                    self.display_board();
                    continue;
                }
                "p" | "play" => {
                    self.computer_side = Some(self.position.side);
                    continue;
                }
                "off" => {
                    self.computer_side = None;
                    continue;
                }
                "q" | "Q" | "quit" => {
                    println!("\nProgram exiting");
                    break;
                }
                "switch" => {
                    self.position.side = self.position.side.opponent();
                    self.position.other_side = self.position.other_side.opponent();
                    self.position
                        .generate_moves_and_captures(self.position.side);
                    continue;
                }
                "undo" => {
                    // TODO: Can this be improved? Should set material scores? Why is ply set to 0?
                    if self.position.ply_from_start_of_game == 0 {
                        println!("\nNo moves to undo");
                        continue;
                    }
                    self.computer_side = None;
                    self.position.take_back_move();
                    self.position.ply = 0;
                    self.position.first_move[0] = 0;
                    self.position
                        .generate_moves_and_captures(self.position.side);
                    self.display_board();
                    continue;
                }
                _ => {}
            }

            // COMMANDS WITH PARAMETERS
            if command.starts_with("fen ") {
                let fen_str = &command[4..];
                match self.position.load_fen(fen_str) {
                    Ok(_) => {
                        self.position.set_material_scores();
                        self.position
                            .generate_moves_and_captures(self.position.side);
                        self.display_board();
                        println!("FEN loaded successfully");
                    }
                    Err(e) => println!("Error loading FEN: {}", e),
                }
                continue;
            }

            if command.starts_with("sd ") {
                if let Ok(depth) = command[3..].parse::<u16>() {
                    self.max_depth = depth;
                    self.max_search_duration_ms = MAX_SEARCH_DURATION_MS;
                    // self.max_search_duration_ms = 1 << 25;
                    self.fixed_depth = true;
                    println!("Search depth set to {}", depth);
                }
                continue;
            }

            if command.starts_with("st ") {
                if let Ok(time) = command[3..].parse::<u32>() {
                    self.max_search_duration_ms = time * 1000;
                    self.max_depth = 64;
                    self.fixed_time = true;
                    println!("Search time set to {} seconds", time);
                }
                continue;
            }

            // PARSE "FROM" AND THEN "TO" SQUARE
            let from_square = self.parse_square(command[..2].trim());
            if from_square.is_none() {
                println!("\nInvalid from square");
                continue;
            }
            let from_square = from_square.unwrap();

            let mut to_input = String::new();

            let cleaned_command = command.replace(" ", "");

            match cleaned_command.len() {
                // Need to prompt for "to" square
                2 => {
                    // Get to square
                    print!("             To square > ");
                    io::stdout().flush().unwrap();

                    match io::stdin().read_line(&mut to_input) {
                        Ok(0) => return,
                        Ok(_) => {}
                        Err(_) => return,
                    }

                    println!();
                }
                // "to" square is included in command
                4 => {
                    to_input = cleaned_command[2..].to_string();
                }
                _ => {
                    println!("\nInvalid command format");
                    continue;
                }
            }

            let to_square = self.parse_square(to_input.trim());
            if to_square.is_none() {
                println!("\nInvalid to square");
                continue;
            }
            let to_square = to_square.unwrap();

            // Construct move string and try to make the move
            let move_str = format!(
                "{}{}{}{}",
                ((from_square % 8) as u8 + b'a') as char,
                ((from_square / 8) as u8 + b'1') as char,
                ((to_square % 8) as u8 + b'a') as char,
                ((to_square / 8) as u8 + b'1') as char
            );

            if let Some(move_idx) = self.parse_move_string(&move_str) {
                if let Some(mv) = self.position.move_list[move_idx] {
                    if !self
                        .position
                        .make_move_with_promotion(mv.from, mv.to, mv.promote)
                    {
                        println!("ILLEGAL MOVE");
                        continue;
                    }

                    self.position.set_material_scores();
                    self.position.ply = 0;
                    self.position.first_move[0] = 0;
                    self.position
                        .generate_moves_and_captures(self.position.side);
                    self.print_result();
                    self.turn += 1;
                    self.display_board();
                } else {
                    panic!("Move found in move list, but is `None`");
                }
            } else {
                println!("ILLEGAL MOVE");
            }
        }
    }

    fn parse_square(&self, input: &str) -> Option<usize> {
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

    fn handle_go_command(&mut self) {
        println!("\nChoose your side:");
        println!("1. White");
        println!("2. Black");
        println!("3. Random");
        print!("Enter choice (1-3)> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(_) => return,
        }

        let choice = input.trim();

        let player_side = match choice {
            "1" => Side::White,
            "2" => Side::Black,
            "3" => {
                use std::time::SystemTime;
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if now % 2 == 0 {
                    Side::White
                } else {
                    Side::Black
                }
            }
            _ => {
                println!("Invalid choice. Defaulting to White.");
                Side::White
            }
        };

        println!("You are playing as {:?}", player_side);
        self.computer_side = Some(player_side.opponent());

        // If computer is white, let it move first
        if self.computer_side == Some(Side::White) {
            println!("Computer plays first");
        }
    }
}

fn main() {
    println!("Brendan's Chess Engine");
    println!("Version 1.0, 2025-10-07");
    println!();
    println!("\"h or help\" displays a list of commands");
    println!();

    let mut engine = ChessEngine::new();
    engine.show_help();
    engine.run_main_loop();
}
