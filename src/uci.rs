use crate::{
    engine::Engine,
    position::Position,
    types::{Board, MoveData},
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

                let result = engine.think(Some(|depth, score, position: &mut Position| {
                    // Output UCI info line if requested
                    if let Some(_) = position
                        .pv_table
                        .get(0)
                        .and_then(|ply| ply.get(0))
                        .and_then(|&m| m)
                    {
                        let time_ms = position.time_manager.elapsed().as_millis() as u64;
                        let nps = if time_ms > 0 {
                            (position.nodes as u64 * 1000) / time_ms
                        } else {
                            0
                        };

                        // Build PV string from pv_table at root (ply 0)
                        let mut pv_string = String::new();
                        for i in 0..position.pv_length[0] {
                            if let Some(mv) = position.pv_table[0][i] {
                                if !pv_string.is_empty() {
                                    pv_string.push(' ');
                                }
                                pv_string.push_str(&Board::move_to_uci_string(
                                    mv.from, mv.to, mv.promote, false,
                                ));
                            }
                        }

                        println!(
                            "info depth {} seldepth {} score cp {} nodes {} nps {} time {} pv {}",
                            depth,
                            position.max_depth_reached,
                            score,
                            position.nodes,
                            nps,
                            time_ms,
                            pv_string
                        );
                    }
                }));

                // Output the best move
                if let (Some(from), Some(to)) = (result.best_move_from, result.best_move_to) {
                    let best_move =
                        Board::move_to_uci_string(from, to, result.best_move_promote, false);
                    println!("bestmove {}", best_move);
                } else {
                    // No legal moves found
                    println!("bestmove 0000");
                }
                stdout.flush().unwrap();
            }
            "stop" => {} // TODO: implement stop functionality
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
        engine.position =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
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
        engine.position = Position::from_fen(&fen).unwrap();
    } else {
        return Err(format!("Unknown position type: {}", parts[index]));
    }

    // Parse moves if present
    if index < parts.len() && parts[index] == "moves" {
        index += 1;

        while index < parts.len() {
            let move_str = parts[index];

            let MoveData { from, to, promote } = Board::move_from_uci_string(move_str)?;

            let legal_moves = engine.position.get_legal_moves();
            let move_uci = Board::move_to_uci_string(from, to, promote, false);

            if !legal_moves.contains(&move_uci) {
                return Err(format!("Illegal move: {}", move_str));
            }

            if !engine.position.make_move(from, to, promote) {
                return Err(format!("Failed to make move: {}", move_str));
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
    let mut max_depth = None;
    let mut max_nodes = None;

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
            "depth" => {
                if i + 1 < parts.len() {
                    max_depth = parts[i + 1].parse::<u16>().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "infinite" => {
                max_depth = Some(100);
                i += 1;
            }
            "nodes" => {
                if i + 1 < parts.len() {
                    max_nodes = parts[i + 1].parse::<usize>().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
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
    if let Some(max_depth) = max_depth {
        engine.search_settings.max_depth = max_depth;
    }
    if let Some(nodes) = max_nodes {
        engine.search_settings.max_nodes = Some(nodes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_from_uci_string() {
        let result = Board::move_from_uci_string("e2e4");
        assert!(result.is_ok());
        let MoveData { from, to, promote } = result.unwrap();
        assert_eq!(from as usize, 12); // e2
        assert_eq!(to as usize, 28); // e4
        assert_eq!(promote, None);
    }

    #[test]
    fn test_move_from_uci_string_promotion() {
        let result = Board::move_from_uci_string("e7e8q");
        assert!(result.is_ok());
        let MoveData { from, to, promote } = result.unwrap();
        assert_eq!(from as usize, 52); // e7
        assert_eq!(to as usize, 60); // e8
        assert_eq!(promote, Some(crate::types::Piece::Queen));
    }

    #[test]
    fn test_move_to_uci() {
        use crate::types::Square;
        let from = Square::try_from(12u8).unwrap(); // e2
        let to = Square::try_from(28u8).unwrap(); // e4
        let uci = Board::move_to_uci_string(from, to, None, false);
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
