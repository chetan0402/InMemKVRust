use std::{io::Write, net::TcpStream};

use criterion::{Criterion, criterion_group, criterion_main};

fn criterion_benchmark(c: &mut Criterion) {
    let mut conn = TcpStream::connect("127.0.0.1:9876").unwrap();
    let command = "SET a b".as_bytes();
    c.bench_function("set", |b| b.iter(|| conn.write_all(command).unwrap()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
