pub struct FrameInfo {
    pub tag: FrameTag,
    pub header: FrameHeader,
}

impl FrameInfo {
    pub fn parse(data: &[u8]) -> anyhow::Result<Self> {
        let (data, tag) = FrameTag::parse(data)
            .map_err(|e| anyhow::anyhow!("error parsing header {:?}", e.map(|e| e.code)))?;
        let header = FrameHeader::parse(tag.frame_type.clone(), data)?;

        Ok(Self { tag, header })
    }
}

#[derive(Debug)]
pub struct FrameTag {
    pub frame_type: FrameTagType,
    pub version: u8, // 3 bits
    pub show_frame: bool,
    pub first_part_size: u32, // 19 bits
}

use nom::number::complete::{le_u16, le_u24};

use self::bitcode::BoolDecoder;

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
    pub fn resolution(&self) -> Option<(u32, u32)> {
        match self {
            FrameTagType::KeyFrame {
                width,
                width_scale,
                height,
                height_scale,
            } => Some((
                scale_dimension(*width as u32, *width_scale),
                scale_dimension(*height as u32, *height_scale),
            )),
            FrameTagType::InterFrame => None,
        }
    }
}

fn scale_dimension(dimension: u32, scale_flag: u8) -> u32 {
    match scale_flag {
        1 => dimension * 5 / 4,
        2 => dimension * 5 / 3,
        3 => dimension * 2,
        _ => dimension,
    }
}

///  https://datatracker.ietf.org/doc/html/rfc6386#section-19.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameHeader {
    pub frame_buffer_update: FrameBufferUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameBufferUpdate {
    pub golden: bool,
    pub altref: bool,
}

fn skip_opt_field(decoder: &mut BoolDecoder, field_size: usize) -> std::io::Result<()> {
    if decoder.read_bit()? {
        let _skip = decoder.read_literal(field_size as u32)?;
    }
    Ok(())
}

impl FrameHeader {
    pub fn parse(frame_type: FrameTagType, data: &[u8]) -> std::io::Result<Self> {
        let mut decoder = bitcode::BoolDecoder::new(data)?;

        if frame_type.is_key_frame() {
            let _color_space = decoder.read_bit()?;
            if _color_space {
                panic!("unsupported color space");
            }
            let _clamping_type = decoder.read_bit()?;
        };

        let segmentation_enabled = decoder.read_bit()?;
        if segmentation_enabled {
            let update_mb_segmentation_map = decoder.read_bit()?;
            let update_segment_feature_data = decoder.read_bit()?;
            if update_segment_feature_data {
                let _segment_feature_mode = decoder.read_bit()?;
                for _ in 0..4 {
                    skip_opt_field(&mut decoder, 8)?;
                }

                for _ in 0..4 {
                    skip_opt_field(&mut decoder, 7)?;
                }
            }

            if update_mb_segmentation_map {
                for _ in 0..3 {
                    skip_opt_field(&mut decoder, 8)?;
                }
            }
        }

        let _filter_type = decoder.read_bit()?;
        let _loop_filter_level = decoder.read_literal(6)?;
        let _sharpness_level = decoder.read_literal(3)?;

        let loop_filter_adj_enable = decoder.read_bit()?;
        if loop_filter_adj_enable {
            let mode_ref_lf_delta_update = decoder.read_bit()?;
            if mode_ref_lf_delta_update {
                for _ in 0..4 {
                    skip_opt_field(&mut decoder, 7)?;
                }
                for _ in 0..4 {
                    skip_opt_field(&mut decoder, 7)?;
                }
            }
        }

        let _log2_nbr_of_dct_partitions = decoder.read_literal(2)?;
        let _y_ac_qi = decoder.read_literal(7)?;
        for _ in 0..5 {
            skip_opt_field(&mut decoder, 5)?;
        }

        let frame_buffer_update = if frame_type.is_key_frame() {
            let _refresh_entropy_probs = decoder.read_bit()?;
            FrameBufferUpdate {
                golden: true,
                altref: true,
            }
        } else {
            let refresh_golden = decoder.read_bit()?;
            let refresh_altref = decoder.read_bit()?;
            //  TODO: more fields here

            FrameBufferUpdate {
                golden: refresh_golden,
                altref: refresh_altref,
            }
        };

        Ok(Self {
            frame_buffer_update,
        })
    }
}

mod bitcode;
#[cfg(test)]
mod testing;
