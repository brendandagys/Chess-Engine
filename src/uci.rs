use crate::{
    engine::{Engine, SearchResult},
    position::Position,
};
use std::io::{self, Write};

const ENGINE_NAME: &str = "Chess Engine";
const ENGINE_AUTHOR: &str = "Brendan Dagys";

pub fn uci_loop(engine: &mut Engine) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];

        match command {
            "uci" => {
                println!("id name {}", ENGINE_NAME);
                println!("id author {}", ENGINE_AUTHOR);
                println!("uciok");
                stdout.flush().unwrap();
            }
            "isready" => {
                println!("readyok");
                stdout.flush().unwrap();
            }
            "ucinewgame" => {
                engine.new_game();
            }
            "position" => {
                if let Err(e) = parse_position_command(engine, input) {
                    eprintln!("Error parsing position: {}", e);
                }
            }
            "go" => {
                parse_go_command(engine, input);

                let (final_depth, final_score) =
                    engine.think(Some(|depth, score, position: &mut Position| {
                        // Output UCI info line if requested
                        if let (Some(from), Some(to)) =
                            ((*position).best_move_from, (*position).best_move_to)
                        {
                            let time_ms = (*position).time_manager.elapsed().as_millis() as u64;
                            let best_move_uci = Engine::move_to_uci_string(from, to, None, false);
                            println!(
                                "info depth {} score cp {} nodes {} time {} pv {}",
                                depth,
                                score,
                                (*position).nodes,
                                time_ms,
                                best_move_uci
                            );
                        }
                    }));

                let time_ms = engine.position.time_manager.elapsed().as_millis() as u64;

                // Build SearchResult
                let (best_move, ponder_move) = if let (Some(from), Some(to)) =
                    (engine.position.best_move_from, engine.position.best_move_to)
                {
                    (Engine::move_to_uci_string(from, to, None, false), None)
                } else {
                    (String::new(), None)
                };

                // Build PV (principal variation) - for now just the best move
                let pv = if !best_move.is_empty() {
                    vec![best_move.clone()]
                } else {
                    vec![]
                };

                let result = SearchResult {
                    best_move,
                    ponder_move,
                    evaluation: final_score,
                    depth: final_depth,
                    nodes: engine.position.nodes as u64,
                    pv,
                    time_ms,
                };

                println!("bestmove {}", result.best_move);
                stdout.flush().unwrap();
            }
            "stop" => {}
            "quit" => {
                break;
            }
            "d" | "display" => {
                engine.position.display_board(false);
            }
            _ => {
                // Unknown command - silently ignore per UCI spec
            }
        }
    }
}

