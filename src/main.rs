use chess_engine::position::Position;
use chess_engine::types::{Piece, Side, Square};
use chess_engine::utils::get_time;
use chess_engine::zobrist_hash::initialize_zobrist_hash_tables;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

struct ChessEngine {
    position: Position,
    computer_side: Option<Side>,
    fixed_time: bool,
    fixed_depth: bool,
    max_time: u32,
    max_depth: u16,
    flip: bool,
    turn: u32,
    player: [bool; 2],
}

impl ChessEngine {
    fn new() -> Self {
        initialize_zobrist_hash_tables();
        let mut position = Position::new();
        position.set_material();

        Self {
            position,
            computer_side: None,
            fixed_time: false,
            fixed_depth: false,
            max_time: 1 << 25,
            max_depth: 4,
            flip: false,
            turn: 0,
            player: [false, false],
        }
    }

    fn new_game(&mut self) {
        self.position = Position::new();
        self.position.generate_moves(self.position.side);
    }

    fn show_help(&self) {
        println!("d - Displays the board.");
        println!("f - Flips the board.");
        println!("go - Starts the engine.");
        println!("help - Displays help on the commands.");
        println!("moves - Displays of list of possible moves.");
        println!("new - Starts a new game.");
        println!("off - Turns the computer player off.");
        println!("on or p - The computer plays a move.");
        println!("sb - Loads a fen diagram.");
        println!("sd - Sets the search depth.");
        println!("st - Sets the time limit per move in seconds.");
        println!("sw - Switches sides.");
        println!("quit - Quits the program.");
        println!("undo - Takes back the last move.");
        println!("xboard - Starts xboard.");
    }

    fn display_board(&self) {
        self.position.display_board(self.flip);
    }

