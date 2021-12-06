use std::{mem::MaybeUninit, ptr::null_mut};

use vpx_sys::VPX_DECODER_ABI_VERSION;

pub mod vp8;

#[test]
fn vpx() {
    unsafe {
        let dec_interface = vpx_sys::vpx_codec_vp8_dx();
        let mut codec: MaybeUninit<vpx_sys::vpx_codec_ctx_t> = MaybeUninit::uninit();

        vpx_sys::vpx_codec_dec_init_ver(
            codec.as_mut_ptr(),
            dec_interface,
            null_mut(),
            0,
            VPX_DECODER_ABI_VERSION as i32,
        );
        let mut codec = codec.assume_init();

        let first_frame = include_bytes!("../test_frames/first_frame.vp8");

        assert!(
            vpx_sys::vpx_codec_decode(
                &mut codec,
                first_frame.as_ptr(),
                first_frame.len() as u32,
                null_mut(),
                1,
            ) as u32
                != 0
        );

        let mut ref_update_flag = 0i32;
        vpx_sys::vpx_codec_control_(
            &mut codec,
            vpx_sys::vp8_dec_control_id::VP8D_GET_LAST_REF_UPDATES as i32,
            &mut ref_update_flag,
        );

        let last_frame_updated =
            (ref_update_flag & vpx_sys::vpx_ref_frame_type::VP8_LAST_FRAME as i32) > 0;
        let alt_frame_updated =
            (ref_update_flag & vpx_sys::vpx_ref_frame_type::VP8_ALTR_FRAME as i32) > 0;
        let gold_frame_updated =
            (ref_update_flag & vpx_sys::vpx_ref_frame_type::VP8_GOLD_FRAME as i32) > 0;

        dbg!((last_frame_updated, alt_frame_updated, gold_frame_updated));
    }
}
