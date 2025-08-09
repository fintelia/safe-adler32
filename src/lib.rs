#![feature(test, portable_simd)]
extern crate test;

use core::{
    simd::{Simd, num::SimdUint},
    slice::ChunksExact,
};

const LANES: usize = 32;
const MOD: u32 = 65521;

pub fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;

    let (chunks, remainder) = data.as_chunks::<{ 16 * LANES }>();

    for chunk in chunks {
        update_simd(&mut a, &mut b, chunk);
        a %= MOD;
        b %= MOD;
    }

    let (vs, vremainder) = remainder.split_at(remainder.len() & !(LANES - 1));
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
        weights[i] = i as u32;
        i += 1;
    }

    Simd::from_array(weights)
};

fn update_simd(a_out: &mut u32, b_out: &mut u32, values: &[u8]) {
    let mut a: Simd<u16, LANES> = Simd::splat(0);
    let mut b: Simd<u16, LANES> = Simd::splat(0);

    let len = values.len() / LANES;

    for v in values.chunks_exact(LANES) {
        a += Simd::from_slice(v).cast();
        b += a;
    }

    *b_out += LANES as u32 * (*a_out * len as u32 + b.cast::<u32>().reduce_sum())
        - (a.cast() * WEIGHTS).reduce_sum();
    *a_out += a.cast::<u32>().reduce_sum();
}

#[cfg(test)]
mod tests {
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
    fn bench_add(b: &mut test::Bencher) {
        // generate random data for testing
        let data: Vec<u8> = (0..100000).map(|_| rand::random::<u8>()).collect();
        b.bytes = data.len() as u64;
        b.iter(|| adler32(&data));
    }
}
