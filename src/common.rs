pub struct Body {
    val: [u8; 1024],
}

impl Body {
    pub fn new(val: [u8; 1024]) -> Self {
        Body { val }
    }
}

pub struct Hash {
    val: [u8; 32],
}

impl Hash {
    pub fn new(val: [u8; 32]) -> Self {
        Hash { val }
    }
}

struct Post {
    prev: Hash, // previous post (32 bytes)
    work: [u8; 32], // extra info and nonce (32 bytes)
    body: Body, // post contents (1280 bytes)
}

impl Post {
    fn hash(&self) -> Hash {
        Hash::new([0; 32])
    }
}

#[cfg(test)]
mod tests {
    use super::Body;

    #[test]
    fn it_works() {
        let _body = Body::new([0; 1024]);
    }
}
