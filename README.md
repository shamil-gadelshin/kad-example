# kad-example

This project was created as an example to reproduce a possible Kademlia bug: https://github.com/libp2p/rust-libp2p/issues/3048

It contains the miminal kademlia setup based on the libp2p v.0.46.1 Kademlia example. It was later upgraded to the latest to date version - 0.49.0.

The project shows that `start_providing` method produces warnings in contrast with `put_record` or `get_closest_peers`: https://github.com/shamil-gadelshin/kad-example/blob/613564ee773d78dc47cfaf30547d5fbf9300fcce/src/minimal_kademlia.rs#L92
