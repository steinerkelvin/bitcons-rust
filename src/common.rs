use primitive_types::U256;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone)]
struct Body {
    val: [u8; 1024],
}

impl Serialize for Body {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq_ser = serializer.serialize_seq(Some(1024))?;
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

#[derive(Debug, Serialize)]
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
        U256::from_little_endian(&[0; 32])
    }
}

#[cfg(test)]
mod tests {
    use primitive_types::U256;
    use rand::prelude::*;

    use super::{Body, Post};

    #[test]
    fn post_hash() {
        let mut rng = rand::thread_rng();
        let prev = U256::from(0);
        let buf = rng.gen::<[u8; 32]>();
        let work = U256::from_little_endian(&buf);

        let body: Body = Body::from([42; 1024]);

        let post = Post::new(&prev, &work, &body);
        // println!("POST: {:?}", post);
        println!("HASH: {:?}", post.hash());
    }
}
