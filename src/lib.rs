mod common;

use common::Body;

#[cfg(test)]
mod tests {
    use super::Body;

    #[test]
    fn it_works() {
        let _body = Body::new([0; 1024]);
    }
}
