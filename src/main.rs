use libp2p::identity;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use futures::channel::{mpsc};
use futures::SinkExt;

mod minimal_kademlia;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    println!("Starting...");

    let address1 = "/ip4/127.0.0.1/tcp/65001";
    let keypair1 = identity::Keypair::generate_ed25519();

    let address2 = "/ip4/127.0.0.1/tcp/65002";
    let keypair2 = identity::Keypair::generate_ed25519();

    let (mut _sender1, receiver1) = mpsc::unbounded();
    let (mut sender2, receiver2) = mpsc::unbounded();

    async_std::task::spawn(minimal_kademlia::start_swarm(
        address1.parse().unwrap(),
        keypair1.clone(),
        keypair2.public().to_peer_id(),
        address2.parse().unwrap(),
        receiver1
    ));

    async_std::task::spawn(minimal_kademlia::start_swarm(
        address2.parse().unwrap(),
        keypair2,
        keypair1.public().to_peer_id(),
        address1.parse().unwrap(),
        receiver2
    ));

    sleep(Duration::from_secs(3));

    for _ in 0..40 {
        sender2.send(()).await.unwrap();
        sleep(Duration::from_millis(100));
    }

    sleep(Duration::from_secs(50));

    Ok(())
}
