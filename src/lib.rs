#[derive(Debug)]
pub struct FrameTag {
    frame_type: FrameTagType,
    version: u8, // 3 bits
    show_frame: bool,
    first_part_size: u32, // 19 bits
}

use bitvec::prelude::*;
use nom_bitvec::BSlice;

type Bits<'a> = BSlice<'a, Msb0, u8>;
use nom::bytes::complete::{tag, take};

impl FrameTag {
    pub fn parse(data: Bits) -> nom::IResult<Bits, Self> {
        let first_24 = take(24usize)(data)?.1;
        dbg!(first_24, first_24.0.get(0));

        // panic!("done");
        //  TODO: endianness?
        let (data, keyframe_bit) = take(1usize)(data)?;

        let (data, version) = take(3usize)(data)?;

        let (data, show_frame) = take(1usize)(data)?;

        let (data, first_part_size) = take(19usize)(data)?;

        let (data, frame_type) = if keyframe_bit.0.get(0).map(|r| *r).unwrap_or(false) {
            let (data2, start_code) = take(24usize)(data)?;
            let expected_start_code = vec![0x9du8, 0x01, 0x2a]
                .into_iter()
                .rev()
                .collect::<Vec<_>>();
            let start_tag = BitSlice::<Lsb0, u8>::from_slice(&expected_start_code).unwrap();
            dbg!(start_code);
            dbg!(start_tag);
            let (data, _start_code) = tag(BSlice(start_tag))(data)?;
            let (data, width) = take(14usize)(data)?;
            let (data, width_scale) = take(2usize)(data)?;
            let (data, height) = take(14usize)(data)?;
            let (data, height_scale) = take(2usize)(data)?;
            (
                data,
                FrameTagType::KeyFrame {
                    width: width.0.load_le(),
                    width_scale: width_scale.0.load_le(),
                    height: height.0.load_le(),
                    height_scale: height_scale.0.load_le(),
                },
            )
        } else {
            (data, FrameTagType::InterFrame)
        };

        Ok((
            data,
            Self {
                version: version.0.load_le(),
                show_frame: show_frame.0.get(0).map(|r| *r).unwrap_or(false),
                first_part_size: first_part_size.0.load_le(),
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
