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
// DEALINGS IN THE SOFTWARE.use futures::StreamExt;

use futures::StreamExt;
use libp2p::{
    identity,
    mdns::{Mdns, MdnsConfig, MdnsEvent, IPV6_MDNS_MULTICAST_ADDRESS},
    swarm::{Swarm, SwarmEvent},
    PeerId,
};
use std::error::Error;

async fn create_swarm(config: MdnsConfig) -> Result<Swarm<Mdns>, Box<dyn Error>> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    let transport = libp2p::development_transport(id_keys).await?;
    let behaviour = Mdns::new(config).await?;
    let mut swarm = Swarm::new(transport, behaviour, peer_id);
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    Ok(swarm)
}

async fn run_test(config: MdnsConfig) -> Result<(), Box<dyn Error>> {
    let mut a = create_swarm(config.clone()).await?;
    let mut b = create_swarm(config).await?;
    let mut discovered_a = false;
    let mut discovered_b = false;
    loop {
        futures::select! {
            ev = a.select_next_some() => match ev {
                SwarmEvent::Behaviour(MdnsEvent::Discovered(peers)) => {
                    for (peer, _addr) in peers {
                        if peer == *b.local_peer_id() {
                            if discovered_a {
                                return Ok(());
                            } else {
                                discovered_b = true;
                            }
                        }
                    }
                }
                _ => {}
            },
            ev = b.select_next_some() => match ev {
                SwarmEvent::Behaviour(MdnsEvent::Discovered(peers)) => {
                    for (peer, _addr) in peers {
                        if peer == *a.local_peer_id() {
                            if discovered_b {
                                return Ok(());
                            } else {
                                discovered_a = true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[async_std::test]
async fn test_discovery_async_std_ipv4() -> Result<(), Box<dyn Error>> {
    run_test(MdnsConfig::default()).await
}

#[async_std::test]
async fn test_discovery_async_std_ipv6() -> Result<(), Box<dyn Error>> {
    let mut config = MdnsConfig::default();
    config.multicast_addr = *IPV6_MDNS_MULTICAST_ADDRESS;
    run_test(MdnsConfig::default()).await
}

#[tokio::test]
async fn test_discovery_tokio_ipv4() -> Result<(), Box<dyn Error>> {
    run_test(MdnsConfig::default()).await
}

#[tokio::test]
async fn test_discovery_tokio_ipv6() -> Result<(), Box<dyn Error>> {
    let mut config = MdnsConfig::default();
    config.multicast_addr = *IPV6_MDNS_MULTICAST_ADDRESS;
    run_test(MdnsConfig::default()).await
}
