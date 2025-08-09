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
        let (chunk1, chunk2) = chunk.split_at(NMAX / 2);

        let mut a2 = 0u32;
        let mut b2 = 0u32;
        for (&v0, &v1) in chunk1.iter().zip(chunk2.iter()) {
            a = a.wrapping_add(v0 as u32);
            b = b.wrapping_add(a);

            a2 = a2.wrapping_add(v1 as u32);
            b2 = b2.wrapping_add(a2);
        }

        b += a * (NMAX / 2) as u32 + b2;
        a += a2;

        // while let [v0, v1, v2, v3, remainder @ ..] = chunk {
        //     b = b.wrapping_add(
        //         a * 4 + *v0 as u32 * 4 + *v1 as u32 * 3 + *v2 as u32 * 2 + *v3 as u32,
        //     );
        //     a = a.wrapping_add(*v0 as u32 + *v1 as u32 + *v2 as u32 + *v3 as u32);
        //     chunk = remainder;
        // }

        // for byte in chunk {
        //     b = b.wrapping_add(a + *byte as u32);
        //     a = a.wrapping_add(*byte as _);
        // }

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

    #[test]
    fn test_adler32() {
        let data: Vec<u8> = (0..10000).map(|_| rand::random::<u8>()).collect();
        let checksum = adler32(&data);
        let reference = adler2::adler32(std::io::Cursor::new(&data)).unwrap();
        assert_eq!(checksum, reference, "Checksum mismatch");
    }

    #[bench]
    fn bench_add(b: &mut test::Bencher) {
        // generate random data for testing
        let data: Vec<u8> = (0..NMAX*10).map(|_| rand::random::<u8>()).collect();
        b.bytes = data.len() as u64;
        b.iter(|| adler32(&data));
    }
}
