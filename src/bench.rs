#![feature(test)]

extern crate test;
extern crate fastdivide;

use fastdivide::DividerU64;
use test::Bencher;

#[bench]
fn bench_normal_divide(b: &mut Bencher) {
    let q: u64 = test::black_box(112u64);
    b.iter(|| {
        let n: u64 = test::black_box(152342341u64);
        n / q
    })
}

#[bench]
fn bench_fast_divide(b: &mut Bencher) {
    let fast_divider = DividerU64::divide_by(112u64);
    b.iter(|| {
        let n: u64 = test::black_box(152342341u64);
        fast_divider.divide(n)
    })
}
