use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use indurs;

struct TestData {
    source: &'static [u8],
    target: &'static [u8],
    description: &'static str,
}

impl std::fmt::Debug for TestData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description)
    }
}

const TEST_DATA: [TestData; 6] = [
    TestData {
        source: include_bytes!("data/40k/random/source.bin"),
        target: include_bytes!("data/40k/random/target.bin"),
        description: "40K random data",
    },
    TestData {
        source: include_bytes!("data/20k/random/source.bin"),
        target: include_bytes!("data/20k/random/target.bin"),
        description: "20K random data",
    },
    TestData {
        source: include_bytes!("data/10k/random/source.bin"),
        target: include_bytes!("data/10k/random/target.bin"),
        description: "10K random data",
    },
    TestData {
        source: include_bytes!("data/5k/random/source.bin"),
        target: include_bytes!("data/5k/random/target.bin"),
        description: "5K random data",
    },
    TestData {
        source: include_bytes!("data/2k/random/source.bin"),
        target: include_bytes!("data/2k/random/target.bin"),
        description: "2K random data",
    },
    TestData {
        source: include_bytes!("data/1k/random/source.bin"),
        target: include_bytes!("data/1k/random/target.bin"),
        description: "1K random data",
    },
];

fn criterion_benchmark(c : &mut Criterion) {
    c.bench_function_over_inputs(
        "delta compression - random data",
        |b, &data| {
            let mut state = indurs::State::<&[u8]>::default();
            b.iter(|| {
                state.process_source(black_box(data.source));
                state.encode(black_box(data.target))
            });
        },
        &TEST_DATA,
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
