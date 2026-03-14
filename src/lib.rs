#![forbid(unsafe_code)]
#![no_std]
#![feature(portable_simd)]
#![cfg_attr(test, feature(test))]
#[cfg(test)]
extern crate test;

use core::simd::{Simd, num::SimdUint};

const LANES: usize = 32;
const MOD: u32 = 65521;

pub fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;

    let (chunks, data) = data.as_chunks::<{ 347 * LANES }>();
    for chunk in chunks {
        update_simd(&mut a, &mut b, chunk);
        a %= MOD;
        b %= MOD;
    }

    let (vs, data) = data.split_at(data.len() & !(LANES - 1));
    update_simd(&mut a, &mut b, vs);

    for byte in data {
        a = a.wrapping_add(*byte as _);
        b = b.wrapping_add(a);
    }

    a %= MOD;
    b %= MOD;

    (b << 16) | a
}

const WEIGHTS: Simd<u32, LANES> = {
    let mut weights = [0; LANES];
    let mut i = 0;
    while i < LANES {
        weights[i] = i as u32;
        i += 1;
    }

    Simd::from_array(weights)
};

fn update_simd(a_out: &mut u32, b_out: &mut u32, data: &[u8]) {
    let mut a: Simd<u32, LANES> = Simd::splat(0);
    let mut b: Simd<u32, LANES> = Simd::splat(0);

    let len = data.len();

    let (chunks, data) = data.as_chunks::<{ 16 * LANES }>();
    for chunk in chunks {
        let (a_part, b_part) = update_simd_inner(chunk);
        b += a * Simd::splat(16) + b_part.cast();
        a += a_part.cast();
    }

    let (a_part, b_part) = update_simd_inner(data);
    b += a * Simd::splat((data.len() / LANES) as u32) + b_part.cast();
    a += a_part.cast();

    *b_out += ((*a_out as u64 * len as u64 + LANES as u64 * b.cast::<u32>().reduce_sum() as u64
        - (a.cast() * WEIGHTS).reduce_sum() as u64)
        % MOD as u64) as u32;
    *a_out += a.cast::<u32>().reduce_sum();
}

fn update_simd_inner(values: &[u8]) -> (Simd<u16, LANES>, Simd<u16, LANES>) {
    let mut a: Simd<u16, LANES> = Simd::splat(0);
    let mut b: Simd<u16, LANES> = Simd::splat(0);

    for v in values.chunks_exact(LANES) {
        a += Simd::from_slice(v).cast();
        b += a;
    }

    (a, b)
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::vec::Vec;

    use super::*;

    #[test]
    fn test_adler32() {
        let data: Vec<u8> = (0..100000).map(|_| rand::random::<u8>()).collect();
        let checksum = adler32(&data);
        let reference = adler2::adler32(std::io::Cursor::new(&data)).unwrap();
        assert_eq!(
            checksum, reference,
            "Checksum mismatch {:x} != {:x}",
            checksum, reference
        );
    }

    #[bench]
    fn bench_10b(b: &mut test::Bencher) {
        let data: Vec<u8> = (0..10).map(|_| rand::random::<u8>()).collect();
        b.bytes = data.len() as u64;
        b.iter(|| adler32(&data));
    }

    #[bench]
    fn bench_10b_simd_adler32(b: &mut test::Bencher) {
        let data: Vec<u8> = (0..10).map(|_| rand::random::<u8>()).collect();
        b.bytes = data.len() as u64;
        b.iter(|| simd_adler32::adler32(&&*data));
    }

    #[bench]
    fn bench_10b_adler2(b: &mut test::Bencher) {
        let data: Vec<u8> = (0..10).map(|_| rand::random::<u8>()).collect();
        b.bytes = data.len() as u64;
        b.iter(|| adler2::adler32(std::io::Cursor::new(&data)));
    }

    #[bench]
    fn bench_100k(b: &mut test::Bencher) {
        let data: Vec<u8> = (0..100000).map(|_| rand::random::<u8>()).collect();
        b.bytes = data.len() as u64;
        b.iter(|| adler32(&data));
    }

    #[bench]
    fn bench_100k_simd_adler32(b: &mut test::Bencher) {
        let data: Vec<u8> = (0..100000).map(|_| rand::random::<u8>()).collect();
        b.bytes = data.len() as u64;
        b.iter(|| simd_adler32::adler32(&&*data));
    }

    #[bench]
    fn bench_100k_adler2(b: &mut test::Bencher) {
        let data: Vec<u8> = (0..100000).map(|_| rand::random::<u8>()).collect();
        b.bytes = data.len() as u64;
        b.iter(|| adler2::adler32(std::io::Cursor::new(&data)));
    }
}
