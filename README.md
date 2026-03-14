# safe-adler32

This crate implements Adler-32 checksums using Rust's `std::simd` and without
any unsafe code. It is designed to be safe, portable, and efficient.

## Performance

With default target features, performance is about 44 GB/s on a Ryzen 9
9900X. When compiling with `-C target-cpu=x86-64-v4`, performance increases to
about 131 GB/s.
