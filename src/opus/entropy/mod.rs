pub mod bits;

use integer_sqrt::IntegerSquareRoot;

use self::bits::{LittleEndianBitReader, BigEndianBitReader};

/// Probability distribution coding, suitable for audio coding scenarios because
/// audio is all continuously varying data with continuous spectral lines, not
/// drastically varying or random data.
///
/// A simplified version of Range Decoder for decoding the range encoding format
/// used by Opus/CELT.
pub struct RangeCodingDecoder<'a> {
    // Used to read forward bitstreams.
    forward_reader: BigEndianBitReader<'a>,
    // For reading reverse bitstreams (rawbits use)
    reverse_reader: LittleEndianBitReader<'a>,
    // Total number of input bits.
    bitstream_length: usize,
    // Current decoder range (similar to the difference between the upper and
    // lower limits)
    current_range: usize,
    // Current cumulative value.
    coded_value: usize,
    // Total bits consumed (for tell/tell_frac)
    consumed_bits: usize,
}

impl<'a> RangeCodingDecoder<'a> {
    // Number of bits used per symbol (8 bits)
    const SYMBOL_BITS: usize = 8;

    // Symbol Maximum (255)
    const SYMBOL_MAX: usize = (1 << Self::SYMBOL_BITS) - 1;

    // Total number of encoded bits (usually 32)
    const UNIFORM_THRESHOLD_BITS: usize = 8;

    // Encoder upper limit value (highest bit is 1)
    const CODE_MAX_VALUE: usize = 1 << (32 - 1);

    // Minimum legal interval (needs to be normalized)
    const CODE_MIN_NORMALIZATION: usize = Self::CODE_MAX_VALUE >> Self::SYMBOL_BITS;

    /// Ensure that range is not less than CODE_BOT (to prevent loss of precision)
    fn ensure_valid_range(&mut self) {
        while self.current_range <= Self::CODE_MIN_NORMALIZATION {
            // Expands the value each time and reads a new 8bit replacement
            // SYM_MAX after XOR for CELT decoding compatibility.
            self.coded_value = ((self.coded_value << Self::SYMBOL_BITS)
                | self.forward_reader.get_bits_32(Self::SYMBOL_BITS) as usize ^ Self::SYMBOL_MAX)
                & (Self::CODE_MAX_VALUE - 1);

            // Interval scaling to maintain accuracy.
            self.current_range <<= Self::SYMBOL_BITS;
            self.consumed_bits += Self::SYMBOL_BITS;
        }
    }

    /// Update cumulative values and probability ranges.
    fn update_range_and_value(&mut self, scale: usize, low: usize, high: usize, total: usize) {
        // Calculate the size of the encoded value corresponding to the high
        // part (in the interval [high, total))
        let size = scale * (total - high);

        // Update the encoded value (subtract the high part from the value)
        self.coded_value -= size;

        // Determine whether to update the range with the difference or compute
        // the difference interval directly based on low/high.
        self.current_range = if low != 0 {
            scale * (high - low)
        } else {
            self.current_range - size
        };

        self.ensure_valid_range();
    }

    /// Get the probability interval where the current value is located, and
    /// calculate the current symbol index.
    fn get_scale_symbol(&self, total: usize) -> (usize, usize) {
        let range_scale = self.current_range / total;

        // value The inverse index mapped to the distribution (the further back,
        // the smaller the value)
        (
            range_scale,
            total - (self.coded_value / range_scale + 1).min(total),
        )
    }
}

