#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum EncodeMode {
    #[default]
    CELT,
    SILK,
    Hybrid,
}

impl From<u8> for EncodeMode {
    fn from(value: u8) -> Self {
        match (value >> 3) & 0x1F {
            0..=11 => Self::SILK,
            12..=15 => Self::Hybrid,
            16..=31 => Self::CELT,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Bandwidth {
    #[default]
    Narrow = 8000,
    Medium = 12000,
    Wide = 16000,
    SuperWide = 24000,
    Full = 48000,
}

impl From<u8> for Bandwidth {
    fn from(value: u8) -> Self {
        match (value >> 3) & 0x1F {
            4..=7 => Self::Medium,
            0..=3 | 16..=19 => Self::Narrow,
            8..=11 | 20..=23 => Self::Wide,
            14..=15 | 28..=31 => Self::Full,
            12..=13 | 24..=27 => Self::SuperWide,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FrameCode {
    #[default]
    Single,
    DoubleCBR,
    DoubleVBR,
    Multiple,
}

impl From<u8> for FrameCode {
    fn from(value: u8) -> Self {
        match value & 0x03 {
            0 => FrameCode::Single,
            1 => FrameCode::DoubleCBR,
            2 => FrameCode::DoubleVBR,
            3 => FrameCode::Multiple,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FrameDuration {
    /// 2.5ms
    #[default]
    VeryShort = 120,
    /// 5ms
    Short = 240,
    /// 10ms
    Medium = 480,
    /// 20ms
    Standard = 960,
    /// 40ms,
    Long = 1920,
    /// 60ms
    VeryLong = 2880,
}

impl From<u8> for FrameDuration {
    fn from(value: u8) -> Self {
        match (value >> 3) & 0x1F {
            v @ 0..=11 => match v & 0b11 {
                0 => Self::Medium,
                1 => Self::Standard,
                2 => Self::Long,
                3 => Self::VeryLong,
                _ => unreachable!(),
            },
            v @ 12..=15 => match v & 0b1 {
                0 => Self::Medium,
                1 => Self::Standard,
                _ => unreachable!(),
            },
            v @ 16..=31 => match v & 0b11 {
                0 => Self::VeryShort,
                1 => Self::Short,
                2 => Self::Medium,
                3 => Self::Standard,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    #[default]
    Mono = 1,
    Stereo = 2,
}

impl From<u8> for Channel {
    fn from(value: u8) -> Self {
        if value >> 2 & 0x01 == 1 {
            Self::Stereo
        } else {
            Self::Mono
        }
    }
}

/// Each opus packet begins with a TOC byte.
///
/// ```text
///  0 1 2 3 4 5 6 7
/// +-+-+-+-+-+-+-+-+
/// | config  |s| c |
/// +-+-+-+-+-+-+-+-+
/// ```
///
/// The first 5 bits of the configuration number define 32 encoding
/// configurations with different encoding modes, audio bandwidths and frame
/// lengths, as shown in the table below:
///
/// | config  | mode        | bandwidth | duration          |
/// |---------|-------------|-----------|-------------------|
/// | 0...3	  | SILK-only   | NB        | 10, 20, 40, 60 ms |
/// | 4...7	  | SILK-only   | MB        | 10, 20, 40, 60 ms |
/// | 8...11  | SILK-only   | WB        | 10, 20, 40, 60 ms |
/// | 12...13 | Hybrid      | SWB       | 10, 20 ms         |
/// | 14...15 | Hybrid      | FB        | 10, 20 ms         |
/// | 16...19 | CELT-only   | NB        | 2.5, 5, 10, 20 ms |
/// | 20...23 | CELT-only   | WB        | 2.5, 5, 10, 20 ms |
/// | 24...27 | CELT-only   | SWB       | 2.5, 5, 10, 20 ms |
/// | 28...31 | CELT-only   | FB        | 2.5, 5, 10, 20 ms |
///
/// The stereo flag bit (s) takes the value 0 for mono, 1 for multi-channel
/// stereo.
///
/// The last two digits (c) in the TOC indicate:
///
/// * There is only one frame of audio in a packet.
/// * There are two frames of audio in a packet and they are of the same size.
/// * There are two frames of audio in a packet, but of different sizes.
/// * There are any frames in a packet.
///
#[derive(Debug, Default, Clone, Copy)]
pub struct TableOfContents {
    pub mode: EncodeMode,
    pub bandwidth: Bandwidth,
    pub duration: FrameDuration,
    pub channel: Channel,
    pub code: FrameCode,
}

impl From<u8> for TableOfContents {
    fn from(value: u8) -> Self {
        Self {
            mode: EncodeMode::from(value),
            bandwidth: Bandwidth::from(value),
            duration: FrameDuration::from(value),
            channel: Channel::from(value),
            code: FrameCode::from(value),
        }
    }
}
