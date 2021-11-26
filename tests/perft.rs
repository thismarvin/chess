use chess;

// Learn more about perft here:
// https://www.chessprogramming.org/Perft_Results

#[test]
#[ignore]
fn test_engine_perft_position_1() -> Result<(), chess::ChessError> {
    let fen = chess::Fen::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")?;
    let mut state = chess::State::from(fen);

    let total_moves = chess::Engine::perft(&mut state, 5);

    assert_eq!(total_moves, 4_865_609);

    Ok(())
}

#[test]
#[ignore]
fn test_engine_perft_position_2() -> Result<(), chess::ChessError> {
    let fen = chess::Fen::try_from(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    )?;
    let mut state = chess::State::from(fen);

    let total_moves = chess::Engine::perft(&mut state, 5);
    assert_eq!(total_moves, 193_690_690);

    Ok(())
}

#[test]
#[ignore]
fn test_engine_perft_position_3() -> Result<(), chess::ChessError> {
    let fen = chess::Fen::try_from("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1")?;
    let mut state = chess::State::from(fen);

    let total_moves = chess::Engine::perft(&mut state, 5);

    assert_eq!(total_moves, 674_624);

    Ok(())
}

#[test]
#[ignore]
fn test_engine_perft_position_4() -> Result<(), chess::ChessError> {
    let fen =
        chess::Fen::try_from("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")?;
    let mut state = chess::State::from(fen);

    let total_moves = chess::Engine::perft(&mut state, 5);

    assert_eq!(total_moves, 15_833_292);

    Ok(())
}

#[test]
#[ignore]
fn test_engine_perft_position_5() -> Result<(), chess::ChessError> {
    let fen = chess::Fen::try_from("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")?;
    let mut state = chess::State::from(fen);

    let total_moves = chess::Engine::perft(&mut state, 5);

    assert_eq!(total_moves, 89_941_194);

    Ok(())
}

#[test]
#[ignore]
fn test_engine_perft_position_6() -> Result<(), chess::ChessError> {
    let fen = chess::Fen::try_from(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    )?;
    let mut state = chess::State::from(fen);

    let total_moves = chess::Engine::perft(&mut state, 5);

    assert_eq!(total_moves, 164_075_551);

    Ok(())
}
