pub mod opus;

use bytes::Buf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OggOpusHeadChannelMappingFamily {
    // mono / stereo
    Normal,
    Complex {
        stream_count: u8,
        coupled_count: u8,
        channel_mapping: Vec<u8>,
    },
}

#[derive(Debug, Clone)]
pub struct OggOpusHead {
    pub channel_count: u8,
    pub pre_skip: u16,
    pub input_sample_rate: u32,
    pub output_gain: u16,
    pub channel_mapping_family: OggOpusHeadChannelMappingFamily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OggOpusHeadDecodeError {
    InvalidData,
    NotOpusHead,
    UnexpectedVersionNumber(u8),
}

impl TryFrom<&[u8]> for OggOpusHead {
    type Error = OggOpusHeadDecodeError;

    fn try_from(mut bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 19 {
            return Err(OggOpusHeadDecodeError::InvalidData);
        }

        // magic signature
        if &bytes[..8] != "OpusHead".as_bytes() {
            return Err(OggOpusHeadDecodeError::NotOpusHead);
        } else {
            bytes.advance(8);
        }

        // fixed 0x01
        let version = bytes.get_u8();
        if version != 0x01 {
            return Err(OggOpusHeadDecodeError::UnexpectedVersionNumber(version));
        }

        let channel_count = bytes.get_u8();
        let pre_skip = bytes.get_u16_le();
        let input_sample_rate = bytes.get_u32_le();
        let output_gain = bytes.get_u16();
        let channel_mapping_family = match bytes.get_u8() {
            0 => OggOpusHeadChannelMappingFamily::Normal,
            _ => {
                if bytes.len() < channel_count as usize + 2 {
                    return Err(OggOpusHeadDecodeError::InvalidData);
                }

                OggOpusHeadChannelMappingFamily::Complex {
                    stream_count: bytes.get_u8(),
                    coupled_count: bytes.get_u8(),
                    channel_mapping: bytes[..channel_count as usize].to_vec(),
                }
            }
        };

        Ok(Self {
            channel_count,
            pre_skip,
            input_sample_rate,
            output_gain,
            channel_mapping_family,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OggOpusTags<'a> {
    pub vendor: &'a str,
    pub comments: Vec<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OggOpusTagsDecodeError {
    InvalidData,
    NotOpusTags,
    NonUTF8,
}

impl<'a> TryFrom<&'a [u8]> for OggOpusTags<'a> {
    type Error = OggOpusTagsDecodeError;

    fn try_from(mut bytes: &'a [u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 12 {
            return Err(OggOpusTagsDecodeError::InvalidData);
        }

        // magic signature
        if &bytes[..8] != "OpusTags".as_bytes() {
            return Err(OggOpusTagsDecodeError::NotOpusTags);
        } else {
            bytes.advance(8);
        }

        let vendor_len = bytes.get_u32_le() as usize;
        if bytes.len() < vendor_len {
            return Err(OggOpusTagsDecodeError::InvalidData);
        }

        let vendor = std::str::from_utf8(&bytes[..vendor_len])
            .map_err(|_| OggOpusTagsDecodeError::NonUTF8)?;

        bytes.advance(vendor_len);

        if bytes.len() < 4 {
            return Err(OggOpusTagsDecodeError::InvalidData);
        }

        let comment_count = bytes.get_u32_le() as usize;
        let mut comments = Vec::with_capacity(comment_count);
        for _ in 0..comment_count {
            if bytes.len() < 4 {
                return Err(OggOpusTagsDecodeError::InvalidData);
            }

            let len = bytes.get_u32_le() as usize;
            if bytes.len() < len {
                return Err(OggOpusTagsDecodeError::InvalidData);
            }

            let comment =
                std::str::from_utf8(&bytes[..len]).map_err(|_| OggOpusTagsDecodeError::NonUTF8)?;

            bytes.advance(len);

            comments.push(comment);
        }

        Ok(Self { vendor, comments })
    }
}
