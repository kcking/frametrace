#[derive(Debug)]
pub struct FrameTag {
    pub frame_type: FrameTagType,
    pub version: u8, // 3 bits
    pub show_frame: bool,
    pub first_part_size: u32, // 19 bits
}

use nom::number::complete::{le_u16, le_u24};

impl FrameTag {
    pub fn parse(data: &[u8]) -> nom::IResult<&[u8], Self> {
        //  https://datatracker.ietf.org/doc/html/rfc6386#section-19.1
        let (data, tmp) = le_u24(data)?;
        //  A 1-bit frame type (0 for key frames, 1 for interframes).
        let key_frame = (tmp & 0x1) == 0;
        let version = ((tmp >> 1) & 0x7) as u8;
        let show_frame = ((tmp >> 4) & 0x1) == 1;
        let first_part_size = (tmp >> 5) & 0x7FFFF;

        let (data, frame_type) = if key_frame {
            let (data, _start_code) = nom::bytes::complete::tag(&[0x9du8, 0x01, 0x2a])(data)?;

            let (data, tmp) = le_u16(data)?;
            let width = tmp & 0x3FFF;
            let width_scale = (tmp >> 14) as u8;

            let (data, tmp) = le_u16(data)?;
            let height = tmp & 0x3FFF;
            let height_scale = (tmp >> 14) as u8;

            (
                data,
                FrameTagType::KeyFrame {
                    width,
                    width_scale,
                    height,
                    height_scale,
                },
            )
        } else {
            (data, FrameTagType::InterFrame)
        };

        Ok((
            data,
            Self {
                version,
                show_frame,
                first_part_size,
                frame_type,
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub enum FrameTagType {
    KeyFrame {
        width: u16,       // 14 bits
        width_scale: u8,  // 2 bits
        height: u16,      // 14 bits
        height_scale: u8, // 2 bits
    },
    InterFrame,
}

impl FrameTagType {
    pub fn is_key_frame(&self) -> bool {
        matches!(self, FrameTagType::KeyFrame { .. })
    }
}

///  https://datatracker.ietf.org/doc/html/rfc6386#section-19.2
#[derive(Debug, Clone)]
pub struct FrameHeader {
    pub frame_buffer_update: FrameBufferUpdate,
}

#[derive(Debug, Clone)]
pub enum FrameBufferUpdate {
    KeyFrame,
    InterFrame { golden: bool, altref: bool },
}

type Bits<'a> = (&'a [u8], usize);

fn take_bool(data: Bits) -> nom::IResult<Bits, bool> {
    let (data, b): (_, u8) = nom::bits::complete::take(1usize)(data)?;

    Ok((data, b == 1))
}

fn skip_opt_field(data: Bits, field_size: usize) -> nom::IResult<Bits, ()> {
    let (data, present) = take_bool(data)?;
    Ok((
        if present {
            let (data, _skip): (_, u8) = nom::bits::complete::take(field_size)(data)?;
            data
        } else {
            data
        },
        (),
    ))
}

impl FrameHeader {
    pub fn parse(frame_type: FrameTagType, data: &[u8]) -> nom::IResult<Bits, Self> {
        use nom::bits::complete::take;
        let data = (data, 0usize);

        let data = if frame_type.is_key_frame() {
            let (data, _color_space) = take_bool(data)?;
            let (data, _clamping_type) = take_bool(data)?;

            data
        } else {
            data
        };

        let (data, segmentation_enabled) = take_bool(data)?;
        let data = if segmentation_enabled {
            let (data, update_mb_segmentation_map) = take_bool(data)?;
            let (data, update_segment_feature_data) = take_bool(data)?;
            let data = if update_segment_feature_data {
                let (data, _segment_feature_mode) = take_bool(data)?;
                let mut data = data;
                for _ in 0..4 {
                    data = skip_opt_field(data, 8)?.0;
                }

                for _ in 0..4 {
                    data = skip_opt_field(data, 7)?.0;
                }

                data
            } else {
                data
            };

            let data = if update_mb_segmentation_map {
                let mut data = data;
                for _ in 0..3 {
                    data = skip_opt_field(data, 8)?.0;
                }
                data
            } else {
                data
            };

            data
        } else {
            data
        };

        let (data, _filter_type) = take_bool(data)?;
        let (data, _loop_filter_level): (_, u8) = take(6usize)(data)?;
        let (data, _sharpness_level): (_, u8) = take(3usize)(data)?;

        let (data, loop_filter_adj_enable) = take_bool(data)?;
        let data = if loop_filter_adj_enable {
            let (mut data, mode_ref_lf_delta_update) = take_bool(data)?;
            if mode_ref_lf_delta_update {
                for _ in 0..4 {
                    data = skip_opt_field(data, 7)?.0;
                }
                for _ in 0..4 {
                    data = skip_opt_field(data, 7)?.0;
                }
            }
            data
        } else {
            data
        };

        let (data, _log2_nbr_of_dct_partitions): (_, u8) = take(2usize)(data)?;
        let (data, _y_ac_qi): (_, u8) = take(7usize)(data)?;
        let mut data = data;
        for _ in 0..5 {
            data = skip_opt_field(data, 5)?.0;
        }

        let (data, frame_buffer_update) = if frame_type.is_key_frame() {
            let (data, _refresh_entropy_probs) = take_bool(data)?;
            (data, FrameBufferUpdate::KeyFrame)
        } else {
            let (data, refresh_golden) = take_bool(data)?;
            let (data, refresh_altref) = take_bool(data)?;
            //  TODO: more fields here

            (
                data,
                FrameBufferUpdate::InterFrame {
                    golden: refresh_golden,
                    altref: refresh_altref,
                },
            )
        };

        Ok((
            data,
            Self {
                frame_buffer_update,
            },
        ))
    }
}
