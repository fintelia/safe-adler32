# safe-adler32

This crate implements Adler-32 checksums using Rust's `std::simd` and without
any unsafe code. It is designed to be safe, portable, and efficient.

## Performance

With default target features, performance is about 30,000 MB/s on a Ryzen 5
5600X. When compiling with `-C target-cpu=x86-64-v3`, performance increases to
about 43,000 MB/s.