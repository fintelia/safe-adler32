#![feature(test)]
extern crate test;

const MOD: u32 = 65521;
const NMAX: usize = 5552;

pub fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;

    let chunks = data.chunks_exact(NMAX);
    let remainder = chunks.remainder();

    for chunk in chunks {
        for byte in chunk {
            a = a.wrapping_add(*byte as _);
            b = b.wrapping_add(a);
        }

        a %= MOD;
        b %= MOD;
    }

    for byte in remainder {
        a = a.wrapping_add(*byte as _);
        b = b.wrapping_add(a);
    }

    a %= MOD;
    b %= MOD;

    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[bench]
    fn bench_add(b: &mut test::Bencher) {
        // generate random data for testing
        let data: Vec<u8> = (0..10000).map(|_| rand::random::<u8>()).collect();
        b.bytes = data.len() as u64;
        b.iter(|| adler32(&data));
    }
}
