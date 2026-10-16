//! PNG encoding/decoding exercises only in-memory SDL streams, never a file sink.

use super::FrameCapture;
use sdl2::image::ImageRWops;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rwops::RWops;

#[test]
fn png_round_trip_is_lossless_and_does_not_require_a_file() {
    let _guard = crate::sdl_test_lock::lock_sdl_for_test().expect("SDL test lock");
    let mut frame = FrameCapture {
        width: 2_u32,
        height: 1_u32,
        rgba: vec![255_u8, 0, 40, 255, 20, 250, 80, 255],
    };
    let expected = frame.rgba.clone();
    let bytes = frame.png_bytes().expect("PNG encodes in memory");
    assert_eq!(
        &bytes[..8_usize],
        &[137_u8, 80, 78, 71, 13, 10, 26, 10],
        "real PNG signature"
    );
    let stream = RWops::from_bytes(&bytes).expect("read-only in-memory stream");
    let surface = stream
        .load_png()
        .expect("PNG decodes")
        .convert_format(PixelFormatEnum::RGBA32)
        .expect("RGBA output");
    assert_eq!(
        (surface.width(), surface.height()),
        (2_u32, 1_u32),
        "dimensions preserved"
    );
    assert_eq!(
        surface.without_lock().expect("unlocked pixels"),
        expected,
        "every channel is lossless"
    );
    assert_eq!(
        frame.rgba, expected,
        "encoding never mutates the source frame"
    );
}

#[test]
fn malformed_frame_is_rejected_before_any_encoder_or_file_sink() {
    let mut frame = FrameCapture {
        width: 2_u32,
        height: 2_u32,
        rgba: vec![0_u8; 4_usize],
    };
    assert!(
        frame.png_bytes().is_err(),
        "short buffers cannot reach SDL raw surface construction"
    );
    frame.width = u32::MAX;
    assert!(frame.png_bytes().is_err(), "pitch overflow is rejected");
    frame.width = 0_u32;
    assert!(frame.png_bytes().is_err(), "zero extents are rejected");
}
