use chess_engine::engine::Engine;
use chess_engine::position::Position;
use chess_engine::types::{GameResult, Side};
use rand::Rng;
use std::io::{self, Write};

struct CLI {
    engine: Engine,
    flip: bool,
    display_enabled: bool,
}

impl CLI {
    fn new() -> Self {
        let engine = Engine::default();

        Self {
            engine,
            flip: false,
            display_enabled: true,
        }
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
        println!("sd        - Sets the maximum search depth");
        println!("st        - Sets the time limit per move in seconds");
    }

    fn display_board(&self) {
        if self.display_enabled {
            self.engine.position.display_board(self.flip);
        }
    }

    fn print_result(&mut self) {
        self.engine.position.set_material_scores();

        let result = self.engine.position.check_game_result();

        match result {
            GameResult::InProgress => {}
            GameResult::Checkmate(winner) => {
                self.engine
                    .position
                    .generate_moves_and_captures(self.engine.position.side);
                self.display_board();
                println!("\nGAME OVER");

                if winner == Side::White {
                    println!("{{White mates}}");
                } else {
                    println!("{{Black mates}}");
                }

                self.engine.new_game();
            }
            GameResult::Stalemate => {
                println!("{{Stalemate}}");
                self.engine.new_game();
            }
            GameResult::DrawByRepetition => {
                println!("{{Draw by repetition}}");
                self.engine.new_game();
            }
            GameResult::DrawByFiftyMoveRule => {
                println!("{{Draw by fifty move rule}}");
                self.engine.new_game();
            }
            GameResult::DrawByInsufficientMaterial => {
                println!("{{Stalemate}}");
                self.engine.new_game();
            }
        }
    }

    fn run_main_loop(&mut self) {
        self.display_board();

        loop {
            // Display current turn info
            println!("\n-------------------------------");
            println!(
                "*   Ply: {} | To move: {:?}   *",
                self.engine.position.ply_from_start_of_game + 1,
                self.engine.position.side
            );
            println!("-------------------------------");

            // Computer's turn
            if Some(self.engine.position.side) == self.engine.computer_side {
                println!("\nComputer is thinking...");
                println!("\nPLY         NODES     SCORE      PV");

                self.engine
                    .think(Some(|depth, score, position: &mut Position| {
                        print!("{:>3}  {:>12}  {:>8}   ", depth, (*position).nodes, score);

                        // Display best move
                        if let (Some(from), Some(to)) =
                            ((*position).best_move_from, (*position).best_move_to)
                        {
                            print!(" ");
                            Position::display_move(from, to);
                        }

                        println!();
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    }));

                let (hash_from, hash_to) = if let (Some(from), Some(to)) =
                    (self.engine.position.hash_from, self.engine.position.hash_to)
                {
                    (from, to)
                } else {
                    // TODO: What is the purpose of this branch?
                    println!("(No legal moves)");
                    self.engine.computer_side = None;
                    self.display_board();
                    self.engine
                        .position
                        .generate_moves_and_captures(self.engine.position.side);
                    continue;
                };

                self.engine.position.make_move(hash_from, hash_to);
                self.engine.position.set_material_scores();

                let elapsed_ms = self.engine.position.time_manager.elapsed().as_millis();

                print!("\nTime: {} ms", elapsed_ms);

                let nps = match elapsed_ms {
                    0 => 0.0, // Avoid division by zero
                    ms => (self.engine.position.nodes as f64 / ms as f64) * 1000.0,
                };

                print!(" | Nodes/s: {}", nps as u64);

                print!(
                    " | Soft: {:?} - Hard: {:?}\n",
                    self.engine.position.time_manager.soft_limit,
                    self.engine.position.time_manager.hard_limit
                );

                println!(
                    "\nComputer plays: \x1b[32m{}\x1b[0m",
                    Engine::move_to_uci_string(hash_from, hash_to, None, true)
                );

                self.engine.position.ply = 0;
                self.engine.position.first_move[0] = 0;
                self.engine
                    .position
                    .generate_moves_and_captures(self.engine.position.side);

                self.print_result();

                self.display_board();

                continue;
            }

            // Show available moves
            self.engine.position.ply = 0;
            self.engine.position.first_move[0] = 0;
            self.engine
                .position
                .generate_moves_and_captures(self.engine.position.side);

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
                    self.engine.position.display_board(self.flip);
                    continue;
                }
                "D" => {
                    self.display_enabled = !self.display_enabled;

                    if self.display_enabled {
                        println!("\nBoard display enabled");
                        self.display_board();
                    } else {
                        println!("\nBoard display disabled");
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
                    println!("\n{}", self.engine.position.to_fen());
                    continue;
                }
                "moves" => {
                    println!("\nLegal moves:");
                    self.engine.display_legal_moves();
                    continue;
                }
                "new" => {
                    self.engine.new_game();
                    self.display_board();
                    continue;
                }
                "p" | "play" => {
                    self.engine.computer_side = Some(self.engine.position.side);
                    continue;
                }
                "off" => {
                    self.engine.computer_side = None;
                    continue;
                }
                "q" | "Q" | "quit" => {
                    println!("\nProgram exiting");
                    break;
                }
                "switch" => {
                    self.engine.position.side = self.engine.position.side.opponent();
                    self.engine.position.other_side = self.engine.position.other_side.opponent();
                    self.engine
                        .position
                        .generate_moves_and_captures(self.engine.position.side);
                    continue;
                }
                "undo" => {
                    // TODO: Can this be improved? Should set material scores? Why is ply set to 0?
                    if self.engine.position.ply_from_start_of_game == 0 {
                        println!("\nNo moves to undo");
                        continue;
                    }
                    self.engine.computer_side = None;
                    self.engine.position.take_back_move();
                    self.engine.position.ply = 0;
                    self.engine.position.first_move[0] = 0;
                    self.engine
                        .position
                        .generate_moves_and_captures(self.engine.position.side);
                    self.display_board();
                    continue;
                }
                _ => {}
            }

            // COMMANDS WITH PARAMETERS
            if command.starts_with("fen ") {
                let fen_str = &command[4..];
                match self.engine.position.from_fen(fen_str) {
                    Ok(_) => {
                        self.engine.position.set_material_scores();
                        self.engine
                            .position
                            .generate_moves_and_captures(self.engine.position.side);
                        self.display_board();
                        println!("FEN loaded successfully");
                    }
                    Err(e) => println!("Error loading FEN: {}", e),
                }
                continue;
            }

            if command.starts_with("sd ") {
                if let Ok(depth) = command[3..].parse::<u16>() {
                    self.engine.search_settings.depth = depth;
                    println!("\nSearch maximum search depth set to {}", depth);
                }
                continue;
            }

            if command.starts_with("st ") {
                if let Ok(time) = command[3..].parse::<u64>() {
                    let time_in_ms = time * 1000;
                    self.engine.search_settings.movetime = Some(time_in_ms);
                    println!("\nSearch time set to {} seconds", time);
                }
                continue;
            }

            // PARSE "FROM" AND THEN "TO" SQUARE
            if command.len() < 2 {
                println!("\nINVALID COMMAND!");
                continue;
            }

            let from_square = Position::parse_square(command[..2].trim());
            if from_square.is_none() {
                println!("\nINVALID FROM SQUARE!");
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
                    println!("\nINVALID COMMAND!");
                    continue;
                }
            }

