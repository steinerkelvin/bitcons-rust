use primitive_types::U256;

// Types

#[derive(Debug)]
pub enum DeserError {
    Message(String),
}

// Boxed bytes iterator
pub type U8IteratorBox<'a> = Box<dyn Iterator<Item = u8> + 'a>;

// Traits

// Serializable to bytes using iterator
pub trait Ser<'a> {
    fn ser_iter(self: &'a Self) -> U8IteratorBox<'a>;
}

// Deserializable from bytes from iterator
pub trait Deser<'a> {
    fn deser_from_iter<I>(it: &mut I) -> Result<Self, DeserError>
    where
        Self: Sized,
        I: Iterator<Item = u8>;
}

// Iterator that owns a vector of bytes

pub struct VecU8Iterator {
    val: Vec<u8>,
    pos: usize,
}

impl VecU8Iterator {
    pub fn new(val: Vec<u8>) -> Self {
        VecU8Iterator { val: val, pos: 0}
    }
}

impl Iterator for VecU8Iterator {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.val.len() {
            let res = self.val[self.pos];
            self.pos += 1;
            Some(res)
        } else {
            None
        }
    }
}

// U256 implementation

const U256_SIZE: usize = 256 / 8;

struct U256SerIter<'a> {
    val: &'a U256,
    pos: usize,
}

impl Iterator for U256SerIter<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.pos >= U256_SIZE {
            None
        } else {
            let res = self.val.byte(self.pos);
            self.pos += 1;
            Some(res)
        }
    }
}

impl<'a> Ser<'a> for U256 {
    fn ser_iter(self: &'a U256) -> U8IteratorBox<'a> {
        Box::new(U256SerIter { val: self, pos: 0 })
    }
}

impl<'a> Deser<'a> for U256 {
    fn deser_from_iter<I>(it: &mut I) -> Result<Self, DeserError>
    where
        I: Iterator<Item = u8>,
    {
        let it = it.take(U256_SIZE);
        let bytes: Vec<u8> = it.collect();
        // TODO test size
        Ok(U256::from_little_endian(bytes.as_slice()))
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u256_ser() {
        let v = U256::from_little_endian(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
            19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
        ]);
        let iter = v.ser_iter();
        for (i, v) in iter.enumerate() {
            println!("{:2}: {:x}", i, v);
        }
        let iter = v.ser_iter();
        let encoded: Vec<u8> = iter.collect();
        assert_eq!(encoded.len(), U256_SIZE);

        let mut stream = encoded.iter().copied();
        let result = U256::deser_from_iter(&mut stream);
        let reconstructed = result.unwrap();
        assert!(reconstructed.eq(&v));
    }
}
