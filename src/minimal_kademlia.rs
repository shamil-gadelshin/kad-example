// Copyright 2018 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! Demonstrates how to perform Kademlia queries on the IPFS network.
//!
//! You can pass as parameter a base58 peer ID to search for. If you don't pass any parameter, a
//! peer ID will be generated randomly.

use async_std::task;
use futures::{select, StreamExt};
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{GetClosestPeersError, Kademlia, KademliaConfig, KademliaEvent, QueryResult};
use libp2p::{
    development_transport, identity,
    swarm::{Swarm, SwarmEvent},
    Multiaddr, PeerId,
};
use std::thread::sleep;
use std::{ time::Duration};
use crate::mpsc::UnboundedReceiver;

pub async fn start_swarm(
    listen_on: Multiaddr,
    keypair: identity::Keypair,
    dial_peer: PeerId,
    dial_address: Multiaddr,
    mut command_receiver: UnboundedReceiver<()>,
) {
    println!(
        "Starting swarm on {:?}. PeerId = {:?}",
        listen_on,
        keypair.public().to_peer_id()
    );

    // Create a random key for ourselves.
    let local_peer_id = PeerId::from(keypair.public());

    // Set up a an encrypted DNS-enabled TCP Transport over the Mplex protocol
    let transport = development_transport(keypair).await.unwrap();

    // Create a swarm to manage peers and events.
    let mut swarm = {
        // Create a Kademlia behaviour.
        let mut cfg = KademliaConfig::default();
        cfg.set_query_timeout(Duration::from_secs(5 * 60));
        let store = MemoryStore::new(local_peer_id);
        let mut behaviour: Kademlia<MemoryStore> = Kademlia::with_config(local_peer_id, store, cfg);

        behaviour.add_address(&dial_peer, dial_address);

        Swarm::new(transport, behaviour, local_peer_id)
    };

    swarm.listen_on(listen_on).unwrap();

    sleep(Duration::from_secs(1));

    // Order Kademlia to search for a random peer.
    let to_search: PeerId = identity::Keypair::generate_ed25519().public().into();

    println!("Searching for the closest peers to {:?}", to_search);
    swarm.behaviour_mut().get_closest_peers(to_search);

    // Kick it off!
    task::block_on(async move {
        loop {
            select! {
                event = swarm.select_next_some() => handle_swarm_event(local_peer_id, event).await,
                _ = command_receiver.next() => {
                    println!("PeerId = {:?}. Command received.", local_peer_id);

                    let key: libp2p::kad::record::Key = PeerId::random().to_bytes().into();

                    // This produces warnings
                    swarm.behaviour_mut().start_providing(key).unwrap();

                    // This works (with some delay)
                    // swarm.behaviour_mut().put_record(libp2p::kad::record::Record {
                    //     key,
                    //     value: vec![12,23,34],
                    //     publisher: None,
                    //     expires: None,
                    // }, libp2p::kad::Quorum::One).unwrap();

                    // This works
                    //swarm.behaviour_mut().get_closest_peers(to_search);

                    println!("PeerId = {:?}. Command finished..", local_peer_id);
                }
            }
        }
    })
}

async fn handle_swarm_event<O>(local_peer_id: PeerId, event: SwarmEvent<KademliaEvent, O>) {
    if let SwarmEvent::Behaviour(KademliaEvent::OutboundQueryCompleted {
        result: QueryResult::GetClosestPeers(result),
        ..
    }) = event
    {
        match result {
            Ok(ok) => {
                if !ok.peers.is_empty() {
                    println!("PeerId: {:?}. Query finished with closest peers: {:#?}",local_peer_id, ok.peers)
                } else {
                    // The example is considered failed as there
                    // should always be at least 1 reachable peer.
                    println!("PeerId: {:?}. Query finished with no closest peers.",local_peer_id,)
                }
            }
            Err(GetClosestPeersError::Timeout { peers, .. }) => {
                if !peers.is_empty() {
                    println!("PeerId: {:?}. Query timed out with closest peers: {:#?}",local_peer_id, peers)
                } else {
                    // The example is considered failed as there
                    // should always be at least 1 reachable peer.
                    println!("PeerId: {:?}. Query timed out with no closest peers.",local_peer_id,);
                }
            }
        };
    }
}