            let to_square = Position::parse_square(to_input.trim());
            if to_square.is_none() {
                println!("\nINVALID TO SQUARE!");
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

            if let Some(move_idx) = self.engine.parse_move_string(&move_str) {
                if let Some(mv) = self.engine.position.move_list[move_idx] {
                    if !self
                        .engine
                        .position
                        .make_move_with_promotion(mv.from, mv.to, mv.promote)
                    {
                        println!("ILLEGAL MOVE!");
                        continue;
                    }

                    self.engine.position.set_material_scores();
                    self.engine.position.ply = 0;
                    self.engine.position.first_move[0] = 0;
                    self.engine
                        .position
                        .generate_moves_and_captures(self.engine.position.side);
                    self.print_result();
                    self.display_board();
                } else {
                    panic!("Move found in move list, but is `None`");
                }
            } else {
                println!("ILLEGAL MOVE!");
            }
        }
    }

    fn handle_go_command(&mut self) {
        println!("\nChoose your side:");
        println!("1. White");
        println!("2. Black");
        println!("3. Random");
        print!("\nEnter choice (1-3) > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(_) => return,
        }

        println!();

        let choice = input.trim();

        let player_side = match choice {
            "1" => Side::White,
            "2" => Side::Black,
            "3" => {
                let side = match rand::thread_rng().gen_bool(0.5) {
                    true => Side::White,
                    false => Side::Black,
                };
                println!("You are playing as {:?}", side);
                side
            }
            _ => {
                println!("Invalid choice. Defaulting to White.");
                Side::White
            }
        };

        self.engine.computer_side = Some(player_side.opponent());
    }
}

fn main() {
    println!("Brendan's Chess Engine");
    println!("Version 1.0, 2025-10-07");
    println!();
    println!("\"h or help\" displays a list of commands");
    println!();

    let mut cli = CLI::new();
    cli.show_help();
    cli.run_main_loop();
}
