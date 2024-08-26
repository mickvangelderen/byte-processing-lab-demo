use std::iter;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn assert_input_valid(input: &[u8]) {
    assert_eq!(input.len() % 2, 0, "input length should be a multiple of 2");
}

fn transform(pair: [u8; 2]) -> u8 {
    (u16::from_le_bytes(pair) >> 2) as u8
}

fn func_safe(input: &[u8]) -> Vec<u8> {
    assert_input_valid(input);
    input
        .chunks(2)
        .map(|chunk| transform(chunk.try_into().unwrap()))
        .collect()
}

fn func_unsafe(input: &[u8]) -> Vec<u8> {
    assert_input_valid(input);
    let pair_count = input.len() / 2;
    let mut output: Vec<u8> = Vec::with_capacity(pair_count);

    let mut r = input.as_ptr();
    let mut w = output.as_mut_ptr();
    let w_end = unsafe { w.add(pair_count) };
    while w != w_end {
        let b0 = unsafe { r.read_then_advance() };
        let b1 = unsafe { r.read_then_advance() };
        unsafe { w.write_then_advance(transform([b0, b1])) };
    }

    unsafe {
        output.set_len(pair_count);
    }
    output
}

trait PtrExt {
    type Element;

    unsafe fn read_then_advance(&mut self) -> Self::Element
    where
        Self::Element: Sized;
}

impl<T> PtrExt for *const T {
    type Element = T;

    unsafe fn read_then_advance(&mut self) -> Self::Element
    where
        Self::Element: Sized,
    {
        let val = self.read();
        *self = self.add(1);
        val
    }
}

trait PtrMutExt {
    type Element;

    unsafe fn write_then_advance(&mut self, val: Self::Element)
    where
        Self::Element: Sized;
}

impl<T> PtrMutExt for *mut T {
    type Element = T;

    unsafe fn write_then_advance(&mut self, val: Self::Element)
    where
        Self::Element: Sized,
    {
        self.write(val);
        *self = self.add(1);
    }
}

fn bench_func(c: &mut Criterion) {
    let mut group = c.benchmark_group("func");
    group.significance_level(0.01).sample_size(500);

    #[allow(clippy::single_element_loop)]
    for pair_count in [16 * 1024] {
        let byte_count = pair_count * 2;
        group.throughput(Throughput::Bytes(byte_count));
        let input = iter::repeat(0u8)
            .take(byte_count as usize)
            .collect::<Vec<_>>();
        group.bench_with_input(
            BenchmarkId::new("safe", pair_count),
            &input,
            |b, input| {
                b.iter(|| func_safe(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("unsafe", pair_count),
            &input,
            |b, i| {
                b.iter(|| func_unsafe(i));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_func);
criterion_main!(benches);
