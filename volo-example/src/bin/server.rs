#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;
use volo_example::LogLayer;

use volo_example::S;

#[volo::main]
async fn main() {
    let addr: SocketAddr = "127.0.0.1:10818".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::volo::example::ItemServiceServer::new(S)
        .layer_front(LogLayer)
        .run(addr)
        .await
        .unwrap();
}
