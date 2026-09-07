//! Forced 4x4 inter partitions must retain exact lossless samples,
//! including partial chroma edges. SUB8_INTER_ARTIFACTS exports raw OBUs
//! and native-depth source planes for independent decoder comparison.
#![cfg(not(target_arch = "wasm32"))]
use zenrav1e::prelude::*;

fn sample(n: usize, p: usize, x: usize, y: usize, depth: usize) -> u16 {
  ((x * 17 + y * 31 + (x * y % 29) * 7 + n * 13 + p * 47) % (1 << depth))
    as u16
}

fn check<P: Pixel>(
  depth: usize, chroma: ChromaSampling, count: usize, trellis: bool,
  tune: Tune, bottomup: bool, w: usize, h: usize,
) {
  let (cw, ch) = if chroma == ChromaSampling::Cs420 {
    (w.div_ceil(2), h.div_ceil(2))
  } else {
    (w, h)
  };
  let plane_count = if chroma == ChromaSampling::Cs400 { 1 } else { 3 };
  let mut speed = SpeedSettings::from_preset(10);
  speed.partition.encode_bottomup = bottomup;
  speed.partition.partition_range =
    PartitionRange::new(BlockSize::BLOCK_4X4, BlockSize::BLOCK_4X4);
  let config = EncoderConfig {
    width: w,
    height: h,
    bit_depth: depth,
    chroma_sampling: chroma,
    quantizer: 0,
    min_quantizer: 0,
    enable_trellis: trellis,
    enable_vaq: true,
    vaq_strength: 2.0,
    seg_boost: 1.5,
    tune,
    still_picture: count == 1,
    low_latency: true,
    min_key_frame_interval: count as u64,
    max_key_frame_interval: count as u64,
    speed_settings: speed,
    ..Default::default()
  };
  let mut ctx: Context<P> = Config::new()
    .with_encoder_config(config)
    .with_threads(1)
    .new_context()
    .unwrap();
  let mut expected = Vec::new();
  for n in 0..count {
    let mut frame = ctx.new_frame();
    let mut pixels = Vec::new();
    for p in 0..plane_count {
      let (pw, ph) = if p == 0 { (w, h) } else { (cw, ch) };
      let mut slice = frame.planes[p].mut_slice(Default::default());
      for (y, row) in slice.rows_iter_mut().take(ph).enumerate() {
        for (x, out) in row[..pw].iter_mut().enumerate() {
          let value = sample(n, p, x, y, depth);
          *out = P::cast_from(value);
          pixels.push(value);
        }
      }
    }
    expected.push(pixels);
    ctx.send_frame(frame).unwrap();
  }
  ctx.flush();
  let mut decoder = rav1d_safe::Decoder::new().unwrap();
  let mut decoded = Vec::new();
  let mut bitstream = Vec::new();
  loop {
    match ctx.receive_packet() {
      Ok(packet) => {
        bitstream.extend_from_slice(&packet.data);
        if let Some(rec) = packet.rec.as_ref() {
          let mut reconstructed = Vec::new();
          for p in 0..plane_count {
            let (pw, ph) = if p == 0 { (w, h) } else { (cw, ch) };
            let slice = rec.planes[p].slice(Default::default());
            for row in slice.rows_iter().take(ph) {
              reconstructed
                .extend(row[..pw].iter().map(|&v| u16::cast_from(v)));
            }
          }
          let n = packet.input_frameno as usize;
          assert!(
            reconstructed == expected[n],
            "encoder recon differs: {chroma:?} depth={depth} frame={n} bottomup={bottomup}"
          );
        }

        if let Some(frame) = decoder.decode(&packet.data).unwrap() {
          decoded.push(frame);
        }
      }
      Err(EncoderStatus::Encoded) => {}
      Err(EncoderStatus::LimitReached) => break,
      Err(error) => panic!("encode: {error:?}"),
    }
  }
  if let Some(dir) = std::env::var_os("SUB8_INTER_ARTIFACTS") {
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    let tag =
      format!("{chroma:?}-d{depth}-{w}x{h}-{tune:?}-bottomup{bottomup}");
    std::fs::write(dir.join(format!("{tag}.obu")), bitstream).unwrap();
    let source: Vec<u8> = expected
      .iter()
      .flatten()
      .flat_map(|&v| {
        if depth == 8 { vec![v as u8] } else { v.to_le_bytes().to_vec() }
      })
      .collect();
    std::fs::write(dir.join(format!("{tag}.source.yuv")), source).unwrap();
  }
  decoded.extend(decoder.flush().unwrap());
  assert_eq!(decoded.len(), count);
  for (n, frame) in decoded.iter().enumerate() {
    assert_eq!((frame.width() as usize, frame.height() as usize), (w, h));
    let mut pixels = Vec::new();
    macro_rules! collect {
      ($planes:expr) => {{
        let planes = $planes;
        for row in planes.y().rows().take(h) {
          pixels.extend(row[..w].iter().map(|&v| u16::from(v)));
        }
        if plane_count == 3 {
          for plane in [planes.u().unwrap(), planes.v().unwrap()] {
            for row in plane.rows().take(ch) {
              pixels.extend(row[..cw].iter().map(|&v| u16::from(v)));
            }
          }
        }
      }};
    }
    match frame.planes() {
      rav1d_safe::Planes::Depth8(planes) => collect!(planes),
      rav1d_safe::Planes::Depth16(planes) => collect!(planes),
    }
    if pixels != expected[n] {
      let first: Vec<_> = pixels
        .iter()
        .zip(&expected[n])
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .take(12)
        .collect();
      eprintln!("bottomup={bottomup} first differences={first:?}");
    }
    assert!(
      pixels == expected[n],
      "source mismatch: depth={depth}, chroma={chroma:?}, frames={count}, frame={n}, trellis={trellis}, tune={tune:?}"
    );
  }
}

fn check_chroma(chroma: ChromaSampling) {
  for (w, h) in [(65, 67), (67, 65), (68, 68), (64, 64)] {
    for tune in [Tune::Psychovisual, Tune::Psnr] {
      for bottomup in [true, false] {
        check::<u8>(8, chroma, 2, false, tune, bottomup, w, h);
        check::<u16>(10, chroma, 2, false, tune, bottomup, w, h);
        check::<u16>(12, chroma, 2, false, tune, bottomup, w, h);
      }
    }
  }
}
#[test]
fn forced_sub8_inter_mono_preserves_source() {
  check_chroma(ChromaSampling::Cs400);
}
#[test]
fn forced_sub8_inter_420_preserves_source() {
  check_chroma(ChromaSampling::Cs420);
}
#[test]
fn forced_sub8_inter_444_preserves_source() {
  check_chroma(ChromaSampling::Cs444);
}
