# Chess Engine

A high-performance chess engine written in Rust with UCI protocol support and a CLI interface.

## Features

- **UCI Protocol Support**: Compatible with UCI-compatible chess GUIs (Arena, Cute Chess, etc.)
- **Advanced Search Algorithm**:
  - Negamax with alpha-beta pruning
  - Quiescence search for tactical stability
  - Principal variation search (PVS)
  - History heuristic for move ordering
- **Opening Book**: Polyglot opening book support for strong opening play
- **Difficulty Levels**: Multiple skill levels from beginner to expert
- **Time Management**: Smart time allocation for timed games
- **Interactive CLI**: Play against the engine directly in your terminal

## Installation

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
chess-engine = "0.1.2"
```

### Install Binaries

Install the command-line interface:

```bash
cargo install chess-engine --bin chess-engine-cli
```

Install the UCI interface:

```bash
cargo install chess-engine --bin chess-engine-uci
```

Or, install both:

```bash
cargo install chess-engine --bins
```

## Usage

### CLI Mode

Play interactively against the engine:

```bash
chess-engine-cli
```

The CLI provides:

- Visual board display
- Move validation
- Configurable difficulty levels
- Game statistics (nodes searched, time, evaluation)
- FEN import/export

### UCI Mode

Use with UCI-compatible chess GUIs:

```bash
chess-engine-uci
```

Or integrate with tools like `cutechess-cli`:

```bash
cutechess-cli -engine cmd=chess-engine-uci -engine cmd=stockfish
```

### Library Usage

```rust
use chess_engine::engine::Engine;
use chess_engine::types::Difficulty;

fn main() {
    // Create a new engine instance
    let mut engine = Engine::new(
        None,           // White time (ms)
        None,           // Black time (ms)
        None,           // White increment (ms)
        None,           // Black increment (ms)
        Some(5000),     // Move time: 5 seconds
        None,           // Max depth
        None,           // Max nodes
        None,           // Opening book path
        Some(Difficulty::Medium),
    );

    // Search for the best move
    let result = engine.think(None::<fn(u16, i32, &mut Position)>);

    println!("Best move: {:?} -> {:?}",
        result.best_move_from,
        result.best_move_to
    );
    println!("Evaluation: {} centipawns", result.evaluation);
    println!("Searched {} nodes in {} ms", result.nodes, result.time_ms);
}
```

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/brendandagys/Chess-Engine.git
cd Chess-Engine

# Build in release mode
cargo build --release

# Binaries will be in target/release/
./target/release/chess-engine-cli
./target/release/chess-engine-uci
```

### Running Tests

The project includes comprehensive test suites:

```bash
# Run all tests with proper stack size
make test

# Or directly with cargo
RUST_MIN_STACK=33554432 cargo test --release
```

Test coverage includes:

- Move generation and validation
- Position evaluation
- Search algorithm correctness
- Hash table operations
- Draw detection
- Principal variation tracking
- FEN parsing and generation
- Perft (performance testing)

### Project Structure

```
src/
├── lib.rs          # Library root
├── engine.rs       # Main engine logic and search
├── position.rs     # Board representation and move generation
├── hash.rs         # Transposition table with Zobrist hashing
├── types.rs        # Core data structures
├── constants.rs    # Game constants and piece values
├── uci.rs          # UCI protocol implementation
├── time.rs         # Time management
├── polyglot.rs     # Opening book support
└── bin/
    ├── cli.rs      # Interactive CLI
    └── uci.rs      # UCI binary entry point

tests/              # Comprehensive test suite
opening_books/      # Polyglot opening books
```

## Performance

- **Search Speed**: ~6 million nodes/second (depends on position complexity)

## Difficulty Levels

The engine supports multiple difficulty settings:

- **Beginner**: Depth 1, suitable for beginners
- **Easy**: Depth 2, easy play
- **Medium**: Depth 3, intermediate play
- **Hard**: Depth 4, strong tactical play
- **Expert**: Depth 5, very strong tactical play
- **Master**: Depth 6, extremely strong tactical play

## UCI Commands Supported

- `uci` - Identify engine
- `isready` - Check readiness
- `ucinewgame` - Start new game
- `position [fen <fenstring> | startpos] moves <move1> ... <movei>` - Set position
- `go [wtime <x> btime <x> winc <x> binc <x> | movetime <x> | depth <x>]` - Start searching
- `stop` - Stop searching
- `quit` - Exit engine

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

## Author

**Brendan Dagys** - [brendandagys@gmail.com](mailto:brendandagys@gmail.com)

## Repository

[https://github.com/brendandagys/Chess-Engine](https://github.com/brendandagys/Chess-Engine)

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.
