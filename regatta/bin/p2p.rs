// 1.50.0-nightly through 1.53.0-nightly
// #![feature(extended_key_value_attributes)]
// #![doc = include_str!("../../book/src/crate/bin/p2p.md")]

use libp2p::futures::StreamExt;
use libp2p::{
    core::upgrade,
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity,
    mdns::{Mdns, MdnsEvent},
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{NetworkBehaviourEventProcess, Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    NetworkBehaviour, PeerId, Transport,
};
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::collections::HashSet;
use tokio::{fs, io::AsyncBufReadExt, sync::mpsc};

use regatta::p2p::*;
use regatta::*;

pub async fn handle_shutdown(signals: signal_hook_tokio::Signals) {
    let mut signals = signals.fuse();
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT | SIGQUIT => {
                // Shutdown the system;
                //signal_hook::low_level::emulate_default_handler(SIGTSTP).await;
            },
            _ => unreachable!(),
        }
    }
}

#[tokio::main]
async fn main() -> ! {
    pretty_env_logger::init();

    let signals = Signals::new(&[
        SIGTERM,
        SIGINT,
        SIGQUIT,
    ]).unwrap();

    let handle = signals.handle();

    let signals_task = tokio::spawn(handle_shutdown(signals));

    info!("Peer Id: {}", regatta::p2p::PEER_ID.clone());
    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();

    let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&regatta::p2p::KEYS)
        .expect("can create auth keys");

    let transp = TokioTcpConfig::new()
        .nodelay(true)
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated()) // XX Handshake pattern, IX exists as well and IK - only XX currently provides interop with other libp2p impls
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    let mut behaviour = regatta::p2p::RecipeBehaviour {
        floodsub: Floodsub::new(regatta::p2p::PEER_ID.clone()),
        // mdns below used to have an `.expect("can create mdns")`
        mdns: Mdns::new(Default::default()).await.expect("Multi-cast DNS"),
        response_sender,
    };

    behaviour.floodsub.subscribe(regatta::p2p::TOPIC.clone());

    let mut swarm = SwarmBuilder::new(transp, behaviour, regatta::p2p::PEER_ID.clone())
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build();

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    swarm
        .listen_on(
            "/ip4/0.0.0.0/tcp/0"
                .parse()
                .expect("can get a local socket"),
        )
        .expect("swarm can be started");

    loop {
        let evt = {
            tokio::select! {
                line = stdin.next_line() => Some(regatta::p2p::EventType::Input(line.expect("can get line").expect("can read line from stdin"))),
                event = swarm.select_next_some() => {
                    info!("Swarm Event (unhandled): {:?}", event);
                    None
                },
                response = response_rcv.recv() => Some(regatta::p2p::EventType::Response(response.expect("response exists"))),
            }
        };

        if let Some(event) = evt {
            match event {
                regatta::p2p::EventType::Response(resp) => {
                    regatta::p2p::publish_event_type(resp, &mut swarm)
                }
                regatta::p2p::EventType::Input(line) => {
                    regatta::p2p::match_command_line(line, &mut swarm).await
                }
            }
        }
    }

    // Terminate the signal stream.
    handle.close();
    signals_task.await;
}
