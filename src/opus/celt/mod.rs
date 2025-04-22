mod filter;

use crate::opus::entropy::CeltRangeCoding;

use self::filter::PostFilter;

use super::{
    entropy::RangeCodingDecoder,
    toc::{Bandwidth, Channel, EncodeMode, TableOfContents},
};

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

#[derive(Debug, Default)]
pub struct CeltBlock {
    pub post_filter: PostFilter,
    pub energy: [f32; CeltFrame::MAX_BANDS],
    pub lin_energy: [f32; CeltFrame::MAX_BANDS],
    pub prev_energy: [f32; CeltFrame::MAX_BANDS],
}

#[derive(Debug, Default)]
pub struct CeltFrame {
    pub block: [CeltBlock; 2],
    pub blocks: usize,
    pub block_size: usize,
    pub has_silence: bool,
    pub transient: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeltFrameDecodeError {
    BandsOverflow,
}

impl CeltFrame {
    const MAX_BANDS: usize = 21;
    const SHORT_BLOCKSIZE: usize = 120;

    pub fn decode(
        &mut self,
        toc: &TableOfContents,
        range_dec: &mut RangeCodingDecoder,
    ) -> Result<Self, CeltFrameDecodeError> {
        // In Hybrid mode, the Opus encoder has to deal specifically with the
        // high frequency part, so it chooses to start at band 17. In other
        // encoding modes, it starts from 0, covering lower frequency bands.
        let band_range = if toc.mode == EncodeMode::Hybrid {
            17
        } else {
            0
        }..toc.bandwidth.band();

        if band_range.end > Self::MAX_BANDS {
            return Err(CeltFrameDecodeError::BandsOverflow);
        }

        // The mdct block is usually divided into several powers of 2 lengths,
        // first by calculating the length of the basic block, and then by
        // calculating the length of the mdct block from the basic block length.
        let mdct_block_dur = (toc.duration as usize / Self::SHORT_BLOCKSIZE).ilog2();

        // Whether or not there are any bits left in the decoder buffer to read,
        // if there are none then the whole frame is silent. If there are no bits
        // left to read, the frame is silent. If there are no bits left to read,
        // the frame is empty or the packet is lost.
        self.has_silence = if range_dec.available() > 0 {
            // Because little mute is uncommon in audio, it is encoded here as a
            // probability, indicating a large probability that it is not muted
            // and only a small probability that it is.
            range_dec.logp(15)
        } else {
            true
        };

        if self.has_silence {
            range_dec.to_end();
        }

        // Parsing post-filter
        //
        // In the case of the low-frequency part, at least 16 bits of
        // available bytes are needed to decode it.
        if band_range.start == 0 && range_dec.available() >= 16 {
            let has_postfilter = range_dec.logp(1);
            if has_postfilter {
                let post_filter = PostFilter::decode(range_dec);

                self.block.iter_mut().for_each(|it| {
                    it.post_filter = post_filter;
                });
            }
        }

        // The "transient" flag indicates whether the frame uses a single long
        // MDCT or several short MDCTs. When it is set, then the MDCT coefficients
        // represent multiple short MDCTs in the frame. When not set, the
        // coefficients represent a single long MDCT for the frame. The flag is
        // encoded in the bitstream with a probability of 1/8.
        self.transient = if mdct_block_dur > 0 && range_dec.available() >= 3 {
            range_dec.logp(3)
        } else {
            false
        };

        self.blocks = if self.transient { 1 << mdct_block_dur } else { 1 } as usize;
        self.block_size = toc.duration as usize / self.blocks;

        if toc.channel == Channel::Mono {
            for i in 0..Self::MAX_BANDS {
                self.block[0].energy[i] = self.block[0].energy[i].max(self.block[1].energy[i]);
            }
        }

        // coarse energy
        {
            
        }

        todo!()
    }
}
