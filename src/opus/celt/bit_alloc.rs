use crate::opus::{
    entropy::RangeCodingDecoder,
    toc::{Channels, TableOfContents},
};

use super::{CeltFrameDecoder, MAX_BANDS};

const VECTORS: usize = 11;

const SPREAD_MODEL_DICT: [usize; 5] = [32, 7, 9, 30, 32];

const STATIC_CAPS: [[[i32; 21]; 2]; 4] = [
    [
        // 120-sample
        [
            224, 224, 224, 224, 224, 224, 224, 224, 160, 160, 160, 160, 185, 185, 185, 178, 178,
            168, 134, 61, 37,
        ],
        [
            224, 224, 224, 224, 224, 224, 224, 224, 240, 240, 240, 240, 207, 207, 207, 198, 198,
            183, 144, 66, 40,
        ],
    ],
    [
        // 240-sample
        [
            160, 160, 160, 160, 160, 160, 160, 160, 185, 185, 185, 185, 193, 193, 193, 183, 183,
            172, 138, 64, 38,
        ],
        [
            240, 240, 240, 240, 240, 240, 240, 240, 207, 207, 207, 207, 204, 204, 204, 193, 193,
            180, 143, 66, 40,
        ],
    ],
    [
        // 480-sample
        [
            185, 185, 185, 185, 185, 185, 185, 185, 193, 193, 193, 193, 193, 193, 193, 183, 183,
            172, 138, 65, 39,
        ],
        [
            207, 207, 207, 207, 207, 207, 207, 207, 204, 204, 204, 204, 201, 201, 201, 188, 188,
            176, 141, 66, 40,
        ],
    ],
    [
        // 960-sample
        [
            193, 193, 193, 193, 193, 193, 193, 193, 193, 193, 193, 193, 194, 194, 194, 184, 184,
            173, 139, 65, 39,
        ],
        [
            204, 204, 204, 204, 204, 204, 204, 204, 201, 201, 201, 201, 198, 198, 198, 187, 187,
            175, 140, 66, 40,
        ],
    ],
];

const LOG_GREQ_RANGE: [i32; 21] = [
    0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8, 16, 16, 16, 21, 21, 24, 29, 34, 36,
];

const FREQ_RANGE: [i32; 21] = [
    1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 6, 6, 8, 12, 18, 22,
];

const ALLOC_TRIM_MODEL: [usize; 12] = [128, 2, 4, 9, 19, 41, 87, 109, 119, 124, 126, 128];

const LOG2_FRAC: [usize; 24] = [
    0, 8, 13, 16, 19, 21, 23, 24, 26, 27, 28, 29, 30, 31, 32, 32, 33, 34, 34, 35, 36, 36, 37, 37,
];

#[derive(Debug)]
pub enum Spread {
    Light,
    Normal,
    Aggressive,
}

pub struct BitAlloc;

impl BitAlloc {
    pub fn decode(
        toc: &TableOfContents,
        dec: &mut CeltFrameDecoder,
        range_dec: &mut RangeCodingDecoder,
    ) {
        // spread
        dec.spread = if range_dec.available() > 4 {
            match range_dec.icdf(&SPREAD_MODEL_DICT) {
                0 => None,
                1 => Some(Spread::Light),
                2 => Some(Spread::Normal),
                3 => Some(Spread::Aggressive),
                _ => unreachable!(),
            }
        } else {
            Some(Spread::Normal)
        };

        // Initialize static allocation caps
        for i in 0..MAX_BANDS {
            let bits =
                (STATIC_CAPS[dec.size][dec.channels as usize - 1][i] + 64) * LOG_GREQ_RANGE[i];

            dec.caps[i] = bits << (dec.channels as i32 - 1) << dec.size as i32 >> 2;
        }

        // Band boosts
        let mut dynalloc = 6;
        let mut tbits_8ths = range_dec.len();
        for i in dec.band_range.clone() {
            let mut band_dynalloc = dynalloc;
            let quanta = {
                let it = FREQ_RANGE[i] << dec.channels as i32 - 1 << dec.size as i32;
                (it << 3).min(it.max(6 << 3))
            };

            let mut boost = [0; MAX_BANDS];
            while range_dec.tell_frac() + (band_dynalloc << 3) < tbits_8ths
                && boost[i] < dec.caps[i]
            {
                if !range_dec.logp(band_dynalloc) {
                    break;
                }

                boost[i] += quanta as i32;
                tbits_8ths -= quanta as usize;
                band_dynalloc = 1;
            }

            if boost[1] > 0 {
                dynalloc = 2.max(dynalloc - 1);
            }
        }

        // Allocation trim
        dec.alloc_trim = if range_dec.tell_frac() + (6 << 3) <= tbits_8ths {
            range_dec.icdf(&ALLOC_TRIM_MODEL)
        } else {
            5
        };

        // Anti-collapse bit reservation
        tbits_8ths = (range_dec.len() << 3) - range_dec.tell_frac() - 1;
        dec.anticollapse_needed =
            if dec.transient && dec.size >= 2 && tbits_8ths >= (dec.size + 2) << 3 {
                1 << 3
            } else {
                0
            };

        tbits_8ths = dec.anticollapse_needed;

        // Band skip bit reservation
        let mut skip_bit = 0;
        if tbits_8ths >= 1 << 3 {
            skip_bit = 1 << 3;
        }

        tbits_8ths -= skip_bit;

        // Intensity/dual stereo bit reservation
        let mut dualstereo_bit = 0;
        let mut intensitystereo_bit = 0;
        if dec.channels == Channels::Stereo {
            intensitystereo_bit = LOG2_FRAC[dec.band_range.end - dec.band_range.start];
            if intensitystereo_bit <= tbits_8ths {
                tbits_8ths -= intensitystereo_bit;
                if tbits_8ths >= 1 << 3 {
                    dualstereo_bit = 1 << 3;
                    tbits_8ths -= 1 << 3;
                }
            } else {
                intensitystereo_bit = 0;
            }
        }

        // Trim offsets
        let mut trim_offset = [0i32; MAX_BANDS];
        let mut threshold = [0i32; MAX_BANDS];
        for i in dec.band_range.clone() {
            let trim = (dec.alloc_trim - 5 - dec.size) as i32;
            let band = FREQ_RANGE[i] * (dec.band_range.end - i - 1) as i32;
            let duration = (dec.size + 3) as i32;
            let scale = duration + dec.channels as i32 - 1;

            // PVQ minimum allocation threshold, below this value the band is skipped
            threshold[i] = (3 * FREQ_RANGE[i] << duration >> 4).max((dec.channels as i32) << 3);
            trim_offset[i] = trim * (band << scale) >> 6;

            if FREQ_RANGE[i] << dec.size as i32 == 1 {
                trim_offset[i] -= (dec.channels as i32) << 3;
            }
        }

        // Bisection
        let mut low = 1;
        let mut high = VECTORS - 1;
        while low <= high {
            let center = (low + high) >> 1;

            for _ in dec.band_range {

            }
        }
    }
}