    fn parse_move(&self, move_str: &str) -> Option<usize> {
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
            "{}{}{}{}",
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
        self.position.set_material();
        self.position.generate_moves(self.position.side);

        let mut has_legal_moves = false;
        for i in 0..self.position.first_move[1] as usize {
            if let Some(mv) = self.position.move_list[i] {
                if self
                    .position
                    .make_move_with_promotion(mv.from, mv.to, mv.promote)
                {
                    self.position.take_back();
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
            self.position.generate_moves(self.position.side);
            self.display_board();
            println!(" end of game ");

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

    fn load_diagram(
        &mut self,
        filename: &str,
        _num: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

        if lines.is_empty() {
            return Err("Empty FEN file".into());
        }

        let fen = &lines[0];
        self.position.load_fen(fen)?;

        self.display_board();
        self.position.new_position();
        self.position.generate_moves(self.position.side);

        println!(" diagram loaded");
        if self.position.side == Side::White {
            println!("White to move");
        } else {
            println!("Black to move");
        }
        println!(" {} ", fen);

        Ok(())
    }

    fn run_main_loop(&mut self) {
        loop {
            // Computer's turn
            if Some(self.position.side) == self.computer_side {
                self.player[self.position.side as usize] = true;

                // Set search parameters
                self.position.max_depth = self.max_depth;
                self.position.max_time = self.max_time;
                self.position.fixed_time = self.fixed_time;
                self.position.fixed_depth = self.fixed_depth;

                self.position.think();
                self.turn += 1;

                let (hash_from, hash_to) = if let (Some(from), Some(to)) =
                    (self.position.hash_from, self.position.hash_to)
                {
                    (from, to)
                } else {
                    // No legal moves
                    println!("(no legal moves)");
                    self.computer_side = None;
                    self.display_board();
                    self.position.generate_moves(self.position.side);
                    continue;
                };

                println!(" collisions {} ", self.position.board.hash.collisions);
                println!();
                self.position.board.hash.collisions = 0;

                println!(
                    "Computer's move: {}",
                    Self::move_string(hash_from, hash_to, None)
                );
                println!();

                self.position.make_move(hash_from, hash_to);
                self.position.set_material();

                let elapsed_time = get_time() - self.position.start_time;
                println!("\nTime: {} ms", elapsed_time);

                let nps = if elapsed_time > 0 {
                    (self.position.nodes as f64 / elapsed_time as f64) * 1000.0
                } else {
                    0.0
                };
                println!("\nNodes per second: {}", nps as u64);

                self.position.ply = 0;
                self.position.first_move[0] = 0;
                self.position.generate_moves(self.position.side);
                self.print_result();
                print!(" turn {}", self.turn);
                self.display_board();
                continue;
            }

            // Human's turn
            print!("Enter move or command> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => return, // EOF
                Ok(_) => {}
                Err(_) => return,
            }

            let command = input.trim();

            match command {
                "d" => {
                    self.display_board();
                    continue;
                }
                "f" => {
                    self.flip = !self.flip;
                    self.display_board();
                    continue;
                }
                "go" => {
                    self.computer_side = Some(self.position.side);
                    continue;
                }
                "help" => {
                    self.show_help();
                    continue;
                }
                "moves" => {
                    println!("Moves");
                    self.position.ply = 0;
                    self.position.first_move[0] = 0;
                    self.position.generate_moves(self.position.side);

                    let move_count = self.position.first_move[1];
                    for i in 0..move_count as usize {
                        if let Some(mv) = self.position.move_list[i] {
                            println!("{}", Self::move_string(mv.from, mv.to, mv.promote));
                        }
                    }
                    continue;
                }
                "new" => {
                    self.new_game();
                    self.computer_side = None;
                    continue;
                }
                "on" | "p" => {
                    self.computer_side = Some(self.position.side);
                    continue;
                }
                "off" => {
                    self.computer_side = None;
                    continue;
                }
                "quit" => {
                    println!("Program exiting");
                    break;
                }
                "sw" => {
                    self.position.side = self.position.side.opponent();
                    self.position.other_side = self.position.other_side.opponent();
                    continue;
                }
                "undo" => {
                    if self.position.ply_from_start_of_game == 0 {
                        continue;
                    }
                    self.computer_side = None;
                    self.position.take_back();
                    self.position.ply = 0;
                    if self.position.first_move[0] != 0 {
                        self.position.first_move[0] = 0;
                    }
                    self.position.generate_moves(self.position.side);
                    continue;
                }
                "xboard" => {
                    self.xboard();
                    break;
                }
                _ => {
                    // Handle move input or commands with parameters
                    if command.starts_with("sb ") {
                        let filename = &command[3..];
                        let full_path = format!("c:\\bscp\\{}.fen", filename);
                        match self.load_diagram(&full_path, 1) {
                            Ok(_) => {}
                            Err(e) => println!("Error loading diagram: {}", e),
                        }
                        continue;
                    }

                    if command.starts_with("sd ") {
                        if let Ok(depth) = command[3..].parse::<u16>() {
                            self.max_depth = depth;
                            self.max_time = 1 << 25;
                            self.fixed_depth = true;
                        }
                        continue;
                    }

                    if command.starts_with("st ") {
                        if let Ok(time) = command[3..].parse::<u32>() {
                            self.max_time = time * 1000;
                            self.max_depth = 64;
                            self.fixed_time = true;
                        }
                        continue;
                    }

                    // Try to parse as a move
                    self.position.ply = 0;
                    self.position.first_move[0] = 0;
                    self.position.generate_moves(self.position.side);

                    if let Some(move_idx) = self.parse_move(command) {
                        if let Some(mv) = self.position.move_list[move_idx] {
                            if !self
                                .position
                                .make_move_with_promotion(mv.from, mv.to, mv.promote)
                            {
                                println!("Illegal move.");
                                println!("{}", command);
                                continue;
                            }

                            // Note: Promotion is now handled in make_move_with_promotion
                            // The old manual promotion code has been removed
                        }
                    } else {
                        println!("Illegal move.");
                        println!("{}", command);
                    }
                }
            }
        }
    }

    fn xboard(&mut self) {
        println!();
        self.new_game();
        self.fixed_time = false;
        self.computer_side = None;

        loop {
            io::stdout().flush().unwrap();

            if Some(self.position.side) == self.computer_side {
                self.position.think();
                self.position.set_material();
                self.position.generate_moves(self.position.side);

                let (hash_from, hash_to) = if let (Some(from), Some(to)) =
                    (self.position.hash_from, self.position.hash_to)
                {
                    (from, to)
                } else {
                    println!(" lookup=0 ");
                    continue;
                };

                if let Some(ref mut mv) = self.position.move_list[0] {
                    mv.from = hash_from;
                    mv.to = hash_to;
                }

                println!("move {}", Self::move_string(hash_from, hash_to, None));
                self.position.make_move(hash_from, hash_to);
                self.position.ply = 0;
                self.position.generate_moves(self.position.side);
                self.print_result();
                continue;
            }

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => return,
                Ok(_) => {}
                Err(_) => return,
            }

            let line = input.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let command = parts[0];

            match command {
                "xboard" => continue,
                "new" => {
                    self.new_game();
                    self.computer_side = Some(Side::Black);
                    continue;
                }
                "quit" => return,
                "force" => {
                    self.computer_side = None;
                    continue;
                }
                "white" => {
                    self.position.side = Side::White;
                    self.position.other_side = Side::Black;
                    self.position.generate_moves(self.position.side);
                    self.computer_side = Some(Side::Black);
                    continue;
                }
                "black" => {
                    self.position.side = Side::Black;
                    self.position.other_side = Side::White;
                    self.position.generate_moves(self.position.side);
                    self.computer_side = Some(Side::White);
                    continue;
                }
                "st" => {
                    if parts.len() > 1 {
                        if let Ok(time) = parts[1].parse::<u32>() {
                            self.max_time = time * 1000;
                            self.max_depth = 64;
                            self.fixed_time = true;
                        }
                    }
                    continue;
                }
                "sd" => {
                    if parts.len() > 1 {
                        if let Ok(depth) = parts[1].parse::<u16>() {
                            self.max_depth = depth;
                            self.max_time = 1 << 25;
                        }
                    }
                    continue;
                }
                "time" => {
                    if parts.len() > 1 {
                        if let Ok(time) = parts[1].parse::<u32>() {
                            self.max_time = if time < 200 {
                                self.max_depth = 1;
                                time
                            } else {
                                self.max_depth = 64;
                                time / 2
                            };
                        }
                    }
                    continue;
                }
                "otim" | "random" | "level" | "hard" | "easy" => continue,
                "go" => {
                    self.computer_side = Some(self.position.side);
                    continue;
                }
                "hint" => {
                    self.position.think();
                    if let (Some(from), Some(to)) = (self.position.hash_from, self.position.hash_to)
                    {
                        println!("Hint: {}", Self::move_string(from, to, None));
                    }
                    continue;
                }
                "undo" => {
                    if self.position.ply_from_start_of_game == 0 {
                        continue;
                    }
                    self.position.take_back();
                    self.position.ply = 0;
                    self.position.generate_moves(self.position.side);
                    continue;
                }
                "remove" => {
                    if self.position.ply_from_start_of_game < 2 {
                        continue;
                    }
                    self.position.take_back();
                    self.position.take_back();
                    self.position.ply = 0;
                    self.position.generate_moves(self.position.side);
                    continue;
                }
                "post" => {
                    // Post mode on (not implemented)
                    continue;
                }
                "nopost" => {
                    // Post mode off (not implemented)
                    continue;
                }
                _ => {
                    // Try to parse as move
                    self.position.first_move[0] = 0;
                    self.position.generate_moves(self.position.side);

                    if let Some(move_idx) = self.parse_move(line) {
                        if let Some(mv) = self.position.move_list[move_idx] {
                            if !self
                                .position
                                .make_move_with_promotion(mv.from, mv.to, mv.promote)
                            {
                                println!("Error (unknown command): {}", command);
                            } else {
                                self.position.ply = 0;
                                self.position.generate_moves(self.position.side);
                                self.print_result();
                            }
                        }
                    } else {
                        println!("Error (unknown command): {}", command);
                    }
                }
            }
        }
    }
}

fn main() {
    println!("Bills Bitboard Chess Engine");
    println!("Version 1.0, 15/1/20");
    println!("Bill Jordan 2020");
    println!();
    println!("\"help\" displays a list of commands.");
    println!();

    let mut engine = ChessEngine::new();
    engine.run_main_loop();
}
