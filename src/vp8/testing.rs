use std::{mem::MaybeUninit, ptr::null_mut};

use vpx_sys::VPX_DECODER_ABI_VERSION;

use super::FrameBufferUpdate;

struct Vp8TestDecoder {
    dec_interface: *mut vpx_sys::vpx_codec_iface,
    ctx: vpx_sys::vpx_codec_ctx_t,
}

impl Vp8TestDecoder {
    pub fn new() -> Self {
        let dec_interface = unsafe { vpx_sys::vpx_codec_vp8_dx() };
        let mut codec: MaybeUninit<vpx_sys::vpx_codec_ctx_t> = MaybeUninit::uninit();

        let codec = unsafe {
            vpx_sys::vpx_codec_dec_init_ver(
                codec.as_mut_ptr(),
                dec_interface,
                null_mut(),
                0,
                VPX_DECODER_ABI_VERSION as i32,
            );
            codec.assume_init()
        };

        Self {
            dec_interface,
            ctx: codec,
        }
    }

    pub fn analyze_frame(&mut self, frame: &[u8]) -> FrameBufferUpdate {
        let mut ref_update_flag = 0i32;

        unsafe {
            assert!(
                vpx_sys::vpx_codec_decode(
                    &mut self.ctx,
                    frame.as_ptr(),
                    frame.len() as u32,
                    null_mut(),
                    1,
                ) as u32
                    != 0
            );

            vpx_sys::vpx_codec_control_(
                &mut self.ctx,
                vpx_sys::vp8_dec_control_id::VP8D_GET_LAST_REF_UPDATES as i32,
                &mut ref_update_flag,
            );
        }

        let last_frame_updated =
            (ref_update_flag & vpx_sys::vpx_ref_frame_type::VP8_LAST_FRAME as i32) > 0;
        let alt_frame_updated =
            (ref_update_flag & vpx_sys::vpx_ref_frame_type::VP8_ALTR_FRAME as i32) > 0;
        let gold_frame_updated =
            (ref_update_flag & vpx_sys::vpx_ref_frame_type::VP8_GOLD_FRAME as i32) > 0;

        dbg!((last_frame_updated, alt_frame_updated, gold_frame_updated));

        match (alt_frame_updated, gold_frame_updated) {
            (true, true) => FrameBufferUpdate::KeyFrame,
            _ => FrameBufferUpdate::InterFrame {
                altref: alt_frame_updated,
                golden: gold_frame_updated,
            },
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
    unsafe {
        let mut decoder = Vp8TestDecoder::new();

        let first_frame = include_bytes!("../../test_frames/first_frame.vp8");

        let update = decoder.analyze_frame(first_frame);

        assert!(matches!(update, FrameBufferUpdate::KeyFrame));
    }
}