/// Parse UCI position command
/// Examples:
///   position startpos
///   position startpos moves e2e4 e7e5
///   position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
///   position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves e2e4
pub fn parse_position_command(engine: &mut Engine, command: &str) -> Result<(), String> {
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.len() < 2 {
        return Err("Invalid position command".to_string());
    }

    let mut index = 1;

    // Parse position type (startpos or fen)
    if parts[index] == "startpos" {
        engine
            .position
            .from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .map_err(|e| e.to_string())?;
        index += 1;
    } else if parts[index] == "fen" {
        index += 1;
        if index >= parts.len() {
            return Err("Missing FEN string".to_string());
        }

        let mut fen_parts = Vec::new();
        // FEN has 6 space-separated fields
        for _ in 0..6 {
            if index < parts.len() && parts[index] != "moves" {
                fen_parts.push(parts[index]);
                index += 1;
            }
        }

        let fen = fen_parts.join(" ");
        engine.position.from_fen(&fen).map_err(|e| e.to_string())?;
    } else {
        return Err(format!("Unknown position type: {}", parts[index]));
    }

    // Parse moves if present
    if index < parts.len() && parts[index] == "moves" {
        index += 1;

        while index < parts.len() {
            let move_str = parts[index];

            let (from, to, promote) = Engine::move_from_uci_string(move_str)?;

            engine
                .position
                .generate_moves_and_captures(engine.position.side);

            // Find the move in the legal move list
            let mut found = false;
            for i in
                engine.position.ply..engine.position.first_move[1 + engine.position.ply] as usize
            {
                if let Some(mv) = engine.position.move_list[i] {
                    if mv.from == from && mv.to == to && mv.promote == promote {
                        // Make the move
                        if !engine.position.make_move(from, to) {
                            return Err(format!("Illegal move: {}", move_str));
                        }
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                return Err(format!("Move not found in legal moves: {}", move_str));
            }

            index += 1;
        }
    }

    Ok(())
}

/// Parse UCI go command and update search settings
/// Examples:
///   go depth 10
///   go movetime 5000
///   go wtime 300000 btime 300000 winc 0 binc 0
///   go infinite
pub fn parse_go_command(engine: &mut Engine, command: &str) {
    let parts: Vec<&str> = command.split_whitespace().collect();

    let mut wtime = None;
    let mut btime = None;
    let mut winc = None;
    let mut binc = None;
    let mut movetime = None;
    // let mut depth = None;

    let mut i = 1; // Skip "go"
    while i < parts.len() {
        match parts[i] {
            "wtime" => {
                if i + 1 < parts.len() {
                    wtime = parts[i + 1].parse::<u64>().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "btime" => {
                if i + 1 < parts.len() {
                    btime = parts[i + 1].parse::<u64>().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "winc" => {
                if i + 1 < parts.len() {
                    winc = parts[i + 1].parse::<u64>().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "binc" => {
                if i + 1 < parts.len() {
                    binc = parts[i + 1].parse::<u64>().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "movetime" => {
                if i + 1 < parts.len() {
                    movetime = parts[i + 1].parse::<u64>().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            // TODO: Implement fixed depth again?
            //
            // "depth" => {
            //     if i + 1 < parts.len() {
            //         depth = parts[i + 1].parse::<u16>().ok();
            //         i += 2;
            //     } else {
            //         i += 1;
            //     }
            // }
            // "infinite" => {
            //     depth = Some(100);
            //     i += 1;
            // }
            _ => {
                i += 1;
            }
        }
    }

    // Update search settings
    if let Some(wt) = wtime {
        engine.search_settings.wtime = wt;
    }
    if let Some(bt) = btime {
        engine.search_settings.btime = bt;
    }
    if let Some(wi) = winc {
        engine.search_settings.winc = wi;
    }
    if let Some(bi) = binc {
        engine.search_settings.binc = bi;
    }
    if let Some(mt) = movetime {
        engine.search_settings.movetime = Some(mt);
    }
    // if let Some(d) = depth {
    //     engine.search_settings.depth = d;
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_from_uci_string() {
        let result = Engine::move_from_uci_string("e2e4");
        assert!(result.is_ok());
        let (from, to, promote) = result.unwrap();
        assert_eq!(from as usize, 12); // e2
        assert_eq!(to as usize, 28); // e4
        assert_eq!(promote, None);
    }

    #[test]
    fn test_move_from_uci_string_promotion() {
        let result = Engine::move_from_uci_string("e7e8q");
        assert!(result.is_ok());
        let (from, to, promote) = result.unwrap();
        assert_eq!(from as usize, 52); // e7
        assert_eq!(to as usize, 60); // e8
        assert_eq!(promote, Some(crate::types::Piece::Queen));
    }

    #[test]
    fn test_move_to_uci() {
        use crate::types::Square;
        let from = Square::try_from(12u8).unwrap(); // e2
        let to = Square::try_from(28u8).unwrap(); // e4
        let uci = Engine::move_to_uci_string(from, to, None, false);
        assert_eq!(uci, "e2e4");
    }

    #[test]
    fn test_startpos_position() {
        let mut engine = Engine::default();
        let result = parse_position_command(&mut engine, "position startpos");
        assert!(result.is_ok());
    }

    #[test]
    fn test_position_with_moves() {
        let mut engine = Engine::default();
        let result = parse_position_command(&mut engine, "position startpos moves e2e4 e7e5");
        assert!(result.is_ok());
    }
}
