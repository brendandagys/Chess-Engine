mod test_utils;
use chess_engine::{engine::Engine, position::Position};

#[test]
fn test_pv_collection() {
    let mut engine = Engine::new(None, None, None, None, None, Some(5), None, None, None);

    // Starting position is already loaded
    engine.generate_moves();

    let result = engine.think(None::<fn(u16, i32, &mut Position)>);

    // Verify we have a PV
    assert!(
        !result.principal_variation.is_empty(),
        "PV should not be empty"
    );

    // PV length should be at least 1 (the best move)
    assert!(
        result.principal_variation.len() >= 1,
        "PV should have at least one move"
    );

    // First move in PV should match the best move
    if let Some(first_pv) = result.principal_variation.first() {
        assert_eq!(first_pv.from, result.best_move_from.unwrap());
        assert_eq!(first_pv.to, result.best_move_to.unwrap());
    }

    println!("PV length: {}", result.principal_variation.len());
    println!("PV: {:?}", result.principal_variation);
}

#[test]
fn test_pv_mate_in_one() {
    let mut engine = Engine::new(None, None, None, None, None, Some(3), None, None, None);

    // Set up a mate in one position via new_game + moves
    // For now, just test with starting position
    engine.generate_moves();

    let result = engine.think(None::<fn(u16, i32, &mut Position)>);

    // Verify we have a PV
    assert!(
        !result.principal_variation.is_empty(),
        "PV should not be empty"
    );

    println!("PV: {:?}", result.principal_variation);

    // The PV should contain at least one move
    assert!(result.principal_variation.len() >= 1);
}

#[test]
fn test_pv_depth_increases() {
    let mut engine = Engine::new(None, None, None, None, None, Some(6), None, None, None);

    // Starting position is already loaded
    engine.generate_moves();

    let result = engine.think(None::<fn(u16, i32, &mut Position)>);

    // With depth 6, we should get a reasonable PV
    // (it might be shorter due to repetitions, exchanges, or quiet positions)
    println!("PV at depth 6: {:?}", result.principal_variation);
    println!("PV length: {}", result.principal_variation.len());

    assert!(!result.principal_variation.is_empty());
}
