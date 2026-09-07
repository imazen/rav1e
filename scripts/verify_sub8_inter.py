#!/usr/bin/env python3
"""Compare all forced-sub8 inter streams to source planes with libaom."""
import argparse
import hashlib
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("artifacts", type=Path)
parser.add_argument("--manifest", type=Path, required=True)
args = parser.parse_args()
files = sorted(args.artifacts.glob("*.obu"))
assert len(files) == 144, len(files)
rows = ["file\tbitstream_sha256\tsource_and_decoded_sha256"]
for path in files:
    depth = int(path.stem.split("-")[1][1:])
    raw = path.with_suffix(".aom.yuv")
    result = subprocess.run([
        "aomdec", "--threads=1", "--rawvideo",
        f"--output-bit-depth={depth}", f"--output={raw}", str(path),
    ], capture_output=True, text=True)
    path.with_suffix(".aom.log").write_text(result.stdout + result.stderr)
    assert result.returncode == 0, (path, result.stderr)
    source = path.with_suffix(".source.yuv").read_bytes()
    decoded = raw.read_bytes()
    assert decoded == source, f"{path}: decoded samples differ from source"
    rows.append("\t".join([path.name] + [
        hashlib.sha256(data).hexdigest()
        for data in [path.read_bytes(), source]
    ]))
    print("PASS", path.name, flush=True)
args.manifest.write_text("\n".join(rows) + "\n")
print("PASS: 144 streams / 288 frames equal source planes exactly")
