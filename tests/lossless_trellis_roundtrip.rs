//! Lossless coding must preserve samples even when lossy RDO tools are requested.
#![cfg(not(target_arch = "wasm32"))]
use zenrav1e::prelude::*;

fn sample(n: usize, p: usize, x: usize, y: usize, depth: usize) -> u16 {
  ((x * 17 + y * 31 + (x * y % 29) * 7 + n * 13 + p * 47) % (1 << depth))
    as u16
}

fn check<P: Pixel>(
  depth: usize, chroma: ChromaSampling, count: usize, trellis: bool,
  tune: Tune,
) {
  let (w, h) = (65, 67);
  let (cw, ch) =
    if chroma == ChromaSampling::Cs420 { (33, 34) } else { (w, h) };
  let plane_count = if chroma == ChromaSampling::Cs400 { 1 } else { 3 };
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
    speed_settings: SpeedSettings::from_preset(10),
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
  loop {
    match ctx.receive_packet() {
      Ok(packet) => {
        if let Some(frame) = decoder.decode(&packet.data).unwrap() {
          decoded.push(frame);
        }
      }
      Err(EncoderStatus::Encoded) => {}
      Err(EncoderStatus::LimitReached) => break,
      Err(error) => panic!("encode: {error:?}"),
    }
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
    assert!(
      pixels == expected[n],
      "source mismatch: depth={depth}, chroma={chroma:?}, frames={count}, frame={n}, trellis={trellis}, tune={tune:?}"
    );
  }
}

#[test]
fn lossless_trellis_never_changes_source_samples() {
  for chroma in
    [ChromaSampling::Cs400, ChromaSampling::Cs420, ChromaSampling::Cs444]
  {
    for count in [1, 2] {
      for tune in [Tune::Psychovisual, Tune::StillImage] {
        for trellis in [false, true] {
          check::<u8>(8, chroma, count, trellis, tune);
          check::<u16>(10, chroma, count, trellis, tune);
          check::<u16>(12, chroma, count, trellis, tune);
        }
      }
    }
  }
}
