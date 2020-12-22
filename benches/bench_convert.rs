#![feature(test)]
extern crate test;
use test::Bencher;

use core::hint::black_box;
use isbn2::*;

#[bench]
fn bench_hyphenate_isbn10(b: &mut Bencher) {
    let digits = Isbn10::new(black_box([9, 9, 7, 1, 5, 0, 2, 1, 0, 0])).unwrap();
    b.iter(|| black_box(digits.hyphenate()))
}

#[bench]
fn bench_hyphenate_isbn13(b: &mut Bencher) {
    let digits = Isbn13::new(black_box([9, 7, 8, 3, 1, 6, 1, 4, 8, 4, 1, 0, 0])).unwrap();
    b.iter(|| black_box(digits.hyphenate()))
}

#[bench]
fn bench_to_string_isbn10(b: &mut Bencher) {
    let digits = Isbn10::new(black_box([9, 9, 7, 1, 5, 0, 2, 1, 0, 0])).unwrap();
    b.iter(|| black_box(digits.to_string()))
}

#[bench]
fn bench_to_string_isbn13(b: &mut Bencher) {
    let digits = Isbn13::new(black_box([9, 7, 8, 3, 1, 6, 1, 4, 8, 4, 1, 0, 0])).unwrap();
    b.iter(|| black_box(digits.to_string()))
}
