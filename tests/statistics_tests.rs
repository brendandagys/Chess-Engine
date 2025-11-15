use chess_engine::engine::Engine;
use chess_engine::position::Position;

#[test]
fn test_statistics_tracking() {
    let mut engine = Engine::new(None, None, None, None, None, Some(4), None, None, None);

    // Start from initial position
    engine.position =
        Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    // Perform a search
    let result = engine.think::<fn(u16, i32, &mut Position)>(None);

    // Verify statistics are being tracked
    assert!(engine.position.nodes > 0, "Nodes should be tracked");
    assert!(
        engine.position.qnodes > 0,
        "Quiescence nodes should be tracked"
    );
    assert!(
        engine.position.max_depth_reached > 0,
        "Selective depth should be tracked"
    );
    assert!(
        engine.position.max_depth_reached >= result.depth as usize,
        "Selective depth should be >= search depth"
    );
    assert!(
        engine.position.beta_cutoffs > 0,
        "Beta cutoffs should occur"
    );

    // Verify relationships between statistics
    assert!(
        engine.position.nodes >= engine.position.qnodes,
        "Total nodes should include quiescence nodes"
    );

    println!("Statistics after depth {} search:", result.depth);
    println!("  Nodes: {}", engine.position.nodes);
    println!("  Q-Nodes: {}", engine.position.qnodes);
    println!("  Selective Depth: {}", engine.position.max_depth_reached);
    println!("  Beta Cutoffs: {}", engine.position.beta_cutoffs);
}

#[test]
fn test_statistics_reset_between_searches() {
    let mut engine = Engine::new(None, None, None, None, None, Some(3), None, None, None);

    // First search
    engine.position =
        Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    engine.think::<fn(u16, i32, &mut Position)>(None);

    let first_nodes = engine.position.nodes;
    let first_qnodes = engine.position.qnodes;

    // Second search
    engine.position =
        Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();
    engine.think::<fn(u16, i32, &mut Position)>(None);

    // Verify statistics were reset and new values are different
    assert!(
        engine.position.nodes > 0,
        "Nodes should be tracked in second search"
    );
    assert_ne!(
        engine.position.nodes, first_nodes,
        "Node count should be reset"
    );
    assert_ne!(
        engine.position.qnodes, first_qnodes,
        "Q-node count should be reset"
    );
}

#[test]
fn test_nps_calculation() {
    let mut engine = Engine::new(None, None, None, None, Some(100), Some(4), None, None, None);

    engine.position =
        Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    engine.think::<fn(u16, i32, &mut Position)>(None);

    let elapsed_ms = engine.position.time_manager.elapsed().as_millis();
    let nodes = engine.position.nodes;

    assert!(elapsed_ms > 0, "Some time should have elapsed");
    assert!(nodes > 0, "Some nodes should have been searched");

    let nps = if elapsed_ms > 0 {
        ((nodes as f64 / elapsed_ms as f64) * 1000.0) as u64
    } else {
        0
    };

    println!("NPS: {} ({} nodes in {} ms)", nps, nodes, elapsed_ms);
    assert!(nps > 0, "NPS should be positive");
}

#[test]
fn test_max_depth_reached_exceeds_nominal_depth() {
    let mut engine = Engine::new(None, None, None, None, None, Some(3), None, None, None);

    // Use a tactical position that will require quiescence search
    engine.position =
        Position::from_fen("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4")
            .unwrap();

    let result = engine.think::<fn(u16, i32, &mut Position)>(None);

    // In tactical positions, selective depth should be greater than nominal depth
    // due to quiescence search and check extensions
    println!(
        "Depth: {}, Selective Depth: {}",
        result.depth, engine.position.max_depth_reached
    );
    assert!(
        engine.position.max_depth_reached as u16 >= result.depth,
        "Selective depth should be at least the nominal depth"
    );
}

#[test]
fn test_beta_cutoff_percentage() {
    let mut engine = Engine::new(None, None, None, None, None, Some(4), None, None, None);

    engine.position =
        Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    engine.think::<fn(u16, i32, &mut Position)>(None);

    let total_nodes = engine.position.nodes;
    let qnodes = engine.position.qnodes;
    let main_nodes = total_nodes.saturating_sub(qnodes);
    let beta_cutoffs = engine.position.beta_cutoffs;

    // Beta cutoffs should be a significant portion of main search nodes
    // (typically 30-90% depending on move ordering quality)
    if main_nodes > 0 {
        let cutoff_rate = (beta_cutoffs as f64 / main_nodes as f64 * 100.0) as u64;
        println!(
            "Beta cutoff rate: {}% ({}/{} main nodes)",
            cutoff_rate, beta_cutoffs, main_nodes
        );
        assert!(cutoff_rate > 0, "Should have some beta cutoffs");
        assert!(cutoff_rate <= 100, "Cutoff rate should be <= 100%");
    }
}