impl<'a> RangeCodingDecoder<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        let mut forward_reader = BigEndianBitReader::new(bytes);

        // Reads 7 bits and subtracts it from 127 to get the initial value.
        let coded_value = 127 - forward_reader.get_bits_32(7) as usize;

        let mut this = Self {
            reverse_reader: LittleEndianBitReader::new(bytes),
            consumed_bits: Self::SYMBOL_BITS + 1,
            bitstream_length: bytes.len() * 8,
            current_range: 128,
            coded_value,
            forward_reader,
        };

        // Ensure that the initial state is legal.
        this.ensure_valid_range();

        this
    }

    /// Dichotomous decoding: cut the current interval by 2^logp, return true
    /// or false.
    pub fn logp(&mut self, logp: usize) -> bool {
        let range_scale = self.current_range >> logp;

        let result = if range_scale > self.coded_value {
            self.current_range = range_scale;

            true
        } else {
            self.current_range -= range_scale;
            self.coded_value -= range_scale;

            false
        };

        self.ensure_valid_range();
        result
    }

    /// Decoding based on iCDF tables (Opus method)
    pub fn icdf(&mut self, icdf: &ICDFContext) -> usize {
        let (range_scale, symbol_index) = self.get_scale_symbol(icdf.total);

        // Determine the probability interval corresponding to the current
        // symbol and then update the decoder state
        let value = icdf.dist.iter().position(|v| *v > symbol_index).unwrap();

        self.update_range_and_value(
            range_scale,
            if value > 0 { icdf.dist[value - 1] } else { 0 },
            icdf.dist[value],
            icdf.total,
        );

        value
    }

    /// Returns the number of bits currently consumed (integer)
    pub fn tell(&self) -> usize {
        self.consumed_bits - self.current_range.ilog2() as usize - 1
    }

    /// Returns the number of bits currently consumed (with decimal precision)
    pub fn tell_frac(&self) -> usize {
        let mut log2_range = self.current_range.ilog2() as usize - 1;
        let mut range_q15 = self.current_range >> (log2_range - 16);

        // Newton's iterative approximation log2 accuracy (3 times)
        for _ in 0..3 {
            range_q15 = (range_q15 * range_q15) >> 15;

            let lastbit = range_q15 >> 16;
            log2_range = log2_range * 2 | lastbit;
            range_q15 >>= lastbit;
        }

        self.consumed_bits * 8 - log2_range
    }

    pub fn len(&self) -> usize {
        self.bitstream_length
    }

    pub fn available(&self) -> usize {
        self.bitstream_length - self.tell()
    }

    pub fn available_frac(&self) -> usize {
        self.bitstream_length * 8 - self.tell_frac()
    }
}

pub trait CeltRangeCoding {
    fn rawbits(&mut self, len: usize) -> usize;
    fn uniform(&mut self, len: usize) -> usize;
    fn laplace(&mut self, symbol: usize, decay: isize) -> isize;
    fn step(&mut self, k0: usize) -> usize;
    fn triangular(&mut self, qn: usize) -> usize;
    fn to_end(&mut self);
}

impl<'a> CeltRangeCoding for RangeCodingDecoder<'a> {
    /// Read raw bits without probabilistic modeling (CELT will use reverse
    /// bitstream to do complementary)
    fn rawbits(&mut self, len: usize) -> usize {
        self.consumed_bits += len;

        self.reverse_reader.get_bits_32(len) as usize
    }

    /// Decode uniformly distributed integer values (0 to len-1)
    fn uniform(&mut self, len: usize) -> usize {
        let bits = ((len - 1).ilog2() - 1) as usize;

        // If bits exceeds UNI_BITS, narrow the denominator first and then add
        // rawbits; otherwise, decode directly.
        let total = if bits > Self::UNIFORM_THRESHOLD_BITS {
            ((len - 1) >> (bits - Self::UNIFORM_THRESHOLD_BITS)) + 1
        } else {
            len
        };

        let (range_scale, symbol_idx) = self.get_scale_symbol(total);

        self.update_range_and_value(range_scale, symbol_idx, symbol_idx + 1, total);

        if bits > Self::UNIFORM_THRESHOLD_BITS {
            symbol_idx << (bits - Self::UNIFORM_THRESHOLD_BITS)
                | self.rawbits(bits - Self::UNIFORM_THRESHOLD_BITS)
        } else {
            symbol_idx
        }
    }

