use chess::{fen_reader, perft};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/*
pub fn initial_position_5(c: &mut Criterion) {

    let mut group = c.benchmark_group("nodes generarion");
    group.sample_size(25);
    group.bench_function("initial position depth 5", |b| b.iter(|| perft::perft(5,black_box(fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")))));

}
*/
pub fn test_positions(c: &mut Criterion) {
    let mut group = c.benchmark_group("nodes generarion");

    group.bench_function(
        "position r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        |b| {
            b.iter(|| {
                perft::perft(
                    4,
                    black_box(&mut fen_reader::read_fen(
                        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                    )),
                )
            })
        },
    );

    group.bench_function(
        "initial position depth 5",
        |b| {
            b.iter(|| {
                perft::perft(
                    6,
                    black_box(&mut fen_reader::read_fen(
                        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ",
                    )),
                )
            })
        },
    );
}

criterion_group!(benches, test_positions);
criterion_main!(benches);
