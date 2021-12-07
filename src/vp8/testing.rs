use std::{mem::MaybeUninit, ptr::null_mut};

use vpx_sys::VPX_DECODER_ABI_VERSION;

use super::FrameBufferUpdate;

#[derive(bincode::Encode, bincode::Decode)]
struct TestFrames {
    frames: Vec<Vec<u8>>,
}

struct Vp8TestDecoder {
    ctx: vpx_sys::vpx_codec_ctx_t,
}

impl Vp8TestDecoder {
    pub fn new() -> Self {
        let mut codec: MaybeUninit<vpx_sys::vpx_codec_ctx_t> = MaybeUninit::uninit();

        let codec = unsafe {
            vpx_sys::vpx_codec_dec_init_ver(
                codec.as_mut_ptr(),
                vpx_sys::vpx_codec_vp8_dx(),
                null_mut(),
                0,
                VPX_DECODER_ABI_VERSION as i32,
            );
            codec.assume_init()
        };

        Self { ctx: codec }
    }

    pub fn analyze_frame(&mut self, frame: &[u8]) -> FrameBufferUpdate {
        let mut ref_update_flag = 0i32;

        unsafe {
            assert_eq!(
                vpx_sys::vpx_codec_decode(
                    &mut self.ctx,
                    frame.as_ptr(),
                    frame.len() as u32,
                    null_mut(),
                    1,
                ),
                vpx_sys::vpx_codec_err_t::VPX_CODEC_OK
            );

            assert_eq!(
                vpx_sys::vpx_codec_control_(
                    &mut self.ctx,
                    vpx_sys::vp8_dec_control_id::VP8D_GET_LAST_REF_UPDATES as i32,
                    &mut ref_update_flag,
                ),
                vpx_sys::vpx_codec_err_t::VPX_CODEC_OK
            );
        }

        let _last_frame_updated =
            (ref_update_flag & vpx_sys::vpx_ref_frame_type::VP8_LAST_FRAME as i32) > 0;
        let alt_frame_updated =
            (ref_update_flag & vpx_sys::vpx_ref_frame_type::VP8_ALTR_FRAME as i32) > 0;
        let gold_frame_updated =
            (ref_update_flag & vpx_sys::vpx_ref_frame_type::VP8_GOLD_FRAME as i32) > 0;

        dbg!(ref_update_flag);

        FrameBufferUpdate {
            golden: gold_frame_updated,
            altref: alt_frame_updated,
        }
    }
}

impl Drop for Vp8TestDecoder {
    fn drop(&mut self) {
        unsafe {
            vpx_sys::vpx_codec_destroy(&mut self.ctx);
        }
    }
}

#[test]
fn vpx() {
    let frames: TestFrames = bincode::decode_from_slice(
        &std::fs::read("test_frames.vp8").unwrap(),
        bincode::config::Configuration::standard(),
    )
    .unwrap();

    let mut decoder = Vp8TestDecoder::new();

    let expected = frames
        .frames
        .iter()
        .map(|f| decoder.analyze_frame(&f))
        .collect::<Vec<_>>();

    let parsed = frames
        .frames
        .iter()
        .map(|f| {
            crate::vp8::FrameInfo::parse(&f)
                .unwrap()
                .header
                .frame_buffer_update
        })
        .collect::<Vec<_>>();

    assert_eq!(expected, parsed);

    // for (idx, frame) in frames.frames.into_iter().enumerate() {
    //     eprintln!("testing frame {}", idx);
    //     let expected = decoder.analyze_frame(&frame);

    //     let parsed = crate::vp8::FrameInfo::parse(&frame).unwrap();

    //     assert_eq!(parsed.header.frame_buffer_update, expected);
    // }
}
