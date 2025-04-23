use crate::opus::entropy::RangeCodingDecoder;

use super::CeltFrameDecoder;

/// OFF ON - Transient frame
/// OFF ON OFF ON - TF select flag
/// OFF ON OFF ON OFF ON OFF ON - TF change flag
pub const TF_SELECT: [[[[i8; 2]; 2]; 2]; 4] = [
    [[[0, -1], [0, -1]], [[0, -1], [0, -1]]], // 120
    [[[0, -1], [0, -2]], [[1, 0], [1, -1]]],  // 240
    [[[0, -2], [0, -3]], [[2, 0], [1, -1]]],  // 480
    [[[0, -2], [0, -3]], [[3, 0], [1, -1]]],  // 960
];

/// The time-frequency (TF) parameters are used to control the time-frequency
/// resolution trade-off in each coded band.
pub struct TimeFrequencyChange;

impl TimeFrequencyChange {
    pub fn parse(dec: &mut CeltFrameDecoder, range_dec: &mut RangeCodingDecoder) {
        let mut bits = if dec.transient { 2 } else { 4 };
        let select_bit = dec.mdct_block_dur != 0 && range_dec.available() > bits;

        let mut diff = false;
        let mut change = false;
        for i in dec.band_range.clone() {
            if range_dec.available() > bits + select_bit as usize {
                diff ^= range_dec.logp(bits);
                change |= diff;
            }

            dec.time_frequency_change[i] = diff as i8;
            bits = if dec.transient { 4 } else { 5 };
        }

        let change = change as usize;
        let tf_select = TF_SELECT[dec.mdct_block_dur][dec.transient as usize];
        let select = if select_bit && tf_select[0][change] != tf_select[1][change] {
            range_dec.logp(1) as usize
        } else {
            0
        };

        for i in dec.band_range.clone() {
            let it = dec.time_frequency_change[i] as usize;
            dec.time_frequency_change[i] = tf_select[select][it];
        }
    }
}
