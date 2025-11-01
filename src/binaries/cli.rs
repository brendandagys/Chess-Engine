use chess_engine::engine::Engine;
use chess_engine::position::Position;
use chess_engine::types::{GameResult, Side};
use rand::Rng;
use std::io::{self, Write};

fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let len = s.len();

    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }

    result
}

struct CLI {
    engine: Engine,
    display_enabled: bool,
    flip: bool,
}

impl CLI {
    fn new() -> Self {
        let engine = Engine::default();

        Self {
            engine,
            display_enabled: true,
            flip: false,
        }
    }

    fn show_help(&self) {
        println!("\n======================= INFORMATION ======================");
        println!("h or help - Displays help on the commands");
        println!("d or dd   - Displays board and toggles display setting");
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
        println!("fen <FEN>    - Loads a FEN string");
        println!("sd <depth>   - Sets the maximum search depth");
        println!("st <seconds> - Sets the time limit per move in seconds");
    }

    fn display_board(&self) {
        if self.display_enabled {
            self.engine.position.display_board(self.flip);
        }
    }

    fn print_result(&mut self, result: GameResult) {
        match result {
            GameResult::InProgress => {}
            GameResult::Checkmate(winner) => {
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
            println!("\n-------------------------------");
            println!(
                "*   Ply: {} | To move: {:?}   *",
                self.engine.position.ply_from_start_of_game, self.engine.position.side
            );
            println!("-------------------------------");

            // Computer's turn
            if self.engine.computer_side == Some(self.engine.position.side) {
                println!("\nComputer is thinking...");
                println!("\n┌──────┬──────────────┬──────────┬────────────────────┐");
                println!("│ DEPTH│    NODES     │  SCORE   │     BEST MOVE      │");
                println!("├──────┼──────────────┼──────────┼────────────────────┤");

                let has_legal_moves = self.make_computer_move();

                let game_result = self.engine.position.check_game_result();

                if has_legal_moves {
                    self.print_result(game_result);
                } else {
                    println!("(No legal moves)");
                    self.engine.computer_side = None;
                }

                self.display_board();

                continue;
            }

            print!("\nFrom square OR command > ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => return, // EOF
                Ok(_) => {}
                Err(_) => return,
            }

            let command = input.trim().to_lowercase();

            // COMMANDS WITHOUT PARAMETERS
            match command.as_str() {
                "d" => {
                    self.engine.position.display_board(self.flip);
                    continue;
                }
                "dd" => {
                    self.display_enabled = !self.display_enabled;

                    if self.display_enabled {
                        println!("\nBoard display enabled");
                        self.display_board();
                    } else {
                        println!("\nBoard display disabled");
                    }
                    continue;
                }
                "f" => {
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
                "q" | "quit" => {
                    println!("\nProgram exiting");
                    break;
                }
                "switch" => {
                    self.engine.position.side = self.engine.position.side.opponent();
                    continue;
                }
                "undo" => {
                    if self.engine.position.ply_from_start_of_game == 0 {
                        println!("\nNo moves to undo");
                        continue;
                    }
                    self.engine.computer_side = None;
                    self.engine.position.take_back_move();
                    self.engine.generate_moves();
                    self.display_board();
                    continue;
                }
                _ => {}
            }

            // COMMANDS WITH PARAMETERS
            if command.starts_with("fen ") {
                let fen_str = &command[4..];
                match Position::from_fen(fen_str) {
                    Ok(position) => {
                        self.engine.position = position;
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
                    if !self.engine.position.make_move(mv.from, mv.to, mv.promote) {
                        println!("\nILLEGAL MOVE!");
                        continue;
                    }

                    self.engine.generate_moves();
                    let game_result = self.engine.position.check_game_result();
                    self.print_result(game_result);
                    self.display_board();
                } else {
                    panic!("Move found in move list, but is `None`");
                }
            } else {
                println!("\nILLEGAL MOVE!");
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

    fn make_computer_move(&mut self) -> bool {
        self.engine
            .think(Some(|depth, score, position: &mut Position| {
                print!(
                    "│ {:>4} │ {:>12} │ {:>8} │ ",
                    depth,
                    format_with_commas((*position).nodes as u64),
                    score
                );

                // Display best move
                if let (Some(from), Some(to)) =
                    ((*position).best_move_from, (*position).best_move_to)
                {
                    print!("{:^18} ", Engine::move_to_uci_string(from, to, None, true));
                } else {
                    print!("{:^18} ", "");
                }

                println!("│");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }));

        println!("└──────┴──────────────┴──────────┴────────────────────┘");

        let (hash_from, hash_to) = if let (Some(from), Some(to)) =
            (self.engine.position.hash_from, self.engine.position.hash_to)
        {
            (from, to)
        } else {
            return false;
        };

        self.engine.position.make_move(hash_from, hash_to, None);
        self.engine.generate_moves();

        let elapsed_ms = self.engine.position.time_manager.elapsed().as_millis();

        // Calculate statistics
        let nodes_per_second = match elapsed_ms {
            0 => 0, // Avoid division by zero
            ms => ((self.engine.position.nodes as f64 / ms as f64) * 1000.0) as u64,
        };

        let q_nodes = self.engine.position.qnodes;
        let total_nodes = self.engine.position.nodes;
        let main_nodes = total_nodes.saturating_sub(q_nodes);
        let q_percent = if total_nodes > 0 {
            (q_nodes as f64 / total_nodes as f64 * 100.0) as u64
        } else {
            0
        };

        let beta_cutoffs = self.engine.position.beta_cutoffs;
        let cutoff_rate = if main_nodes > 0 {
            (beta_cutoffs as f64 / main_nodes as f64 * 100.0) as u64
        } else {
            0
        };

        // Display comprehensive statistics
        println!("\n┌─────────────────────── SEARCH STATISTICS ───────────────────────┐");
        println!(
            "│ Time:        {:>9} ms  │  Depth:  {:>4}     Quiescence: {:>3}  │",
            format_with_commas(elapsed_ms as u64),
            self.engine.search_settings.depth,
            self.engine.position.seldepth - self.engine.search_settings.depth as usize
        );
        println!(
            "│ Nodes:       {:>12}  │  Qui-Nodes:    {:>12} ({}%)  │",
            format_with_commas(total_nodes as u64),
            format_with_commas(q_nodes as u64),
            q_percent
        );
        println!(
            "│ NPS:         {:>12}  │  β-Cutoffs:    {:>12} ({}%)  │",
            format_with_commas(nodes_per_second),
            format_with_commas(beta_cutoffs as u64),
            cutoff_rate
        );
        println!("└─────────────────────────────────────────────────────────────────┘");

        println!(
            "\nComputer plays: \x1b[32m{}\x1b[0m",
            Engine::move_to_uci_string(hash_from, hash_to, None, true)
        );

        true
    }
}

fn main() {
    println!("\n==============================");
    println!("|   Brendan's Chess Engine   |");
    println!("==============================\n");
    println!("Version 0.1, 2025-11-01");
    println!("\n\"h or help\" displays a list of commands\n");

    let mut cli = CLI::new();
    // cli.show_help();
    cli.run_main_loop();
}
