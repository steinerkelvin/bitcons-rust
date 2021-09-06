#![allow(dead_code)]

use primitive_types::U256;
use sha3::Digest;

use super::seredere::{Deser, DeserError, Ser, U8IteratorBox, VecU8Iterator};

pub const WORD_SIZE: usize = 32; // 256 bits
pub const BODY_SIZE: usize = 1024;
pub const POST_SIZE: usize = 2 * WORD_SIZE + BODY_SIZE;

// ipv6 address size
pub const ADDRESS_SIZE: usize = 16;
// pub const ADDRESS_LIST_SIZE = |x: usize| x + 1 * ADDRESS_SIZE;

// Post Body

#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    val: [u8; BODY_SIZE],
}

impl Default for Body {
    fn default() -> Self {
        Body {
            val: [0u8; BODY_SIZE], // TODO Vec ?
        }
    }
}

impl From<[u8; BODY_SIZE]> for Body {
    fn from(val: [u8; 1024]) -> Self {
        Body { val }
    }
}

impl<'a> Ser<'a> for Body {
    fn ser_iter(self: &'a Self) -> Box<dyn Iterator<Item = u8> + 'a> {
        Box::new(self.val.iter().cloned())
    }
}

impl<'a> Deser<'a> for Body {
    fn deser_from_iter<I>(it: &mut I) -> Result<Self, DeserError>
    where
        I: Iterator<Item = u8>,
    {
        let it = it.take(BODY_SIZE);
        let bytes: Vec<u8> = it.collect();
        // TODO check size
        let mut arr: [u8; BODY_SIZE] = [0u8; BODY_SIZE];
        arr.clone_from_slice(bytes.as_slice());
        Ok(Body { val: arr })
    }
}

// Post

#[derive(Debug, Default, PartialEq)]
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
    fn ser_iter(self: &'a Self) -> U8IteratorBox<'a> {
        Box::new(
            self.prev
                .ser_iter()
                .chain(self.work.ser_iter())
                .chain(self.body.ser_iter()),
        )
    }
}

impl<'a> Deser<'a> for Post {
    fn deser_from_iter<I>(it: &mut I) -> Result<Self, DeserError>
    where
        I: Iterator<Item = u8>,
    {
        let prev_iter = it.take(WORD_SIZE);
        let prev_vec: Vec<u8> = prev_iter.collect();
        let prev = U256::from_little_endian(prev_vec.as_slice());
        let work_iter = it.take(WORD_SIZE);
        let work_vec: Vec<u8> = work_iter.collect();
        let work = U256::from_little_endian(work_vec.as_slice());
        // TODO clean ugly code repetition
        let body_iter = it.take(BODY_SIZE);
        let body_vec: Vec<u8> = body_iter.collect();
        let mut body = Body::default();
        body.val.clone_from_slice(body_vec.as_slice());
        Ok(Post { prev, work, body })
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

// Address

enum Address {
    IP(std::net::IpAddr, u16),
}

impl<'a> Ser<'a> for Address {
    fn ser_iter(self: &'a Self) -> U8IteratorBox<'a> {
        match self {
            Address::IP(ip, port) => {
                use std::net::IpAddr::{V4, V6};
                let ip6 = match ip {
                    V4(ip4) => ip4.to_ipv6_mapped(),
                    V6(ip6) => ip6.clone(),
                };
                let bytes: Vec<u8> = Vec::from(ip6.octets());
                let iter = VecU8Iterator::new(bytes);
                // TODO port
                Box::new(iter)
            }
        }
    }
}

impl<'a> Ser<'a> for Vec<Address> {
    fn ser_iter(self: &'a Self) -> U8IteratorBox<'a> {
        let size = self.len();
        if size > 255 {
            panic!("Address vector size is too big.")
        }
        let size = size as u8;

        let base_it = std::iter::once(size);

        // Then serialize all addresses
        // TODO reduce overhead
        let it = self
            .iter()
            .fold(Box::new(base_it) as U8IteratorBox<'a>, |acc, x| {
                Box::new(acc.chain(x.ser_iter()))
            });
        it
    }
}

// Message

enum Message {
    Ping(Vec<Address>),
    RequestPost(U256),
    SharePost(Post),
}

impl<'a> Ser<'a> for Message {
    fn ser_iter(self: &'a Self) -> U8IteratorBox<'a> {
        let it = match self {
            Message::Ping(addresses) => {
                let code = 0;
                let base_it = std::iter::once(code);
                base_it.chain(addresses.ser_iter())
            }
            Message::RequestPost(hash) => {
                let code = 1;
                let base_it = std::iter::once(code);
                base_it.chain(hash.ser_iter())
            }
            Message::SharePost(post) => {
                let code = 2;
                let base_it = std::iter::once(code);
                base_it.chain(post.ser_iter())
            }
        };
        Box::new(it)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use rand::prelude::*;

    #[test]
    fn post_seredere() {
        let mut rng = rand::thread_rng();
        let prev_arr = rng.gen::<[u8; 32]>();
        let prev = U256::from_little_endian(&prev_arr);

        let work = U256::from(0x04050607u64);
        let body: Body = Body::from([0x42; BODY_SIZE]);

        let post = Post::new(&prev, &work, &body);
        println!("POST: {:?}", post);

        let encoded: Vec<u8> = post.ser_iter().collect();

        println!("LEN: {:?}", encoded.len());
        assert_eq!(encoded.len(), POST_SIZE);

        println!("ENCODED:");
        for i in 0..encoded.len() {
            println!("{:4}: {:#04x}", i, encoded[i]);
        }

        let mut stream = encoded.iter().copied();
        let reconstructed = Post::deser_from_iter(&mut stream).unwrap();
        assert_eq!(post, reconstructed);
        assert_eq!(post.hash(), post.hash());
        assert!(!stream.next().is_some());
    }

    #[test]
    fn post_hash_score() {
        let prev = U256::from_little_endian(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
            19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
        ]);
        let work = U256::from(0x04050607u64);
        let body: Body = Body::from([0x42; BODY_SIZE]);

        let post = Post::new(&prev, &work, &body);
        let hash = post.hash();
        println!("HASH: {:x}", hash);
        println!("SCORE: {}", hash_score(hash));
        // TODO assertion
    }

    #[test]
    fn genesis_post_hash_is_zero() {
        use std::default::Default;
        let genesis = Post::default();
        assert_eq!(genesis.hash(), U256::zero());
    }

    #[test]
    fn addresses_ser() {
        use std::net::IpAddr;
        let ips: Vec<IpAddr> = vec![
            "2804:d45:e0e5:8100:a42e:8a4:3e95:deaf".parse().unwrap(),
            "200.137.85.200".parse().unwrap(),
        ];

        let addrs: Vec<Address> = ips
            .iter()
            .map(|ip| Address::IP(ip.clone(), 42000))
            .collect();

        let encoded: Vec<u8> = addrs.ser_iter().collect();

        for i in 0..encoded.len() {
            println!("{:4}: {}", i, encoded[i]);
            // print!("{:02x}", encoded[i]);
        }
        println!("");

        assert_eq!(encoded[..],
            hex!("0228040d45e0e58100a42e08a43e95deaf00000000000000000000ffffc88955c8"));

        println!("ips: {:?}", ips);
    }

    #[test]
    fn message_ping() {
        let ips: Vec<std::net::IpAddr> = vec![
            "2804:d45:e0e5:8100:a42e:8a4:3e95:deaf".parse().unwrap(),
            "200.137.85.200".parse().unwrap(),
        ];

        let addrs: Vec<Address> = ips
            .iter()
            .map(|ip| Address::IP(ip.clone(), 42000))
            .collect();
        let message = Message::Ping(addrs);
        let encoded: Vec<u8> = message.ser_iter().collect();
        for i in 0..encoded.len() {
            println!("{:4}: {}", i, encoded[i]);
        }
        assert_eq!(encoded[..],
            hex!("000228040d45e0e58100a42e08a43e95deaf00000000000000000000ffffc88955c8"));
    }
}
