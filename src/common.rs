#![allow(dead_code)]

use primitive_types::U256;
use serde::{Serialize, Serializer};
use sha3::Digest;

const WORD_SIZE: usize = 32; // 256 bits
const BODY_SIZE: usize = 1024;
const POST_SIZE: usize = 2 * WORD_SIZE + BODY_SIZE;

#[derive(Debug, Clone)]
struct Body {
    val: [u8; BODY_SIZE],
}

impl Default for Body {
    fn default() -> Self {
        Body {
            val: [0u8; BODY_SIZE],
        }
    }
}

impl Serialize for Body {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq_ser = serializer.serialize_seq(Some(BODY_SIZE))?;
        for i in 0..(self.val.len()) {
            seq_ser.serialize_element(&self.val[i])?;
        }
        seq_ser.end()
    }
}

impl From<[u8; 1024]> for Body {
    fn from(val: [u8; 1024]) -> Self {
        Body { val }
    }
}

impl Body {
    fn serialize_to(&self, bytes: &mut [u8]) {
        for i in 0..(self.val.len()) {
            bytes[i] = self.val[i];
        }
    }
}

#[derive(Debug, Serialize, Default)]
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
    fn serialize_to(&self, buf: &mut [u8; POST_SIZE]) {
        self.prev.to_little_endian(&mut buf[00..32]);
        self.work.to_little_endian(&mut buf[32..64]);
        self.body.serialize_to(&mut buf[64..(64 + BODY_SIZE)]);
    }
    fn serialize(&self) -> [u8; POST_SIZE] {
        let mut buf = [0u8; POST_SIZE];
        self.serialize_to(&mut buf);
        buf
    }
    fn hash(&self) -> U256 {
        if self.prev.is_zero() && self.work.is_zero() {
            return U256::zero();
        }
        let ser = self.serialize();
        let hasher = sha3::Keccak256::new();
        let hash = hasher.chain(&ser).finalize();
        let res = U256::from_little_endian(&hash);
        res
    }
}

fn hash_score(hash: U256) -> U256 {
    if hash.is_zero() {
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

        // let work = U256::from_little_endian(&[0x42]);
        // let work = U256::from(0x44556677u64);

        let work = U256::from(0x04050607u64);
        let body: Body = Body::from([0x42; BODY_SIZE]);

        let post = Post::new(&prev, &work, &body);
        println!("POST: {:?}", post);

        // // Serialize Post with `bincode`
        // let encoded: Vec<u8> = bincode::serialize(&post).unwrap();

        // // Serialize with `flexbuffers`
        // use serde::Serialize;
        // let mut s = flexbuffers::FlexbufferSerializer::new();
        // post.serialize(&mut s).unwrap();
        // let encoded = s.view();

        // let mut encoded = [0u8; Post_size];
        // post.ser(&mut encoded);

        let encoded = post.serialize();

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
