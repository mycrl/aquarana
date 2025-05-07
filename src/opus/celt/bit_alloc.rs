use crate::opus::{entropy::RangeCodingDecoder, toc::Channels};

use super::{CeltFrameDecoder, MAX_BANDS};

pub const VECTORS: i32 = 11;

pub const SPREAD_MODEL_DICT: [usize; 5] = [32, 7, 9, 30, 32];

pub const STATIC_CAPS: [[[u8; 21]; 2]; 4] = [
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

pub const LOG_GREQ_RANGE: [u8; 21] = [
    0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8, 16, 16, 16, 21, 21, 24, 29, 34, 36,
];

pub const FREQ_RANGE: [u8; 21] = [
    1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 6, 6, 8, 12, 18, 22,
];

pub const ALLOC_TRIM_MODEL: [usize; 12] = [128, 2, 4, 9, 19, 41, 87, 109, 119, 124, 126, 128];

pub const LOG2_FRAC: [u8; 24] = [
    0, 8, 13, 16, 19, 21, 23, 24, 26, 27, 28, 29, 30, 31, 32, 32, 33, 34, 34, 35, 36, 36, 37, 37,
];

pub const STATIC_ALLOC: [[u8; 21]; 11] = [
    /* 1/32 bit/sample */
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        90, 80, 75, 69, 63, 56, 49, 40, 34, 29, 20, 18, 10, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        110, 100, 90, 84, 78, 71, 65, 58, 51, 45, 39, 32, 26, 20, 12, 0, 0, 0, 0, 0, 0,
    ],
    [
        118, 110, 103, 93, 86, 80, 75, 70, 65, 59, 53, 47, 40, 31, 23, 15, 4, 0, 0, 0, 0,
    ],
    [
        126, 119, 112, 104, 95, 89, 83, 78, 72, 66, 60, 54, 47, 39, 32, 25, 17, 12, 1, 0, 0,
    ],
    [
        134, 127, 120, 114, 103, 97, 91, 85, 78, 72, 66, 60, 54, 47, 41, 35, 29, 23, 16, 10, 1,
    ],
    [
        144, 137, 130, 124, 113, 107, 101, 95, 88, 82, 76, 70, 64, 57, 51, 45, 39, 33, 26, 15, 1,
    ],
    [
        152, 145, 138, 132, 123, 117, 111, 105, 98, 92, 86, 80, 74, 67, 61, 55, 49, 43, 36, 20, 1,
    ],
    [
        162, 155, 148, 142, 133, 127, 121, 115, 108, 102, 96, 90, 84, 77, 71, 65, 59, 53, 46, 30, 1,
    ],
    [
        172, 165, 158, 152, 143, 137, 131, 125, 118, 112, 106, 100, 94, 87, 81, 75, 69, 63, 56, 45,
        20,
    ],
    [
        200, 200, 200, 200, 200, 200, 200, 200, 198, 193, 188, 183, 178, 173, 168, 163, 158, 153,
        148, 129, 104,
    ],
];

#[derive(Debug)]
pub enum Spread {
    Light,
    Normal,
    Aggressive,
}

pub struct BitAlloc;

impl BitAlloc {
    pub fn decode(dec: &mut CeltFrameDecoder, range_dec: &mut RangeCodingDecoder) {
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
                (STATIC_CAPS[dec.size][dec.channels as usize - 1][i] + 64) * FREQ_RANGE[i];

            dec.caps[i] = (bits as i32) << (dec.channels as i32 - 1) << dec.size as i32 >> 2;
        }

        // Band boosts
        let mut dynalloc = 6i32;
        let mut boost = [0i32; MAX_BANDS];
        let mut tbits_8ths = (range_dec.len() as i32) << 3;
        for i in dec.band_range.clone() {
            let mut band_dynalloc = dynalloc;
            let quanta = {
                let it = (FREQ_RANGE[i]) << (dec.channels as u8 - 1) << dec.size;
                (it << 3).min(it.max(6 << 3))
            };

            while range_dec.tell_frac() as i32 + (band_dynalloc << 3) < tbits_8ths
                && boost[i] < dec.caps[i]
            {
                if !range_dec.logp(band_dynalloc as usize) {
                    break;
                }

                boost[i] += quanta as i32;
                tbits_8ths -= quanta as i32;
                band_dynalloc = 1;
            }

            if boost[1] > 0 {
                dynalloc = 2.max(dynalloc - 1);
            }
        }

        // Allocation trim
        dec.alloc_trim = if range_dec.tell_frac() as i32 + (6 << 3) <= tbits_8ths {
            range_dec.icdf(&ALLOC_TRIM_MODEL) as i32
        } else {
            5
        };

        // Anti-collapse bit reservation
        tbits_8ths = ((range_dec.len() as i32) << 3) - range_dec.tell_frac() as i32 - 1;
        dec.anticollapse_needed =
            if dec.transient && dec.size >= 2 && tbits_8ths >= (dec.size as i32 + 2) << 3 {
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
            intensitystereo_bit = LOG2_FRAC[dec.band_range.end - dec.band_range.start] as i32;
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
            let trim = dec.alloc_trim - 5 - dec.size as i32;
            let band = FREQ_RANGE[i] as i32 * (dec.band_range.end as i32 - i as i32 - 1);
            let duration = dec.size as i32 + 3;
            let scale = duration + dec.channels as i32 - 1;

            // PVQ minimum allocation threshold, below this value the band is skipped
            threshold[i] = (3 * (FREQ_RANGE[i] as i32) << duration >> 4).max((dec.channels as i32) << 3);
            trim_offset[i] = trim * (band << scale) >> 6;

            if (FREQ_RANGE[i] as usize) << dec.size == 1 {
                trim_offset[i] -= (dec.channels as i32) << 3;
            }
        }

        // Bisection
        let mut done = false;
        let mut total = 0;
        let mut bandbits = 0;
        let mut low = 1;
        let mut high = VECTORS - 1;
        while low <= high {
            done = false;
            total = 0;

            let center = (low + high) >> 1;
            for i in dec.band_range.clone().into_iter().rev() {
                bandbits = (FREQ_RANGE[i] as i32 * STATIC_ALLOC[center as usize][i] as i32)
                    << (dec.channels as u8 - 1)
                    << dec.size
                    >> 2;

                if bandbits > 0 {
                    bandbits = 0.max(bandbits as i32 + trim_offset[i] as i32);
                }

                bandbits += boost[i];

                if bandbits >= threshold[i] || done {
                    done = true;
                    total += bandbits.min(dec.caps[i]);
                } else if bandbits >= (dec.channels as i32) << 3 {
                    total += (dec.channels as i32) << 3;
                }
            }

            if total > tbits_8ths {
                high = center - 1;
            } else {
                low = center + 1;
            }
        }
        high = low - 1;

        // Bisection
        for i in dec.band_range.clone() {
            let mut bandbits = (FREQ_RANGE[i] as i32 * STATIC_ALLOC[low as usize][i] as i32)
                << (dec.channels as u8 - 1)
                << dec.size
                >> 2;
                
                
        }
    }
}
