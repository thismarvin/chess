use chess;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn kiwipete(depth: u8) -> Result<u128, chess::ChessError> {
    let fen = chess::Fen::try_from(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    )?;
    let mut state = chess::State::from(fen);

    chess::Engine::perft(&mut state, depth)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("perft_kiwipete", |b| b.iter(|| kiwipete(black_box(3))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
