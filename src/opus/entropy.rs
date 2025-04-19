use bytes::Buf;

/// Probability distribution coding, suitable for audio coding scenarios because
/// audio is all continuously varying data with continuous spectral lines, not
/// drastically varying or random data.
///
/// A simplified version of Range Decoder for decoding the range encoding format
/// used by Opus/CELT.
pub struct RangeCodingDecoder<'a> {
    // Input bitstream as byte buffer (e.g. CELT frame data)
    bytes: &'a [u8],
    // Position in the current interval ("value" of the current encoding status)
    value: u32,
    // Current interval size (shrinking with each decoding)
    range: u32,
    // Indicates how many more complementary bits are required to maintain
    // sufficient precision (initially negative)
    count: i32,
}

impl<'a> RangeCodingDecoder<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        let mut this = Self {
            bytes,
            value: 0,
            range: 0xFFFF_FFFF, // Initial interval is [0, 1), maximum accuracy
            // Initial count < 0, trigger fill
            count: -15,
        };

        this
    }

    fn refill(&mut self) {}
}
