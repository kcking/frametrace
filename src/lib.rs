use std::io::Write;

use rtp::{codecs::vp8::Vp8Packet, packetizer::Depacketizer};
use serde::Serialize;
use webrtc_util::Unmarshal;

pub mod vp8;

/// Structure of a video log line (json)
#[derive(Serialize)]
pub struct FrameLogLine {
    pub rtp_sequence_number: u16,
    pub pts: u32,
    pub picture_id: Option<u16>,
    //  only set on keyframes, could remember last frame's resolution but then dropped frames wouldn't be accounted for
    pub resolution: Option<(u32, u32)>,
    pub show_frame: bool,
    pub keyframe: bool,
    pub modify_golden_frame: bool,
    pub modify_altref_frame: bool,
}

/// Handles parsing RTP packets down through VP8 compressed frame header
pub struct RtpVp8FrameInfo {
    rtp_header: rtp::header::Header,
    vp8_rtp_header: Vp8Packet,
    vp8_frame: vp8::FrameInfo,
}

impl RtpVp8FrameInfo {
    pub fn parse(mut pkt: &[u8]) -> anyhow::Result<Option<Self>> {
        let rtp_packet = rtp::packet::Packet::unmarshal(&mut pkt)?;

        let mut vp8_pkt = Vp8Packet::default();
        let vp8_frame = vp8_pkt.depacketize(&bytes::Bytes::from(rtp_packet.payload.to_vec()))?;

        if vp8_pkt.s == 1 && vp8_pkt.pid == 0 {
            let frame_info = vp8::FrameInfo::parse(&vp8_frame)?;
            Ok(Some(Self {
                vp8_rtp_header: vp8_pkt,
                rtp_header: rtp_packet.header,
                vp8_frame: frame_info,
            }))
        } else {
            return Ok(None);
        }
    }

    pub fn to_log_line(&self) -> FrameLogLine {
        FrameLogLine {
            rtp_sequence_number: self.rtp_header.sequence_number,
            pts: self.rtp_header.timestamp,
            picture_id: if self.vp8_rtp_header.i == 1 {
                Some(self.vp8_rtp_header.picture_id)
            } else {
                None
            },
            resolution: self.vp8_frame.tag.frame_type.resolution(),
            show_frame: self.vp8_frame.tag.show_frame,
            keyframe: self.vp8_frame.tag.frame_type.is_key_frame(),
            modify_golden_frame: self.vp8_frame.header.frame_buffer_update.golden,
            modify_altref_frame: self.vp8_frame.header.frame_buffer_update.altref,
        }
    }
}

/// Spawns a thread that listens to the returned Sender, writing logs to the provided `w`.
pub fn spawn_rtp_logger<W: Write + Send + Sync + 'static>(
    mut w: W,
) -> std::sync::mpsc::SyncSender<Vec<u8>> {
    let (tx, rx) = sync_channel(128);

    std::thread::spawn(move || {
        while let Ok(rtp_pkt) = rx.recv() {
            match RtpVp8FrameInfo::parse(&rtp_pkt) {
                Ok(Some(info)) => {
                    if let Ok(mut json) = serde_json::to_vec(&info.to_log_line()) {
                        json.push(b'\n');
                        match w.write_all(&json) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("error writing to log {:?}", e);
                                return;
                            }
                        }
                    }
                }
                Ok(Option::None) => {}
                Err(e) => {
                    eprintln!("error parsing rtp packet: {:?}", e);
                }
            }
        }
    });

    tx
}
