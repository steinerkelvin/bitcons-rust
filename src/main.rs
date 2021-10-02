use tokio;

#[tokio::main]
async fn main(){
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:42000").await.unwrap();

    // socket.

    println!("Hello, world!");
}
