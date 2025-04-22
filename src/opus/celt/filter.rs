use crate::opus::entropy::{CeltRangeCoding, RangeCodingDecoder};

#[derive(Debug, Default, Clone, Copy)]
pub struct PostFilter {
    pub period_new: usize,
    pub gains_new: [f32; 3],
}

impl PostFilter {
    const POSTFILTER_MINPERIOD: usize = 15;

    // Tapset Filter coefficients
    const TAPS: [[f32; 3]; 3] = [
        // Tapset zero corresponds to the filter coefficients
        // g0 = 0.3066406250,
        // g1 = 0.2170410156,
        // g2 = 0.1296386719.
        [0.3066406250, 0.2170410156, 0.1296386719],
        // Tapset one corresponds to the filter coefficients
        // g0 = 0.4638671875,
        // g1 = 0.2680664062,
        // g2 = 0.
        [0.4638671875, 0.2680664062, 0.0],
        // tapset two uses filter coefficients
        // g0 = 0.7998046875,
        // g1 = 0.1000976562,
        // g2 = 0.
        [0.7998046875, 0.1000976562, 0.0],
    ];

    pub fn decode(range_dec: &mut RangeCodingDecoder) -> Self {
        // Octaves are decoded as integer values ​​between 0 and 6 with uniform
        // probability.
        let octave = range_dec.uniform(6);

        // Fine pitches within the octave will be decoded using 4+ octave raw bits.
        // The final pitch period is equal to (16<< octave) + fine pitch -1,
        // so it ranges between and including 15 and 1022.
        let period = (16 << octave) + range_dec.rawbits(4 + octave) - 1;

        // The gain is decoded as three raw bits and is equal to G = 3 * (int_gain+1) / 32
        let gain = 0.09375 * (range_dec.rawbits(3) + 1) as f32;

        // The set of post-filter taps is decoded last, using a pdf equal to {2, 1, 1} / 4.
        let tapset = if range_dec.available() >= 2 {
            range_dec.icdf(4, &[2, 3, 4])
        } else {
            0
        };

        Self {
            period_new: period.max(Self::POSTFILTER_MINPERIOD),
            gains_new: {
                let mut gains = [0.0, 0.0, 0.0];
                for i in 0..3 {
                    gains[i] = gain * Self::TAPS[tapset][i];
                }

                gains
            },
        }
    }
}
