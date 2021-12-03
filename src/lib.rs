#[derive(Debug)]
pub struct FrameTag {
    frame_type: FrameTagType,
    version: u8, // 3 bits
    show_frame: bool,
    first_part_size: u32, // 19 bits
}

type Bits<'a> = (&'a [u8], usize);
use nom::bits::complete::{tag, take};

impl FrameTag {
    pub fn parse(data: Bits) -> nom::IResult<Bits, Self> {
        //  TODO: endianness?
        let (data, keyframe_bit): (Bits, u8) = take(1usize)(data)?;

        let (data, version): (Bits, u8) = take(3usize)(data)?;

        let (data, show_frame): (Bits, u8) = take(1usize)(data)?;

        let (data, first_part_size): (Bits, u32) = take(19usize)(data)?;

        let (data, frame_type) = if keyframe_bit == 1 {
            let (data, _start_code): (Bits, u32) = tag(0x9d012a, 24usize)(data)?;
            let (data, width): (Bits, u16) = take(14usize)(data)?;
            let (data, width_scale): (Bits, u8) = take(2usize)(data)?;
            let (data, height): (Bits, u16) = take(14usize)(data)?;
            let (data, height_scale): (Bits, u8) = take(2usize)(data)?;
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
                show_frame: show_frame != 0,
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
