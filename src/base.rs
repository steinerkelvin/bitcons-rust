#![allow(dead_code)]

use primitive_types::U256;
use sha3::Digest;

pub const WORD_SIZE: usize = 32; // 256 bits
pub const BODY_SIZE: usize = 1280;
pub const POST_SIZE: usize = 2 * WORD_SIZE + BODY_SIZE;

// ipv6 address size
pub const IPV6_SIZE: usize = 16;
pub const PORT_SIZE: usize = 2;
pub const ADDRESS_SIZE: usize = IPV6_SIZE + PORT_SIZE;

pub fn address_list_size(num_addr: usize) -> usize {
    // byte for list size
    1 + num_addr * ADDRESS_SIZE
}
pub fn message_ping_size(num_addr: usize) -> usize {
    // byte for message code
    1 + address_list_size(num_addr)
}
pub const MESSAGE_REQUEST_POST_SIZE: usize = 1 + WORD_SIZE;
pub const MESSAGE_SHARE_POST_SIZE: usize = 1 + PORT_SIZE;

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
    fn from(val: [u8; BODY_SIZE]) -> Self {
        Body { val }
    }
}

// Post

#[derive(Debug, Default, PartialEq)]
pub struct Post {
    prev: U256, // previous post (32 bytes)
    work: U256, // extra info and nonce (32 bytes)
    body: Body, // post contents (1280 bytes)
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
        let sered: Vec<u8> = vec![]; // TODO
        let hasher = sha3::Keccak256::new();
        let hash = hasher.chain(&sered).finalize();
        let res = U256::from_little_endian(&hash);
        res
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

#[derive(Debug, PartialEq)]
enum Address {
    IP(std::net::IpAddr, u16),
}

// Message

use bitvec::prelude as bv;

#[derive(Debug, PartialEq)]
struct Slice {
    nonc: u64,
    data: bv::BitVec,
}

#[derive(Debug, PartialEq)]
enum Message {
    PutPeers(Vec<Address>),
    PutSlice(Slice),
    PutBlock(Post),
    AskBlock(U256),
}

pub const MESSAGE_PING_CODE: u8 = 1;
pub const MESSAGE_REQUEST_POST_CODE: u8 = 2;
pub const MESSAGE_SHARE_POST_CODE: u8 = 3;

#[cfg(test)]
mod tests {
    use super::*;
    // use hex_literal::hex;
    // use rand::prelude::*;

    // #[test]
    // fn post_seredere() {
    //     let mut rng = rand::thread_rng();
    //     let prev_arr = rng.gen::<[u8; 32]>();
    //     let prev = U256::from_little_endian(&prev_arr);

    //     let work = U256::from(0x04050607u64);
    //     let body: Body = Body::from([0x42; BODY_SIZE]);

    //     let post = Post::new(&prev, &work, &body);
    //     println!("POST: {:?}", post);

    //     let encoded: Vec<u8> = post.ser_iter().collect();

    //     println!("LEN: {:?}", encoded.len());
    //     assert_eq!(encoded.len(), POST_SIZE);

    //     println!("ENCODED:");
    //     for i in 0..encoded.len() {
    //         println!("{:4}: {:#04x}", i, encoded[i]);
    //     }

    //     let mut stream = encoded.iter().copied();
    //     let reconstructed = Post::deser_from_iter(&mut stream).unwrap();
    //     assert_eq!(post, reconstructed);
    //     assert_eq!(post.hash(), post.hash());
    //     assert!(!stream.next().is_some());
    // }

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

    // #[test]
    // fn genesis_post_hash_is_zero() {
    //     use std::default::Default;
    //     let genesis = Post::default();
    //     assert_eq!(genesis.hash(), U256::zero());
    // }

    // #[test]
    // fn addresses_ser() {
    //     use std::net::IpAddr;
    //     let ips: Vec<IpAddr> = vec![
    //         "2804:d45:e0e5:8100:a42e:8a4:3e95:deaf".parse().unwrap(),
    //         "200.137.85.200".parse().unwrap(),
    //     ];
    //     let addresses: Vec<Address> = ips
    //         .iter()
    //         .map(|ip| Address::IP(ip.clone(), 42000))
    //         .collect();

    //     let encoded: Vec<u8> = addresses.ser_iter().collect();
    //     // TODO test size

    //     for i in 0..encoded.len() {
    //         println!("{:4}: {}", i, encoded[i]);
    //         // print!("{:02x}", encoded[i]);
    //     }
    //     println!("");

    //     assert_eq!(encoded[..],
    //         hex!("0228040d45e0e58100a42e08a43e95deaf10a400000000000000000000ffffc88955c810a4"));

    //     let mut stream = encoded.iter().copied();
    //     let reconstructed =
    //         Vec::<Address>::deser_from_iter(&mut stream).unwrap();
    //     assert_eq!(addresses, reconstructed);
    // }

    //     #[test]
    //     fn message_ping() {
    //         let ips: Vec<std::net::IpAddr> = vec![
    //             "2804:d45:e0e5:8100:a42e:8a4:3e95:deaf".parse().unwrap(),
    //             "200.137.85.200".parse().unwrap(),
    //         ];
    //         let addresses: Vec<Address> = ips
    //             .iter()
    //             .map(|ip| Address::IP(ip.clone(), 42000))
    //             .collect();
    //         let message = Message::Ping(addresses);

    //         let encoded: Vec<u8> = message.ser_iter().collect();
    //         assert_eq!(encoded.len(), message_ping_size(ips.len()));

    //         for i in 0..encoded.len() {
    //             println!("{:4}: {}", i, encoded[i]);
    //         }

    //         assert_eq!(encoded[..],
    //             hex!("000228040d45e0e58100a42e08a43e95deaf00000000000000000000ffffc88955c8"));

    //         let mut stream = encoded.iter().copied();
    //         let reconstructed = Message::deser_from_iter(&mut stream).unwrap();
    //         assert_eq!(message, reconstructed);
    //     }
}
