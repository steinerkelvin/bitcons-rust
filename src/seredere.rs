use primitive_types::U256;

// Serializable to bytes using iterator
trait Ser<'a> {
    fn ser_iter(val: &'a U256) -> Box<dyn Iterator<Item = u8> + 'a>; // TODO generic return
}

// Deserializable from bytes from iterator
trait Deser {}

struct U256SerIter<'a> {
    val: &'a U256,
    pos: usize,
}

impl<'a> Ser<'a> for U256SerIter<'a> {
    fn ser_iter(val: &'a U256) -> Box<dyn Iterator<Item = u8> + 'a> {
        Box::new(U256SerIter { val, pos: 0 })
    }
}

impl Iterator for U256SerIter<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.pos >= 256 {
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
    fn test_U256_ser() {
        let v = U256::from_little_endian(&[0, 1, 2, 3, 4, 5, 6, 7]);
    }
}
