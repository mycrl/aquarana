/// A big-endian reader.
///
/// It contains the data structures necessary to create a big-endian reader.
pub struct BigEndianBitReader<'a> {
    /// read buffer, 8-bytes padded
    bytes: &'a [u8],
    index: usize,
    cache: u64,
    left: usize,
}

impl<'a> BigEndianBitReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        let mut this = Self {
            bytes,
            index: 0,
            cache: 0,
            left: 0,
        };

        {
            if this.readable() {
                this.cache = this.read::<u64>();
                this.index += 8;
                this.left += 64;
            }
        }

        this
    }

    /// Returns n bits from the internal buffer as a 32-bit sequence.
    pub fn get_bits_32(&mut self, n: usize) -> u32 {
        if n == 0 {
            return 0;
        }

        // Gets the number of bits left in an internal buffer.
        if self.left <= n {
            // Extracts a 32-bit sequence from an internal buffer and saves it
            // within an internal cache.

            if self.readable() {
                self.cache = self.cache | self.read::<u32>() << (32 - self.left);
                self.index += 4;
                self.left += 32;
            }
        }

        // Returns n bits from an internal buffer.
        ({
            // Peeks n bits from the cache.
            let value = self.cache.checked_shr(64 - n as u32).unwrap_or(0);

            // Removes n bits from the cache.
            {
                self.cache = self.cache.checked_shl(n as u32).unwrap_or(0);
                self.left = self.left.saturating_sub(n);
            }

            value
        }) as u32
    }

    /// Tells if it is still possible to read bits from an internal buffer.
    fn readable(&self) -> bool {
        self.index < self.bytes.len()
    }

    // Gets a 32-bits / 64-bits sequence from an internal buffer.
    fn read<T>(&self) -> u64 {
        let count = size_of::<T>();
        let start = self.index;
        let end = self.bytes.len().min(self.index + count);

        let mut value = 0;
        for byte in &self.bytes[start..end] {
            value = value << 8 | *byte as u64;
        }

        value << (8 * (count - (end - start)))
    }
}

/// A little-endian reader.
///
/// It contains the data structures necessary to create a little-endian reader.
pub struct LittleEndianBitReader<'a> {
    /// read buffer, 8-bytes padded
    bytes: &'a [u8],
    index: usize,
    cache: u64,
    left: usize,
}

impl<'a> LittleEndianBitReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        let mut this = Self {
            bytes,
            index: 0,
            cache: 0,
            left: 0,
        };

        {
            if this.readable() {
                this.cache = this.read::<u64>();
                this.index += 8;
                this.left += 64;
            }
        }

        this
    }

    pub fn get_bits_32(&mut self, n: usize) -> u32 {
        if n == 0 {
            return 0;
        }

        // Gets the number of bits left in an internal buffer.
        if self.left <= n {
            // Extracts a 32-bit sequence from an internal buffer and saves it
            // within an internal cache.

            if self.readable() {
                self.cache = self.cache | self.read::<u32>() << self.left;
                self.index += 4;
                self.left += 32;
            }
        }

        // Returns n bits from an internal buffer.
        ({
            // Peeks n bits from the cache.
            let value = self.cache
                & ((1u64.checked_shl(n as u32).unwrap_or(0))
                    .overflowing_sub(1)
                    .0);

            // Removes n bits from the cache.
            {
                self.cache = self.cache.checked_shr(n as u32).unwrap_or(0);
                self.left = self.left.saturating_sub(n);
            }

            value
        }) as u32
    }

    /// Tells if it is still possible to read bits from an internal buffer.
    fn readable(&self) -> bool {
        self.index <= self.bytes.len()
    }

    // Gets a 32-bits / 64-bits sequence from an internal buffer.
    fn read<T>(&self) -> u64 {
        let count = size_of::<T>();
        let end = self.bytes.len() - self.index;
        let start = end.saturating_sub(count);

        let mut value = 0;
        for byte in &self.bytes[start..end] {
            value = value << 8 | *byte as u64;
        }

        value
    }
}

