use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chess::{fen_reader,main};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut game = black_box(fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));

    
    c.bench_function("perft initial position depth 6", |b| b.iter(|| main::perft(6,&mut game)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);