#![allow(dead_code)]

use primitive_types::U256;
// use serde::{Serialize, Serializer};
use sha3::Digest;

use super::seredere::{Deser, Ser, U8Iterator};

pub const WORD_SIZE: usize = 32; // 256 bits
pub const BODY_SIZE: usize = 1024;
pub const POST_SIZE: usize = 2 * WORD_SIZE + BODY_SIZE;

#[derive(Debug, Clone)]
pub struct Body {
    val: [u8; BODY_SIZE],
}

impl Default for Body {
    fn default() -> Self {
        Body {
            val: [0u8; BODY_SIZE],
        }
    }
}

impl From<[u8; 1024]> for Body {
    fn from(val: [u8; 1024]) -> Self {
        Body { val }
    }
}

impl<'a> Ser<'a> for Body {
    fn ser_iter(self: &'a Self) -> Box<dyn Iterator<Item = u8> + 'a> {
        Box::new(self.val.iter().cloned())
    }
}

impl Body {
    fn serialize_to(&self, bytes: &mut [u8]) {
        for i in 0..(self.val.len()) {
            bytes[i] = self.val[i];
        }
    }
}

#[derive(Debug, Default)]
pub struct Post {
    prev: U256, // previous post (32 bytes)
    work: U256, // extra info and nonce (32 bytes)
    body: Body, // post contents (1280(?) 1024 bytes)
}

impl Post {
    fn new(prev: &U256, work: &U256, body: &Body) -> Self {
        Post {
            prev: prev.clone(),
            work: work.clone(),
            body: body.clone(),
        }
    }
    fn hash(&self) -> U256 {
        if self.prev.is_zero() && self.work.is_zero() {
            return U256::zero();
        }
        let sered: Vec<u8> = self.ser_iter().collect();
        let hasher = sha3::Keccak256::new();
        let hash = hasher.chain(&sered).finalize();
        let res = U256::from_little_endian(&hash);
        res
    }
}

impl<'a> Ser<'a> for Post {
    fn ser_iter(self: &'a Self) -> U8Iterator<'a> {
        Box::new(
            self.prev
                .ser_iter()
                .chain(self.work.ser_iter())
                .chain(self.body.ser_iter()),
        )
    }
}

fn hash_score(hash: U256) -> U256 {
    if hash.is_zero() {
        // redundant. keep for clarity?
        U256::zero()
    } else {
        // divides hash by max 256-bit value
        U256::max_value().checked_div(hash).unwrap_or(U256::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    #[test]
    fn post_hash() {
        let mut rng = rand::thread_rng();
        let prev_arr = rng.gen::<[u8; 32]>();
        let prev = U256::from_little_endian(&prev_arr);

        let work = U256::from(0x04050607u64);
        let body: Body = Body::from([0x42; BODY_SIZE]);

        let post = Post::new(&prev, &work, &body);
        println!("POST: {:?}", post);

        let encoded: Vec<u8> = post.ser_iter().collect();

        println!("ENCODED:");
        for i in 0..encoded.len() {
            println!("{:4}: {:#04x}", i, encoded[i]);
        }

        println!("LEN: {:?}", encoded.len());
        assert_eq!(encoded.len(), POST_SIZE);

        let hash = post.hash();
        println!("HASH: {:x}", hash);
        println!("SCORE: {}", hash_score(hash));
    }

    #[test]
    fn genesis_post_hash_is_zero() {
        use std::default::Default;
        let genesis = Post::default();
        assert_eq!(genesis.hash(), U256::zero());
    }
}