    /// Decoding Laplace distributions (for centrosymmetric structures)
    fn laplace(&mut self, mut symbol: usize, decay: isize) -> isize {
        let range_scale = self.current_range >> 15;

        // Constructed center value: the closer to 0 the more likely it is to be
        // sampled, consistent with the properties of the Laplace distribution.
        // center is larger, the more off-center it is.
        let center = (1 << 15) - (self.coded_value / range_scale + 1).min(1 << 15);

        // Starting at center, the exponential expansion looks for possible
        // symbolic values on both sides.
        let (value, low) = if center >= symbol {
            let mut value = 1;
            let mut low = symbol;
            symbol = 1 + ((32768 - 32 - symbol) * (16384 - decay as usize) >> 15);

            while symbol > 1 && center >= low + 2 * symbol {
                value += 1;
                symbol *= 2;
                low += symbol;

                // Exponential decay: the probability decreases with distance
                // from the center.
                symbol = (((symbol - 2) * decay as usize) >> 15) + 1;
            }

            if symbol <= 1 {
                let dist = (center - low) >> 1;
                value += dist as isize;
                low += 2 * dist;
            }

            // Determines if it falls in the negative range based on the final
            // center position.
            if center < low + symbol {
                value *= -1;
            } else {
                low += symbol;
            }

            (value, low)
        } else {
            (0, 0)
        };

        self.update_range_and_value(range_scale, low, 32768.min(low + symbol), 32768);

        value
    }

    /// CELT custom non-uniform distribution modeler (for pulse-distributed coding)
    fn step(&mut self, k0: usize) -> usize {
        let k1 = (k0 + 1) * 3;
        let total = k1 + k0;
        let range_scale = self.current_range / total;
        let symbol = total - (self.coded_value / range_scale + 1).min(total);

        // First half: each k maps to three symbols.
        // Second half: each k maps to a symbol (asymmetric)
        let k = if symbol < k1 {
            symbol / 3
        } else {
            // Here the derivation of k corresponds to the CELT coding end
            // compensation interval.
            symbol - (k0 + 1) / 2
        };

        if k <= k0 {
            self.update_range_and_value(range_scale, 3 * (k + 0), 3 * (k + 1), total);
        } else {
            self.update_range_and_value(
                range_scale,
                3 * (k + 1) + (k - 1 - k0),
                3 * (k0 + 1) + (k - 0 - k0),
                total,
            );
        }

        k
    }

    /// Decoding a triangular distribution (pulse-symmetric encoding of CELT)
    fn triangular(&mut self, qn: usize) -> usize {
        let half_level = qn >> 1;
        let total = (half_level + 1) * (half_level + 1);
        let range_scale = self.current_range / total;

        // center maps to an inverse-order integer, which is used to quickly
        // find the distribution segment on which it falls.
        let center = total - (self.coded_value / range_scale + 1).min(total);

        // Upper Triangle vs Lower Triangle distribution judgment.
        let (k, low, symbol) = if center < total >> 1 {
            // Bottom half: incremental fill from top left (top left to bottom right)
            let k = ((8 * center + 1).integer_sqrt() - 1) >> 1;

            (k, k * (k + 1) >> 1, k + 1)
        } else {
            // Top: Push back from the bottom right corner to the top left corner.
            let k = (2 * (qn + 1) - (8 * (total - center - 1) + 1).integer_sqrt()) >> 1;

            (k, total - ((qn + 1 - k) * (qn + 2 - k) >> 1), qn + 1 - k)
        };

        self.update_range_and_value(range_scale, low, low + symbol, total);

        k
    }

    /// Advances the decoder to the end of the data, patching unused bits
    /// (CELT is usually called)
    fn to_end(&mut self) {
        self.consumed_bits += self.bitstream_length - self.tell();
    }
}

#[derive(Debug)]
pub struct ICDFContext {
    pub total: usize,
    pub dist: &'static [usize],
}

#[cfg(test)]
mod test {
    use super::{CeltRangeCoding, RangeCodingDecoder};

