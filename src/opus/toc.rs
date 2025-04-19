#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeMode {
    CELT,
    SILK,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bandwidth {
    Narrow = 8000,
    Medium = 12000,
    Wide = 16000,
    SuperWide = 24000,
    Full = 48000,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameCode {
    Single,
    DoubleCBR,
    DoubleVBR,
    Multiple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameDuration {
    /// 2.5ms
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
#[derive(Debug, Clone, Copy)]
pub struct TableOfContents {
    pub mode: EncodeMode,
    pub bandwidth: Bandwidth,
    pub duration: FrameDuration,
    pub code: FrameCode,
}

impl TableOfContents {
    pub fn decode(toc: u8) -> Self {
        let code = match toc & 0b00000011 {
            0b00 => FrameCode::Single,
            0b01 => FrameCode::DoubleCBR,
            0b10 => FrameCode::DoubleVBR,
            0b11 => FrameCode::Multiple,
            _ => unreachable!(),
        };

        match (toc >> 3) & 0b0001_1111 {
            0 => Self {
                mode: EncodeMode::SILK,
                bandwidth: Bandwidth::Narrow,
                duration: FrameDuration::Medium,
                code,
            },
            1 => Self {
                mode: EncodeMode::SILK,
                bandwidth: Bandwidth::Narrow,
                duration: FrameDuration::Standard,
                code,
            },
            2 => Self {
                mode: EncodeMode::SILK,
                bandwidth: Bandwidth::Narrow,
                duration: FrameDuration::Long,
                code,
            },
            3 => Self {
                mode: EncodeMode::SILK,
                bandwidth: Bandwidth::Narrow,
                duration: FrameDuration::VeryLong,
                code,
            },
            4 => Self {
                mode: EncodeMode::SILK,
                bandwidth: Bandwidth::Medium,
                duration: FrameDuration::Medium,
                code,
            },
            5 => Self {
                mode: EncodeMode::SILK,
                bandwidth: Bandwidth::Medium,
                duration: FrameDuration::Standard,
                code,
            },
            6 => Self {
                mode: EncodeMode::SILK,
                bandwidth: Bandwidth::Medium,
                duration: FrameDuration::Long,
                code,
            },
            7 => Self {
                mode: EncodeMode::SILK,
                bandwidth: Bandwidth::Medium,
                duration: FrameDuration::VeryLong,
                code,
            },
            8 => Self {
                mode: EncodeMode::SILK,
                bandwidth: Bandwidth::Wide,
                duration: FrameDuration::Medium,
                code,
            },
            9 => Self {
                mode: EncodeMode::SILK,
                bandwidth: Bandwidth::Wide,
                duration: FrameDuration::Standard,
                code,
            },
            10 => Self {
                mode: EncodeMode::SILK,
                bandwidth: Bandwidth::Wide,
                duration: FrameDuration::Long,
                code,
            },
            11 => Self {
                mode: EncodeMode::SILK,
                bandwidth: Bandwidth::Wide,
                duration: FrameDuration::VeryLong,
                code,
            },
            12 => Self {
                mode: EncodeMode::Hybrid,
                bandwidth: Bandwidth::SuperWide,
                duration: FrameDuration::Medium,
                code,
            },
            13 => Self {
                mode: EncodeMode::Hybrid,
                bandwidth: Bandwidth::SuperWide,
                duration: FrameDuration::Standard,
                code,
            },
            14 => Self {
                mode: EncodeMode::Hybrid,
                bandwidth: Bandwidth::Full,
                duration: FrameDuration::Medium,
                code,
            },
            15 => Self {
                mode: EncodeMode::Hybrid,
                bandwidth: Bandwidth::Full,
                duration: FrameDuration::Standard,
                code,
            },
            16 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::Narrow,
                duration: FrameDuration::VeryShort,
                code,
            },
            17 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::Narrow,
                duration: FrameDuration::Short,
                code,
            },
            18 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::Narrow,
                duration: FrameDuration::Medium,
                code,
            },
            19 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::Narrow,
                duration: FrameDuration::Standard,
                code,
            },
            20 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::Wide,
                duration: FrameDuration::VeryShort,
                code,
            },
            21 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::Wide,
                duration: FrameDuration::Short,
                code,
            },
            22 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::Wide,
                duration: FrameDuration::Medium,
                code,
            },
            23 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::Wide,
                duration: FrameDuration::Standard,
                code,
            },
            24 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::SuperWide,
                duration: FrameDuration::VeryShort,
                code,
            },
            25 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::SuperWide,
                duration: FrameDuration::Short,
                code,
            },
            26 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::SuperWide,
                duration: FrameDuration::Medium,
                code,
            },
            27 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::SuperWide,
                duration: FrameDuration::Standard,
                code,
            },
            28 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::Full,
                duration: FrameDuration::VeryShort,
                code,
            },
            29 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::Full,
                duration: FrameDuration::Short,
                code,
            },
            30 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::Full,
                duration: FrameDuration::Medium,
                code,
            },
            31 => Self {
                mode: EncodeMode::CELT,
                bandwidth: Bandwidth::Full,
                duration: FrameDuration::Standard,
                code,
            },
            _ => unreachable!(),
        }
    }
}
