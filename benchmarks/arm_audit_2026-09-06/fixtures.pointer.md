# Encoder artifacts

Raw AV1 OBU packets, external root: `/Users/lilith/work/codec-artifacts/zenrav1e-arm-audit`.

- `256x256-neon_asm.obu`: 57141 bytes, SHA256 `5aad99a5d5df04e3fd44fb4623930dd621b365790e3d2ef71f3c3ef1ee0d9843`.
- `256x256-rust_fallback.obu`: 57141 bytes, SHA256 `5aad99a5d5df04e3fd44fb4623930dd621b365790e3d2ef71f3c3ef1ee0d9843`.
- `512x512-neon_asm.obu`: 227994 bytes, SHA256 `fc1c9788eec7f34dd2fec3f09119e593d892a8249bb30c8791afd843a2e6ee02`.
- `512x512-rust_fallback.obu`: 227994 bytes, SHA256 `fc1c9788eec7f34dd2fec3f09119e593d892a8249bb30c8791afd843a2e6ee02`.

256x256: saved assembly and Rust bitstreams are byte-identical.

512x512: saved assembly and Rust bitstreams are byte-identical.
