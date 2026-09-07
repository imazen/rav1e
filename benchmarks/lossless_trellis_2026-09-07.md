# Lossless trellis correction — 2026-09-07

Base: `605946821afa839ca80f2b9bb226917238e9dba3` on linux-x86_64.

While wiring animation quantization controls in cavif-rs, raw decoded planes
showed that lossless plus trellis changed source samples. This is a backend
lossless defect, including stills: the pre-fix regression first fails on an
8-bit monochrome single frame with Psychovisual tuning and trellis enabled.

`quantize::trellis::optimize` mapped the lossless WHT pseudo-transform onto
the DCT scan and ran its lossy coefficient-level descent. Quantizer zero does
not make that descent lossless. It now returns the original coefficients and
EOB unchanged for WHT_WHT. Ordinary DCT/ADST/identity-transform optimization
is retained, including the existing high-quantizer policy. The read-only
coefficient-rate estimator still supports WHT's shared scan.

The new source-exact decoder regression covers 72 configurations / 108 frames:
8/10/12-bit, monochrome/4:2:0/4:4:4, one/two frames, Psychovisual/StillImage,
and trellis off/on. VAQ and segmentation boost are requested in every case.
The pre-fix tree fails; the corrected tree passes all configurations against
rav1d-safe. This compares decoded samples directly to the generated source,
not just the encoder's potentially incorrect reconstruction.

The owner isolation first disabled VAQ, which did not repair the combined
lossless output. Disabling only trellis did repair every source comparison.
Source inspection confirms segmentation is already gated on quantizer > 0;
there is no basis for attributing this defect to VAQ. The owner test has been
restored to request trellis, and its integration awaits the published fix.

Before-change gates pass: identity 81/81 pinned baselines and 360/360 feature
arms; reconstruction 54/54 cells with both rav1d-safe and libaom. The `just`
binary is absent on this host, so the commands from its recipes were run
directly. All commands use run-heavy with 16G/four jobs and RUST_TEST_THREADS=4.
Raw logs: `~/tmp/slower-preset-probe/lossless-trellis-*.log`.
Full pure-Rust/threading validation passes 225 tests (six existing ignored
doctests). Library and regression-test clippy passes with warnings denied.
After-change identity repeats 81 pinned baselines and 360 feature arms with
zero drift; reconstruction repeats 54/54 cells against both decoders. The
new 72-case source-exact matrix is included in the full suite. Scoped rustfmt
and git diff --check also pass. No pins, thresholds or skips changed.
