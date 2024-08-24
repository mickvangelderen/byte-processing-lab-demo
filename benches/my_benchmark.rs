
use std::{hint::unreachable_unchecked, iter};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn func_flt_safe(bytes: &[u8]) -> Vec<u8> {
    assert_eq!(bytes.len() % 2, 0, "bytes should be a multiple of 2");
    bytes.chunks(2).map(|chunk| {
        let value = u16::from_le_bytes(chunk.try_into().unwrap());
        (f32::from(value) * 0.25) as u8
    }).collect()
}

fn func_int_safe(bytes: &[u8]) -> Vec<u8> {
    assert_eq!(bytes.len() % 2, 0, "bytes should be a multiple of 2");
    bytes.chunks(2).map(|chunk| {
        let value = u16::from_le_bytes(chunk.try_into().unwrap());
        (value >> 2) as u8
    }).collect()
}

fn func_int_unsafe(bytes: &[u8]) -> Vec<u8> {
    assert_eq!(bytes.len() % 2, 0, "bytes should be a multiple of 2");
    bytes.chunks(2).map(|chunk| {
        let value = {
            let [b0, b1] = *chunk else {
                unsafe {
                    unreachable_unchecked();
                }
            };
            u16::from_le_bytes([b0, b1])
        };
        (value >> 2) as u8
    }).collect()
}

fn proc_flt_safe(bytes: &[u8]) -> Vec<u8> {
    assert_eq!(bytes.len() % 2, 0, "bytes should be a multiple of 2");
    let pixels = bytes.len() / 2;
    let mut output = Vec::with_capacity(pixels);
    for pixel in 0..pixels {
        let value = u16::from_le_bytes([
            bytes[pixel*2],
            bytes[pixel*2 + 1],
        ]);
        output.push((f32::from(value) * 0.25) as u8);
    }
    output
}

fn proc_int_safe(bytes: &[u8]) -> Vec<u8> {
    assert_eq!(bytes.len() % 2, 0, "bytes should be a multiple of 2");
    let pixels = bytes.len() / 2;
    let mut output = vec![0; pixels];
    for pixel in 0..pixels {
        let value = u16::from_le_bytes([
            bytes[pixel*2],
            bytes[pixel*2 + 1],
        ]);
        output[pixel] = (value >> 2) as u8;
    }
    output
}

fn proc_int_unsafe(bytes: &[u8]) -> Vec<u8> {
    assert_eq!(bytes.len() % 2, 0, "bytes should be a multiple of 2");
    let pixels = bytes.len() / 2;
    let mut output: Vec<u8> = Vec::with_capacity(pixels);
    for pixel in 0..pixels {
        let value = u16::from_le_bytes(unsafe {[
            *bytes.get_unchecked(pixel*2),
            *bytes.get_unchecked(pixel*2 + 1),
        ]});
        unsafe {
            output.as_mut_ptr().add(pixel).write((value >> 2) as u8);
        }
    }
    unsafe {
        output.set_len(pixels);
    }
    output
}

fn proc_int_unsafe_2(bytes: &[u8]) -> Vec<u8> {
    assert_eq!(bytes.len() % 2, 0, "bytes should be a multiple of 2");
    let pixels = bytes.len() / 2;
    let mut output: Vec<u8> = Vec::with_capacity(pixels);

    let mut r = bytes.as_ptr();
    let mut w = output.as_mut_ptr();
    let w1 = unsafe { w.add(pixels) };
    while w != w1 {
        let b0 = unsafe {
            let v = r.read();
            r = r.add(1);
            v
        };
        let b1 = unsafe {
            let v = r.read();
            r = r.add(1);
            v
        };
        let value = u16::from_le_bytes([b0, b1]);
        unsafe {
            w.write((value >> 2) as u8);
            w = w.add(1);
        }
    }

    unsafe {
        output.set_len(pixels);
    }
    output
}

fn bench_rggb10_to_rggb8(c: &mut Criterion) {
    let mut group = c.benchmark_group("rggb10_to_rggb8");
    group.significance_level(0.01).sample_size(500);

    #[allow(clippy::single_element_loop)]
    for size_kb in [16] {
        let pixel_count = size_kb * 1024;
        group.throughput(Throughput::Bytes(pixel_count * 2));
        let input = iter::repeat(0u8).take((pixel_count * 2) as usize).collect::<Vec<_>>();
        group.bench_with_input(BenchmarkId::new("func_flt_safe", pixel_count), &input, |b, input| {
            b.iter(|| func_flt_safe(input));
        });
        group.bench_with_input(BenchmarkId::new("func_int_safe", pixel_count), &input, |b, input| {
            b.iter(|| func_int_safe(input));
        });
        group.bench_with_input(BenchmarkId::new("func_int_unsafe", pixel_count), &input, |b, input| {
            b.iter(|| func_int_unsafe(input));
        });
        group.bench_with_input(BenchmarkId::new("proc_flt_safe", pixel_count), &input, |b, input| {
            b.iter(|| proc_flt_safe(input));
        });
        group.bench_with_input(BenchmarkId::new("proc_int_safe", pixel_count), &input, |b, input| {
            b.iter(|| proc_int_safe(input));
        });
        group.bench_with_input(BenchmarkId::new("proc_int_unsafe", pixel_count), &input, |b, input| {
            b.iter(|| proc_int_unsafe(input));
        });
        group.bench_with_input(BenchmarkId::new("proc_int_unsafe_2", pixel_count), &input, |b, input| {
            b.iter(|| proc_int_unsafe_2(input));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_rggb10_to_rggb8);
criterion_main!(benches);