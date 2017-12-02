extern crate criterion;

use criterion::Criterion;

extern crate indurs;

struct TestData {
    source : &'static [u8],
    target : &'static [u8],
    description : &'static str
}

fn get_test_data(size : &str) -> TestData {
    match size {
        "40k" => TestData {
            source : include_bytes!("data/40k/random/source.bin"),
            target : include_bytes!("data/40k/random/target.bin"),
            description : "40K random data"
        },
        "20k" => TestData {
            source : include_bytes!("data/20k/random/source.bin"),
            target : include_bytes!("data/20k/random/target.bin"),
            description : "20K random data"
        },
        "10k" => TestData {
            source : include_bytes!("data/10k/random/source.bin"),
            target : include_bytes!("data/10k/random/target.bin"),
            description : "10K random data"
        },
        "5k" => TestData {
            source : include_bytes!("data/5k/random/source.bin"),
            target : include_bytes!("data/5k/random/target.bin"),
            description : "5K random data"
        },
        "2k" => TestData {
            source : include_bytes!("data/2k/random/source.bin"),
            target : include_bytes!("data/2k/random/target.bin"),
            description : "2K random data"
        },
        "1k" => TestData {
            source : include_bytes!("data/1k/random/source.bin"),
            target : include_bytes!("data/1k/random/target.bin"),
            description : "1K random data"
        },
        _ => unimplemented!("Missing test data")
    }
}

impl std::fmt::Display for TestData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.description)
    }
}

#[test]
fn criterion_benchmark() {
    Criterion::default()
        .bench_function_over_inputs("delta compression - random data", |b, &data| {
                let mut state = indurs::State::<&[u8]>::default();
                b.iter(|| {
                    state.process_source(data.source);
                    state.encode(data.target)
                });
            },
            &[get_test_data("40k"),
                get_test_data("20k"),
                get_test_data("10k"),
                get_test_data("5k"),
                get_test_data("2k"),
                get_test_data("1k")]);
}
