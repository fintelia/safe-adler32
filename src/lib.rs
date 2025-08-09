#![feature(test, portable_simd)]
extern crate test;

use core::{
    simd::{Simd, num::SimdUint},
    slice::ChunksExact,
};

const LANES: usize = 32;
const MOD: u32 = 65521;
const NMAX: usize = 5552 & !(LANES - 1);

pub fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;

    let chunks = data.chunks_exact(NMAX);
    let remainder = chunks.remainder();

    for chunk in chunks {
        update_simd(&mut a, &mut b, chunk.chunks_exact(LANES));
        a %= MOD;
        b %= MOD;
    }

    let vs = remainder.chunks_exact(LANES);
    let vremainder = vs.remainder();
    update_simd(&mut a, &mut b, vs);

    for byte in vremainder {
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
        // weights[LANES - 1 - i] = i as u32 + 1;
        weights[i] = i as u32;
        i += 1;
    }

    Simd::from_array(weights)
};

fn update_simd(a_out: &mut u32, b_out: &mut u32, values: ChunksExact<u8>) {
    let mut a: Simd<u32, LANES> = Simd::splat(0);
    let mut b: Simd<u32, LANES> = Simd::splat(0);

    let len = values.len() * LANES;

    for v in values {
        a += Simd::from_slice(v).cast::<u32>();
        b += a;
    }

    *b_out += *a_out * len as u32 + LANES as u32 * b.reduce_sum() - (a * WEIGHTS).reduce_sum();
    *a_out += a.cast::<u32>().reduce_sum();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adler32() {
        // panic!("weights = {:?}", WEIGHTS);

        let data: Vec<u8> = (0..100000).map(|_| rand::random::<u8>()).collect();
        let checksum = adler32(&data);
        let reference = adler2::adler32(std::io::Cursor::new(&data)).unwrap();
        assert_eq!(checksum, reference, "Checksum mismatch {:x} != {:x}", checksum, reference);
    }

    #[bench]
    fn bench_add(b: &mut test::Bencher) {
        // generate random data for testing
        let data: Vec<u8> = (0..10000).map(|_| rand::random::<u8>()).collect();
        b.bytes = data.len() as u64;
        b.iter(|| adler32(&data));
    }
}
