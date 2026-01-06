//! Benchmarks for intrinsic protobuf encoding operations.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use piecemeal::{
    Writer,
    helpers::{sizeof_int32, sizeof_int64, sizeof_sint32, sizeof_sint64, sizeof_varint, tag},
    types::WireType,
};

/// Representative values covering all varint byte widths (1-10 bytes).
const VARINT_TEST_VALUES: [(u64, &str); 10] = [
    (0, "1_byte_zero"),
    (127, "1_byte_max"),
    (128, "2_byte_min"),
    (16383, "2_byte_max"),
    (16384, "3_byte_min"),
    (1 << 21, "4_byte"),
    (1 << 28, "5_byte"),
    (1 << 35, "6_byte"),
    (1 << 49, "8_byte"),
    (u64::MAX, "10_byte_max"),
];

fn bench_sizeof_varint(c: &mut Criterion) {
    let mut group = c.benchmark_group("sizeof_varint");

    for (value, name) in VARINT_TEST_VALUES {
        group.bench_with_input(BenchmarkId::new("value", name), &value, |b, &v| {
            b.iter(|| sizeof_varint(black_box(v)))
        });
    }
    group.finish();
}

fn bench_write_varint(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_varint");

    for (value, name) in VARINT_TEST_VALUES {
        group.bench_with_input(BenchmarkId::new("value", name), &value, |b, &v| {
            let mut buf = Vec::with_capacity(10);
            b.iter(|| {
                buf.clear();
                buf.write_varint(black_box(v)).unwrap()
            })
        });
    }
    group.finish();
}

fn bench_sizeof_signed(c: &mut Criterion) {
    let mut group = c.benchmark_group("sizeof_signed");

    // int32: negative values expand to 10 bytes, positive use fewer
    let int32_values: [(i32, &str); 4] = [
        (0, "int32_zero"),
        (127, "int32_small"),
        (-1, "int32_neg_one"),
        (i32::MIN, "int32_min"),
    ];

    for (value, name) in int32_values {
        group.bench_with_input(BenchmarkId::new("sizeof_int32", name), &value, |b, &v| {
            b.iter(|| sizeof_int32(black_box(v)))
        });
    }

    // int64: similar pattern
    let int64_values: [(i64, &str); 4] = [
        (0, "int64_zero"),
        (127, "int64_small"),
        (-1, "int64_neg_one"),
        (i64::MIN, "int64_min"),
    ];

    for (value, name) in int64_values {
        group.bench_with_input(BenchmarkId::new("sizeof_int64", name), &value, |b, &v| {
            b.iter(|| sizeof_int64(black_box(v)))
        });
    }

    // sint32: zigzag encoding makes negative values compact
    let sint32_values: [(i32, &str); 4] = [
        (0, "sint32_zero"),
        (63, "sint32_small"),
        (-1, "sint32_neg_one"),
        (i32::MIN, "sint32_min"),
    ];

    for (value, name) in sint32_values {
        group.bench_with_input(BenchmarkId::new("sizeof_sint32", name), &value, |b, &v| {
            b.iter(|| sizeof_sint32(black_box(v)))
        });
    }

    // sint64: zigzag encoding
    let sint64_values: [(i64, &str); 4] = [
        (0, "sint64_zero"),
        (63, "sint64_small"),
        (-1, "sint64_neg_one"),
        (i64::MIN, "sint64_min"),
    ];

    for (value, name) in sint64_values {
        group.bench_with_input(BenchmarkId::new("sizeof_sint64", name), &value, |b, &v| {
            b.iter(|| sizeof_sint64(black_box(v)))
        });
    }

    group.finish();
}

fn bench_write_signed(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_signed");

    let int32_values: [(i32, &str); 3] =
        [(0, "int32_zero"), (127, "int32_small"), (-1, "int32_neg")];

    for (value, name) in int32_values {
        group.bench_with_input(BenchmarkId::new("write_int32", name), &value, |b, &v| {
            let mut buf = Vec::with_capacity(10);
            b.iter(|| {
                buf.clear();
                buf.write_int32(black_box(v)).unwrap()
            })
        });
    }

    let sint32_values: [(i32, &str); 3] =
        [(0, "sint32_zero"), (63, "sint32_small"), (-1, "sint32_neg")];

    for (value, name) in sint32_values {
        group.bench_with_input(BenchmarkId::new("write_sint32", name), &value, |b, &v| {
            let mut buf = Vec::with_capacity(10);
            b.iter(|| {
                buf.clear();
                buf.write_sint32(black_box(v)).unwrap()
            })
        });
    }

    group.finish();
}

fn bench_write_fixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_fixed");

    group.bench_function("write_fixed32", |b| {
        let mut buf = Vec::with_capacity(4);
        b.iter(|| {
            buf.clear();
            buf.write_fixed32(black_box(0x12345678)).unwrap()
        })
    });

    group.bench_function("write_fixed64", |b| {
        let mut buf = Vec::with_capacity(8);
        b.iter(|| {
            buf.clear();
            buf.write_fixed64(black_box(0x123456789ABCDEF0)).unwrap()
        })
    });

    group.bench_function("write_float", |b| {
        let mut buf = Vec::with_capacity(4);
        b.iter(|| {
            buf.clear();
            buf.write_float(black_box(std::f32::consts::PI)).unwrap()
        })
    });

    group.bench_function("write_double", |b| {
        let mut buf = Vec::with_capacity(8);
        b.iter(|| {
            buf.clear();
            buf.write_double(black_box(std::f64::consts::PI)).unwrap()
        })
    });

    group.finish();
}

fn bench_tag(c: &mut Criterion) {
    let mut group = c.benchmark_group("tag");

    let field_numbers: [(u32, &str); 4] = [
        (1, "field_1"),
        (15, "field_15"),
        (16, "field_16"),
        (1000, "field_1000"),
    ];

    for (field_num, name) in field_numbers {
        group.bench_with_input(BenchmarkId::new("varint", name), &field_num, |b, &f| {
            b.iter(|| tag(black_box(f), WireType::Varint))
        });
    }

    group.finish();
}

fn bench_write_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_bytes");

    let sizes: [(usize, &str); 5] = [
        (0, "empty"),
        (10, "small_10"),
        (100, "medium_100"),
        (1000, "large_1k"),
        (10000, "xlarge_10k"),
    ];

    for (size, name) in sizes {
        let data = vec![0xABu8; size];
        group.bench_with_input(BenchmarkId::new("bytes", name), &data, |b, d| {
            let mut buf = Vec::with_capacity(size + 5);
            b.iter(|| {
                buf.clear();
                buf.write_bytes(black_box(d)).unwrap()
            })
        });
    }

    // Also benchmark strings
    let string_sizes: [(usize, &str); 3] = [
        (10, "str_small_10"),
        (100, "str_medium_100"),
        (1000, "str_large_1k"),
    ];

    for (size, name) in string_sizes {
        let s: String = "a".repeat(size);
        group.bench_with_input(BenchmarkId::new("string", name), &s, |b, s| {
            let mut buf = Vec::with_capacity(size + 5);
            b.iter(|| {
                buf.clear();
                buf.write_string(black_box(s)).unwrap()
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_sizeof_varint,
    bench_write_varint,
    bench_sizeof_signed,
    bench_write_signed,
    bench_write_fixed,
    bench_tag,
    bench_write_bytes,
);
criterion_main!(benches);