    #[test]
    fn decode_laplace() {
        let buf = [
            255, 201, 249, 161, 77, 172, 239, 17, 161, 157, 220, 130, 101, 192, 199, 41, 223, 112,
            126, 194, 59, 131, 246, 99, 239, 250, 102, 73, 130, 190, 207, 77, 157, 254, 59, 79,
            240, 126, 166, 230, 157, 142, 227, 61, 198, 110, 75, 187, 94, 218, 58, 183, 246, 167,
            234, 223, 218, 159, 168, 63, 125, 254, 80, 85, 117, 128, 138, 1, 68, 51, 4, 53, 68,
            128, 222, 96, 236, 107, 71, 34, 144, 68, 200, 90, 232, 144, 173, 216, 248, 210, 30,
            126, 125, 27, 252, 125, 25, 86, 247, 139, 163, 76, 176, 113, 222, 186, 237, 158, 228,
            21, 234, 154, 90, 113, 107, 5, 13, 60, 197, 169, 172, 9, 217, 128, 155, 163, 157, 34,
            130, 47, 235, 183, 24, 22, 236, 226, 21, 207, 195, 113, 103, 74, 227, 166, 6, 124, 55,
            25, 22, 178, 213, 218,
        ];

        let mut rd = RangeCodingDecoder::new(&buf);

        let val = [
            (3, 32497, 60),
            (0, 32505, 58),
            (-1, 32512, 56),
            (0, 32185, 139),
            (1, 32425, 78),
            (3, 32134, 152),
            (2, 32189, 138),
            (1, 32303, 109),
            (-7, 32122, 155),
            (5, 32212, 132),
            (5, 32248, 123),
            (0, 32342, 99),
            (3, 32173, 142),
            (-4, 32421, 79),
            (2, 32271, 117),
            (3, 32326, 103),
            (2, 32386, 88),
            (4, 32319, 105),
            (0, 32473, 66),
            (-5, 32232, 127),
            (-2, 32326, 103),
            (-3, 32461, 69),
            (2, 32138, 151),
            (0, 32449, 72),
            (6, 32315, 106),
            (-2, 32161, 145),
            (1, 32334, 101),
            (2, 32114, 157),
            (4, 32130, 153),
            (6, 32362, 94),
            (6, 32142, 150),
            (2, 32354, 96),
            (-7, 32169, 143),
            (0, 32157, 146),
            (2, 32244, 124),
            (4, 32248, 123),
            (5, 32505, 58),
            (-4, 32485, 63),
            (6, 32413, 81),
            (3, 32169, 143),
            (-7, 32481, 64),
            (1, 32315, 106),
            (-2, 32185, 139),
            (-7, 32362, 94),
            (2, 32216, 131),
            (-5, 32161, 145),
            (-7, 32204, 134),
            (-1, 32149, 148),
            (-4, 32524, 53),
            (-2, 32449, 72),
            (5, 32315, 106),
            (-6, 32271, 117),
            (1, 32520, 54),
            (-1, 32271, 117),
            (4, 32267, 118),
            (-5, 32110, 158),
            (0, 32311, 107),
            (2, 32402, 84),
            (7, 32493, 61),
            (-2, 32319, 105),
            (-7, 32307, 108),
            (0, 32461, 69),
            (1, 32330, 102),
            (-7, 32232, 127),
            (2, 32189, 138),
            (3, 32189, 138),
            (1, 32299, 110),
            (-6, 32291, 112),
            (0, 32122, 155),
            (0, 32528, 52),
            (0, 32374, 91),
            (-7, 32252, 122),
            (-1, 32413, 81),
            (-2, 32138, 151),
            (1, 32311, 107),
            (-6, 32142, 150),
            (-2, 32145, 149),
            (-2, 32236, 126),
            (-5, 32346, 98),
            (-7, 32263, 119),
            (2, 32287, 113),
            (-3, 32378, 90),
            (3, 32236, 126),
            (-4, 32394, 86),
            (1, 32259, 120),
            (-4, 32204, 134),
            (-7, 32204, 134),
            (-6, 32323, 104),
            (4, 32126, 154),
            (6, 32110, 158),
            (-3, 32142, 150),
            (-2, 32421, 79),
            (6, 32429, 77),
            (3, 32271, 117),
            (-5, 32224, 129),
            (-5, 32319, 105),
            (-6, 32122, 155),
            (6, 32181, 140),
            (7, 32386, 88),
            (3, 32398, 85),
        ];

        val.iter().for_each(|&(ret, symbol, decay)| {
            assert_eq!(ret, rd.laplace(symbol, decay));
        });
    }
}
