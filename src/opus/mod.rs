pub mod celt;
pub mod entropy;
pub mod toc;

use bytes::Buf;
use celt::CeltFrameDecodeError;
use entropy::RangeCodingDecoder;

use self::toc::{EncodeMode, FrameCode, TableOfContents};

#[derive(Debug)]
pub struct OpusFrame {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpusFrameDecoderError {
    Celt(CeltFrameDecodeError),
}

impl From<CeltFrameDecodeError> for OpusFrameDecoderError {
    fn from(value: CeltFrameDecodeError) -> Self {
        Self::Celt(value)
    }
}

impl OpusFrame {
    pub fn deocde(toc: &TableOfContents, bytes: &[u8]) -> Result<Self, OpusFrameDecoderError> {
        let mut range_dec = RangeCodingDecoder::new(bytes);

        let consumed = range_dec.tell();
        let has_redundancy = if toc.mode == EncodeMode::Hybrid && consumed + 37 <= bytes.len() * 8 {
            range_dec.logp(12)
        } else if toc.mode == EncodeMode::SILK && consumed + 17 <= bytes.len() * 8 {
            true
        } else {
            false
        };

        if has_redundancy {
            todo!("skip redundancy info");
        }

        if toc.mode == EncodeMode::CELT {
            // CeltFrame::default().decode(toc, &mut range_dec)?;
        } else {
            todo!("Only CELT is supported");
        }

        Ok(Self {})
    }
}

#[derive(Debug)]
pub struct OpusPacket {
    pub toc: TableOfContents,
    pub frames: Vec<OpusFrame>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpusPacketDecodeError {
    InvalidData,
    FramesOverflow,
    FrameDecodeError(OpusFrameDecoderError),
}

impl From<OpusFrameDecoderError> for OpusPacketDecodeError {
    fn from(value: OpusFrameDecoderError) -> Self {
        Self::FrameDecodeError(value)
    }
}

impl OpusPacket {
    const MAX_FRAME_LEN: usize = 1275;
    const MAX_FRAMES: usize = 48;

    pub fn decode(mut bytes: &[u8]) -> Result<Self, OpusPacketDecodeError> {
        if bytes.len() < 1 {
            return Err(OpusPacketDecodeError::InvalidData);
        }

        let toc = TableOfContents::from(bytes.get_u8());

        let mut datas = Vec::with_capacity(10);
        match toc.code {
            // A packet contains only one frame of audio.
            FrameCode::Single => {
                if bytes.len() <= Self::MAX_FRAME_LEN {
                    datas.push(&bytes[..]);
                }

                bytes.advance(bytes.len());
            }
            // A package contains two frames of the same size.
            FrameCode::DoubleCBR => {
                if bytes.len() & 1 != 1 {
                    return Err(OpusPacketDecodeError::InvalidData);
                }

                // The two frames are each half the size of the remaining bytes
                // of the packet.
                let half = bytes.len() / 2;
                if half <= Self::MAX_FRAME_LEN {
                    datas.push(&bytes[..half]);
                    datas.push(&bytes[half..]);
                }

                bytes.advance(bytes.len());
            }
            // A package contains two frames of different sizes.
            FrameCode::DoubleVBR => {
                // The 1 ~ 2 bytes after the TOC byte are the number of bytes in
                // the first frame.
                let len = read_variable_length(&mut bytes);
                if len > Self::MAX_FRAME_LEN {
                    return Err(OpusPacketDecodeError::InvalidData);
                }

                if len > 0 {
                    datas.push(&bytes[..len]);
                    datas.push(&bytes[len..]);

                    bytes.advance(bytes.len());
                }
            }
            // A packet contains any number of frames.
            FrameCode::Multiple => {
                // This type of packet has a byte encoding the number of frames
                // inside the packet after the TOC byte, the structure of this
                // byte is shown below.
                //
                // ```
                //  0 1 2 3 4 5 6 7
                // +-+-+-+-+-+-+-+-+
                // |v|p|     M     |
                // +-+-+-+-+-+-+-+-+
                // ```
                //
                // - v equals 0 for CBR and 1 for VBR.
                // - p equals 1 for packet containing padding bytes.
                // - M indicates the number of frames contained in the packet.
                let flag = bytes.get_u8();
                let is_vbr = (flag & 0x80) != 0;
                let frame_count = (flag & 0x3F) as usize;
                let has_padding = (flag & 0x40) == 1;

                if frame_count == 0 || frame_count > Self::MAX_FRAMES {
                    return Err(OpusPacketDecodeError::FramesOverflow);
                }

                if has_padding {
                    let mut padding_len = 0;

                    loop {
                        let byte = bytes.get_u8() as usize;
                        if byte > u32::MAX as usize - 255 {
                            return Err(OpusPacketDecodeError::InvalidData);
                        }

                        padding_len += byte;
                        if byte < 255 {
                            break;
                        } else {
                            padding_len -= 1;
                        }
                    }

                    bytes = &bytes[..bytes.len() - padding_len];
                }

                if is_vbr {
                    let mut sizes = Vec::with_capacity(frame_count);
                    for _ in 0..frame_count {
                        let len = read_variable_length(&mut bytes);

                        if len > 0 {
                            sizes.push(len);
                        }
                    }

                    for len in sizes {
                        if len <= Self::MAX_FRAME_LEN {
                            datas.push(&bytes[..len]);
                        }

                        bytes.advance(len);
                    }

                    if bytes.len() <= Self::MAX_FRAME_LEN {
                        datas.push(&bytes[..]);
                    }

                    bytes.advance(bytes.len());
                } else {
                    let len = bytes.len() / frame_count;
                    for _ in 0..frame_count {
                        datas.push(&bytes[..len]);

                        bytes.advance(len);
                    }
                }
            }
        };

        let mut frames = Vec::with_capacity(datas.len());
        for data in datas {
            frames.push(OpusFrame::deocde(&toc, data)?);
        }

        Ok(Self { toc, frames })
    }
}

/// Frame length encoding
///
/// When an audio frame contains more than one VBR, then the length of the frame
/// needs to be encoded for all previous frames except the last one. The
/// encoding of the frame length takes up 1 ~ 2 bytes, the rules are as follows:
///
/// - the first byte takes the value 0: there is no frame data (this is usually
/// a non-sequential transmission (DTX) or a loss of the audio packet)
///
/// - the first byte takes the value 1 ~ 251: it means the number of bytes in
/// the first frame
///
/// - the first byte takes the value 252 ~ 255: the second byte is also involved
/// in the encoding of the frame length, and the total number of bytes in the
/// first frame is Total number of bytes in the first frame is: (second byte * 4) + first byte
///
fn read_variable_length<T: Buf>(bytes: &mut T) -> usize {
    let mut len = bytes.get_u8() as usize;
    if len >= 252 {
        len += 4 * bytes.get_u8() as usize;
    }

    len
}
