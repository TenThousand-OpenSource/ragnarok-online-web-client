//! Reading the game's files: one reader of little-endian numbers for every format (`.gat`, `.gnd`, `.rsw`,
//! `.rsm`, `.str`, `.spr`, `.act`, `.imf`, `.bmp`), written here once. It never reads past the file's end:
//! what is not there is an error with its reason, and a parser leaves with `?`.
//!
//! The packets of the protocol have their own reading, by place: `net::bytes`.

/// A file being read from its start: its bytes and how far the reading has come.
pub struct Reader<'a> {
    pub bytes: &'a [u8],
    pub at: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    /// How many bytes are left to read.
    pub fn left(&self) -> usize {
        self.bytes.len() - self.at
    }

    /// The next `count` bytes. A count so large that it cannot be added to the place is "a size too large",
    /// one past the file's end "the file ends too early".
    pub fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let slice = self.bytes.get(self.at..self.at.checked_add(count).ok_or("a size too large")?).ok_or("the file ends too early")?;
        self.at += count;
        Ok(slice)
    }

    pub fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    pub fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    pub fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn i32(&mut self) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    /// So many numbers in a row: a vector, a colour, a matrix.
    pub fn floats<const N: usize>(&mut self) -> Result<[f32; N], String> {
        let mut floats = [0.0; N];
        for float in &mut floats {
            *float = self.f32()?;
        }
        Ok(floats)
    }

    /// A count or a size (an `i32` in the file) of things `each` bytes long that the rest of the file could
    /// hold: a negative one, or one of more than is left, is an error.
    pub fn count(&mut self, each: usize) -> Result<usize, String> {
        let count = usize::try_from(self.i32()?).map_err(|_| "a negative count")?;
        if count.saturating_mul(each) > self.left() {
            return Err("a count larger than the file".to_owned());
        }
        Ok(count)
    }

    /// A zero-terminated name in a field of `size` bytes, as its bytes (the client's files name things in
    /// EUC-KR: the caller decodes them).
    pub fn name(&mut self, size: usize) -> Result<&'a [u8], String> {
        let field = self.take(size)?;
        Ok(&field[..field.iter().position(|&byte| byte == 0).unwrap_or(size)])
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;

    #[test]
    fn reads_numbers_one_after_another() {
        let mut bytes = vec![7, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0xfe, 0xff, 0xff, 0xff];
        bytes.extend(1.5f32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        assert_eq!((reader.u8(), reader.u16(), reader.u32()), (Ok(7), Ok(0x1234), Ok(0x1234_5678)));
        assert_eq!((reader.i32(), reader.f32()), (Ok(-2), Ok(1.5)));
        assert_eq!((reader.at, reader.left()), (15, 0));
    }

    #[test]
    fn never_reads_past_the_end() {
        let mut reader = Reader::new(&[1, 2, 3]);
        assert_eq!(reader.u32(), Err("the file ends too early".to_owned()));
        // What could not be read was not passed over.
        assert_eq!((reader.at, reader.u16()), (0, Ok(0x0201)));
        assert_eq!(reader.take(usize::MAX), Err("a size too large".to_owned()));
        assert_eq!((reader.take(1), reader.u8()), (Ok(&[3u8][..]), Err("the file ends too early".to_owned())));
    }

    #[test]
    fn reads_floats_in_a_row() {
        let bytes: Vec<u8> = [1.0f32, -2.0, 0.25].iter().flat_map(|float| float.to_le_bytes()).collect();
        assert_eq!(Reader::new(&bytes).floats::<3>(), Ok([1.0, -2.0, 0.25]));
        assert!(Reader::new(&bytes).floats::<4>().is_err());
    }

    #[test]
    fn a_count_must_fit_what_is_left() {
        let file = |count: i32| [count.to_le_bytes().as_slice(), &[0; 8]].concat();
        assert_eq!(Reader::new(&file(2)).count(4), Ok(2));
        assert_eq!(Reader::new(&file(3)).count(4), Err("a count larger than the file".to_owned()));
        assert_eq!(Reader::new(&file(-1)).count(1), Err("a negative count".to_owned()));
        assert_eq!(Reader::new(&file(i32::MAX)).count(usize::MAX), Err("a count larger than the file".to_owned()));
    }

    #[test]
    fn a_name_ends_at_its_zero_or_with_its_field() {
        let mut reader = Reader::new(b"ab\0junk!whole");
        assert_eq!(reader.name(8), Ok(&b"ab"[..]));
        assert_eq!((reader.name(5), reader.name(1).is_err()), (Ok(&b"whole"[..]), true));
    }
}
