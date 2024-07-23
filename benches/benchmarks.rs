use criterion::{criterion_group, criterion_main, Criterion};

use fastcmpstr::Str;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

const PREFIX: &str =
    "this_is_an_example_string_thats_used_to_make_the_first_part_of_the_string_predictable";

fn rand_str(prefix_len: usize, rand_len: usize) -> String {
    let r: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(rand_len)
        .map(char::from)
        .collect();

    return format!("{}{}", &PREFIX[..prefix_len], r);
}

fn bench_eq(c: &mut Criterion) {
    for i in [0, 8, 16, 64] {
        let std_strs: Vec<String> = (0..1000).map(|_| rand_str(i, 64 - i)).collect();
        let strs: Vec<Str> = std_strs.iter().map(|s| Str::from(&s)).collect();

        let mut group = c.benchmark_group(&format!("Equality_{}_static_{}_rand", i, 64 - i));
        group.bench_function("fastcmpstr::Str", |b| {
            b.iter(|| {
                let mut x = 0;
                for a in &strs {
                    for b in &strs {
                        x += (a == b) as i32;
                    }
                }
                return x;
            })
        });

        group.bench_function("std::string::String", |b| {
            b.iter(|| {
                let mut x = 0;
                for a in &std_strs {
                    for b in &std_strs {
                        x += (a == b) as i32;
                    }
                }
                return x;
            })
        });
    }
}

fn bench_contains(c: &mut Criterion) {
    for i in [0, 8, 16, 32] {
        let std_strs: Vec<String> = (0..10000).map(|_| rand_str(i, 64 - i)).collect();
        let strs: Vec<Str> = std_strs.iter().map(|s| Str::from(&s)).collect();

        let mut group = c.benchmark_group(&format!("Contains_10k_{}_static_{}_rand", i, 64 - i));
        group.bench_function("fastcmpstr::Str", |b| {
            b.iter(|| {
                let mut x = 0;
                for a in &strs {
                    x += strs.contains(a) as i32;
                }
                return x;
            })
        });

        group.bench_function("std::string::String", |b| {
            b.iter(|| {
                let mut x = 0;
                for a in &std_strs {
                    x += std_strs.contains(a) as i32;
                }
                return x;
            })
        });
    }
}

criterion_group!(benches, bench_eq, bench_contains);
criterion_main!(benches);
