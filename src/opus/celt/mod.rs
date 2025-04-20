use super::toc::{Bandwidth, EncodeMode, TableOfContents};

pub mod mdct;

pub trait CeltBandwidthBand {
    fn band(&self) -> usize;
}

impl CeltBandwidthBand for Bandwidth {
    fn band(&self) -> usize {
        match self {
            Self::Narrow => 13,
            Self::Medium => 17,
            Self::Wide => 17,
            Self::SuperWide => 19,
            Self::Full => 21,
        }
    }
}

pub struct CeltFrame {}

#[derive(Debug)]
pub enum CeltFrameDecodeError {
    BandsOverflow,
}

impl CeltFrame {
    const MAX_BANDS: usize = 21;
    const SHORT_BLOCKSIZE: usize = 120;

    pub fn decode(toc: &TableOfContents) -> Result<Self, CeltFrameDecodeError> {
        let range = if toc.mode == EncodeMode::Hybrid {
            17
        } else {
            0
        }..toc.bandwidth.band();

        if range.end > Self::MAX_BANDS {
            return Err(CeltFrameDecodeError::BandsOverflow);
        }

        let mdct_block_dur = (toc.duration as usize / Self::SHORT_BLOCKSIZE).ilog2();

        todo!()
    }
}
