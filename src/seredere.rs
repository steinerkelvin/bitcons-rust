use primitive_types::U256;

// Serializable to bytes using iterator
trait Ser<'a> {
    fn ser_iter(self: &'a Self) -> Box<dyn Iterator<Item = u8> + 'a>;
}

// Deserializable from bytes from iterator
trait Deser {}

struct U256SerIter<'a> {
    val: &'a U256,
    pos: usize,
}

impl<'a> Ser<'a> for U256 {
    fn ser_iter(self: &'a U256) -> Box<dyn Iterator<Item = u8> + 'a> {
        Box::new(U256SerIter { val: self, pos: 0 })
    }
}

impl Iterator for U256SerIter<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.pos >= 256 / 8 {
            None
        } else {
            let res = self.val.byte(self.pos);
            self.pos += 1;
            Some(res)
        }
    }
}

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
        let items: Vec<u8> = iter.collect();
        assert_eq!(items.len(), 256 / 8);
        let iter = v.ser_iter();
        for (i, v) in iter.enumerate() {
            println!("{:2}: {:x}", i, v);
        }
    }
}
