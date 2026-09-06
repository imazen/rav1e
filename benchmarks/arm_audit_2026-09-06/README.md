# ARM still-encoder audit, 2026-09-06

Coverage: two 8-bit 4:2:0 still images, 256×256 and 512×512, speed 8, q-index 100, one encoder thread. Other speeds, bit depths, quality settings, content classes, and animation are unmeasured. No source constants or production coding path changed.

Apple M4 Pro, 24 GiB RAM, macOS Darwin 25.5, rustc 1.98 / LLVM 22. Base codec `5011a173`, with the accompanying benchmark rewrite. No target-cpu=native. Both commands use `CARGO_BUILD_JOBS=4 RAYON_NUM_THREADS=4 OMP_NUM_THREADS=4 TMPDIR=/Users/lilith/tmp nice -n 19 /usr/bin/time -l cargo bench --locked -p zenrav1e --bench tier_isolation`, followed by `-- --format=llm` or `--features asm -- --format=llm`.

| Size | Rust fallback mean | Compiled NEON assembly mean |
| --- | ---: | ---: |
| 256×256 | 155.56 ms | 116.21 ms |
| 512×512 | 601.71 ms | 448.50 ms |

These are separate builds/runs, not a paired same-process comparison. Each mean's confidence interval and round count are preserved in the logs. The assembly build is faster in both measured cases. The default library build uses Rust; enabling asm selects the existing ARM assembly. Scalar Rust may auto-vectorize.

The updated zenbench harness fails on encoder errors, checks exactly one packet, saves raw OBU bytes, and compares the decoded YUV output against the encoder reconstruction before timing. All four reconstruction checks passed. Fixtures use seeded noise plus patches and are not a quality-calibration corpus. Artifact hashes are in [fixtures.pointer.md](fixtures.pointer.md).

Full logs include `/usr/bin/time -l` resource output for build plus benchmark, not isolated codec memory usage. The asm build's full compiler log exceeds the repository size limit and is retained through its pointer; the measurement section is committed separately.