#[cfg(test)]
mod test {
    use super::{LittleEndianBitReader, BigEndianBitReader};

    #[test]
    fn reverse_bitread() {
        let buf = &[
            197, 105, 76, 120, 136, 74, 169, 50, 225, 8, 231, 211, 227, 151, 186, 58, 173, 139,
        ];

        let mut r = LittleEndianBitReader::new(buf);

        assert_eq!(r.get_bits_32(3), 3);
        assert_eq!(r.get_bits_32(3), 1);
        assert_eq!(r.get_bits_32(3), 6);
        assert_eq!(r.get_bits_32(3), 6);
        assert_eq!(r.get_bits_32(3), 2);
        assert_eq!(r.get_bits_32(3), 5);
        assert_eq!(r.get_bits_32(3), 6);
        assert_eq!(r.get_bits_32(3), 1);
        assert_eq!(r.get_bits_32(2), 2);
        assert_eq!(r.get_bits_32(2), 2);
        assert_eq!(r.get_bits_32(3), 3);
        assert_eq!(r.get_bits_32(3), 7);
        assert_eq!(r.get_bits_32(3), 5);
        assert_eq!(r.get_bits_32(3), 4);
        assert_eq!(r.get_bits_32(2), 3);
        assert_eq!(r.get_bits_32(2), 0);
        assert_eq!(r.get_bits_32(3), 6);
        assert_eq!(r.get_bits_32(3), 7);
        assert_eq!(r.get_bits_32(3), 4);
        assert_eq!(r.get_bits_32(3), 6);
        assert_eq!(r.get_bits_32(3), 7);
        assert_eq!(r.get_bits_32(3), 4);
        assert_eq!(r.get_bits_32(3), 3);
        assert_eq!(r.get_bits_32(3), 4);
        assert_eq!(r.get_bits_32(3), 0);
        assert_eq!(r.get_bits_32(3), 2);
        assert_eq!(r.get_bits_32(3), 0);
        assert_eq!(r.get_bits_32(3), 7);
        assert_eq!(r.get_bits_32(3), 2);
        assert_eq!(r.get_bits_32(3), 6);
        assert_eq!(r.get_bits_32(3), 4);
        assert_eq!(r.get_bits_32(3), 4);
        assert_eq!(r.get_bits_32(3), 2);
        assert_eq!(r.get_bits_32(3), 5);
        assert_eq!(r.get_bits_32(3), 2);
        assert_eq!(r.get_bits_32(3), 2);
        assert_eq!(r.get_bits_32(3), 0);
        assert_eq!(r.get_bits_32(3), 1);
        assert_eq!(r.get_bits_32(3), 2);
        assert_eq!(r.get_bits_32(3), 4);
        assert_eq!(r.get_bits_32(4), 7);
        assert_eq!(r.get_bits_32(4), 12);
        assert_eq!(r.get_bits_32(19), 284308);
    }

    #[test]
fn unpadded_bitread() {
    let buf = &[
        0b11010110, // 0xD6
        0b00101101, // 0x2D
        0b11100011, // 0xE3
        0b01010101, // 0x55
        0b10101010, // 0xAA
        0b00011100, // 0x1C
        0b11110000, // 0xF0
        0b00000001, // 0x01
    ];

    let mut r = BigEndianBitReader::new(buf);

    assert_eq!(r.get_bits_32(4), 0b1101);
    assert_eq!(r.get_bits_32(4), 0b0110);
    assert_eq!(r.get_bits_32(8), 0b00101101);
    assert_eq!(r.get_bits_32(6), 0b111000);
    assert_eq!(r.get_bits_32(2), 0b11);
    assert_eq!(r.get_bits_32(8), 0b01010101);
    assert_eq!(r.get_bits_32(8), 0b10101010);
    assert_eq!(r.get_bits_32(8), 0b00011100);
    assert_eq!(r.get_bits_32(8), 0b11110000);
    assert_eq!(r.get_bits_32(8), 0b00000001);
}
}
