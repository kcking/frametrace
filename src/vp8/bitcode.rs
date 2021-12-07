/// Arithmetic bit-coding from https://datatracker.ietf.org/doc/html/rfc6386#section-7
use std::io::Read;

pub struct BoolEncoder {
    output: Vec<u8>,
    range: u32,
    bottom: u32,
    bit_count: i32,
}

impl BoolEncoder {
    pub fn new() -> Self {
        Self {
            output: vec![],
            range: 255,
            bottom: 0,
            bit_count: 24,
        }
    }

    fn add_one_to_output(&mut self) {
        let mut iter = self.output.iter_mut().rev();

        while let Some(b) = iter.next() {
            if b == &255 {
                *b = 0;
            } else {
                *b += 1;
                break;
            }
        }
    }

    pub fn write_bool(&mut self, prob: u32, value: u8) {
        let split: u32 = 1 + (((self.range - 1) * prob) >> 8);

        if value != 0 {
            self.bottom += split;
            self.range -= split;
        } else {
            self.range = split;
        }

        while self.range < 128 {
            self.range <<= 1;

            if (self.bottom & (1 << 31)) != 0 {
                self.add_one_to_output();
            }

            self.bottom <<= 1; /* before shifting bottom */

            self.bit_count -= 1;
            if self.bit_count == 0 {
                /* write out high byte of bottom ... */

                self.output.push((self.bottom >> 24) as u8);

                self.bottom &= (1 << 24) - 1; /* ... keeping low 3 bytes */

                self.bit_count = 8; /* 8 shifts until next output */
            }
        }
    }

    pub fn flush(mut self) -> Vec<u8> {
        let mut c: i32 = self.bit_count;
        let mut v = self.bottom;

        if (v & (1 << (32 - c))) != 0
        /* propagate (unlikely) carry */
        {
            self.add_one_to_output();
        }
        v <<= c & 7; /* before shifting remaining output */
        c >>= 3; /* to top of internal buffer */
        loop {
            c -= 1;
            if c >= 0 {
                v <<= 8;
            } else {
                break;
            }
        }

        c = 4;
        loop {
            c -= 1;
            if c >= 0 {
                self.output.push((v >> 24) as u8);
                v <<= 8;
            } else {
                break;
            }
        }

        self.output
    }
}

pub struct BoolDecoder<'a> {
    input: std::io::Cursor<&'a [u8]>,
    range: u32,
    value: u32,
    bit_count: i32,
}

impl<'a> BoolDecoder<'a> {
    pub fn new(input: &'a [u8]) -> std::io::Result<Self> {
        let mut input = std::io::Cursor::new(input);

        let mut first_2 = [0u8; 2];
        input.read_exact(&mut first_2)?;

        Ok(Self {
            input,
            value: ((first_2[0] as u32) << 8) | first_2[1] as u32,
            range: 255,
            bit_count: 0,
        })
    }

    pub fn read_bit(&mut self) -> std::io::Result<bool> {
        self.read_bool(128)
    }

    #[allow(non_snake_case)]
    pub fn read_bool(&mut self, prob: u32) -> std::io::Result<bool> {
        let split: u32 = 1 + (((self.range - 1) * prob) >> 8);
        let SPLIT: u32 = split << 8;

        let retval = if self.value >= SPLIT {
            /* encoded a one */
            self.range -= split; /* reduce range */
            self.value -= SPLIT; /* subtract off left endpoint of interval */
            1
        } else {
            /* encoded a zero */
            self.range = split; /* reduce range, no change in left endpoint */
            0
        };

        while self.range < 128 {
            /* shift out irrelevant value bits */
            self.value <<= 1;
            self.range <<= 1;
            self.bit_count += 1;
            if self.bit_count == 8 {
                /* shift in new bits 8 at a time */
                self.bit_count = 0;
                let mut next_byte = [0u8; 1];
                self.input.read_exact(&mut next_byte)?;
                self.value |= next_byte[0] as u32;
            }
        }
        Ok(retval != 0)
    }

    pub fn read_literal(&mut self, num_bits: u32) -> std::io::Result<u32> {
        let mut v: u32 = 0;

        for _ in 0..num_bits {
            v = (v << 1) + self.read_bool(128)? as u32;
        }

        Ok(v)
    }

    pub fn read_signed_literal(&mut self, num_bits: u32) -> std::io::Result<i32> {
        let mut v: i32 = 0;
        if num_bits == 0 {
            return Ok(0);
        }
        if self.read_bool(128)? {
            v = -1;
        }
        for _ in 0..num_bits {
            v = (v << 1) + self.read_bool(128)? as i32;
        }
        Ok(v)
    }
}

#[test]
fn roundtrip() {
    let d = vec![0u8, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0, 1]
        .into_iter()
        .map(|b| b != 0)
        .collect::<Vec<_>>();

    let mut encoder = BoolEncoder::new();
    for d in d.iter() {
        encoder.write_bool(128, *d as u8);
    }

    let out = encoder.flush();

    let mut decoder = BoolDecoder::new(&out).unwrap();

    let mut decoded = vec![];
    for _ in 0..d.len() {
        decoded.push(decoder.read_bool(128).unwrap());
    }

    assert_eq!(d, decoded);
}
