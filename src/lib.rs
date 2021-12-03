#[derive(Debug)]
pub struct FrameTag {
    pub frame_type: FrameTagType,
    pub version: u8, // 3 bits
    pub show_frame: bool,
    pub first_part_size: u32, // 19 bits
}

use nom::{
    bytes::complete::{self, tag, take},
    number::{
        self,
        complete::{be_u16, be_u24, le_u16, le_u24},
    },
};

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

#[derive(Debug)]
pub enum FrameTagType {
    KeyFrame {
        width: u16,       // 14 bits
        width_scale: u8,  // 2 bits
        height: u16,      // 14 bits
        height_scale: u8, // 2 bits
    },
    InterFrame,
}

///  https://datatracker.ietf.org/doc/html/rfc6386#section-19.2
struct Vp8FrameHeader {}
