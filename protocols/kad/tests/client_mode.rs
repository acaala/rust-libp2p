use libp2p_identify as identify;
use libp2p_identity as identity;
use libp2p_kad::store::MemoryStore;
use libp2p_kad::{Kademlia, KademliaConfig, KademliaEvent, Mode};
use libp2p_swarm::Swarm;
use libp2p_swarm_test::SwarmExt;

#[async_std::test]
async fn connection_to_node_in_client_mode_does_not_update_routing_table() {
    let mut client = Swarm::new_ephemeral(MyBehaviour::client);
    let mut server = Swarm::new_ephemeral(MyBehaviour::server);

    server.listen().await;
    client.connect(&mut server).await;

    let server_peer_id = *server.local_peer_id();

    match libp2p_swarm_test::drive(&mut client, &mut server).await {
        (
            [MyBehaviourEvent::Identify(_), MyBehaviourEvent::Identify(_), MyBehaviourEvent::Kad(KademliaEvent::RoutingUpdated { peer, .. })],
            [MyBehaviourEvent::Identify(_), MyBehaviourEvent::Identify(_)],
        ) => {
            assert_eq!(peer, server_peer_id)
        }
        other => panic!("Unexpected events: {other:?}"),
    }
}

#[derive(libp2p_swarm::NetworkBehaviour)]
#[behaviour(prelude = "libp2p_swarm::derive_prelude")]
struct MyBehaviour {
    identify: identify::Behaviour,
    kad: Kademlia<MemoryStore>,
}

impl MyBehaviour {
    fn client(k: identity::Keypair) -> Self {
        let mut config = KademliaConfig::default();
        config.set_mode(Mode::Client);

        Self::with_config(k, config)
    }

    fn server(k: identity::Keypair) -> Self {
        Self::with_config(k, KademliaConfig::default())
    }

    fn with_config(k: identity::Keypair, config: KademliaConfig) -> MyBehaviour {
        let local_peer_id = k.public().to_peer_id();

        Self {
            identify: identify::Behaviour::new(identify::Config::new(
                "/test/1.0.0".to_owned(),
                k.public(),
            )),
            kad: Kademlia::with_config(local_peer_id, MemoryStore::new(local_peer_id), config),
        }
    }
}