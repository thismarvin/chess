use chess;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn go(depth: u8) -> Result<(), chess::ChessError> {
    let mut engine = chess::Pescado::new(|_| {});

    engine
        .send("position fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    engine.send(format!("go depth {}", depth).as_str());

    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("go_kiwipete", |b| b.iter(|| go(black_box(3))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
