use chess_engine::engine::Engine;
use chess_engine::uci;

fn main() {
    let mut engine = Engine::default();
    uci::uci_loop(&mut engine);
}
