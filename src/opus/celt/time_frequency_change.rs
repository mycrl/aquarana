use crate::opus::entropy::RangeCodingDecoder;

use super::CeltFrameDecoder;

/// Time-frequency conversion selection table
/// 
/// This table defines the time-frequency conversion parameters for different 
/// frame sizes.
/// 
/// Structure is [frame size][transient or not][select bit][transform flag].
/// 
/// Frame size index: 0=120, 1=240, 2=480, 3=960
/// 
/// Each value represents the offset of the time-frequency transform.
pub const TF_SELECT: [[[[i8; 2]; 2]; 2]; 4] = [
    // 120
    [[[0, -1], [0, -1]], [[0, -1], [0, -1]]],
    // 240
    [[[0, -1], [0, -2]], [[1, 0], [1, -1]]],
    // 480
    [[[0, -2], [0, -3]], [[2, 0], [1, -1]]],
    // 960
    [[[0, -2], [0, -3]], [[3, 0], [1, -1]]],
];

/// Time-frequency conversion parameter decoder
/// 
/// Used to control the time-frequency resolution trade-off for each coding band
/// 
/// Main functions:
/// 
/// - decodes the time-conversion flag
/// - Selects different transform parameters depending on the frame type (transient/non-transient).
/// - Apply time-frequency transforms to each band.
pub struct TimeFrequencyChange;

impl TimeFrequencyChange {
    pub fn decode(dec: &mut CeltFrameDecoder, range_dec: &mut RangeCodingDecoder) {
        // Determine the initial number of bits: 2 bits for transient frames 
        // and 4 bits for non-transient frames.
        let mut bits = if dec.transient { 2 } else { 4 };
        // Check if there are enough bits for the selection bit
        let select_bit = dec.size != 0 && range_dec.available() > bits;

        let mut diff = false;
        let mut change = false;
        // Iterate through all bands to decode the time-frequency transform flag
        for i in dec.band_range.clone() {
            if range_dec.available() > bits + select_bit as usize {
                diff ^= range_dec.logp(bits);
                change |= diff;
            }

            dec.time_frequency_change[i] = diff as i32;
            // Update the number of bits: 4 bits for transient frames 
            // and 5 bits for non-transient frames.
            bits = if dec.transient { 4 } else { 5 };
        }

        let change = change as usize;
        // Select the transform parameter table based on the frame size and transient state
        let tf_select = TF_SELECT[dec.size][if dec.transient { 1 } else { 0 }];
        // Decode the selection bit
        let select = if select_bit && tf_select[0][change] != tf_select[1][change] {
            range_dec.logp(1) as usize
        } else {
            0
        };

        // Apply time-frequency transforms to each band
        for i in dec.band_range.clone() {
            dec.time_frequency_change[i] =
                tf_select[select][dec.time_frequency_change[i] as usize] as i32;
        }
    }
}
