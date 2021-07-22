# Regatta

[Regatta (collective noun)](https://lenichoir.org/collective-nouns/)
: A regatta of swans.

[Raft](https://dictionary.cambridge.org/dictionary/english/raft)
: a small rubber or plastic boat that can be filled with air.
: a large number or range; a lot

[Regatta (noun)](https://dictionary.cambridge.org/dictionary/english/regatta)
: a boat race or series of races.

## Overview

Swanling treats recording data about the requests and responses over the duration of distributed load test or benchmark, as a distributed storage problem in a peer to peer network.

Holding those opinions, it is natural to build on the [Raft (PDF)](https://raft.github.io/raft.pdf) algorithm, a consensus algorithm for distributed systems, and the [libp2p] system, a modular system of protocols, specifications and libraries that enable the development of peer-to-peer network applications.

Consequently, `regatta` is a peer-to-peer network CLI application, built using the Rust [Crate libp2p](https://crates.io/crates/libp2p) implementation of the [libp2p system](https://libp2p.io). Hence, Swanling uses [multi-cast DNS (mDNS)](https://docs.libp2p.io/reference/glossary/#mdns) to connect `regatta` peers.

## Requirements

### Network

The ability to send and receive data over a network is integral to the proper functionality of nodes within a peer-to-peer netwrok and a Raft cluster.  Swanling assumes the cluster of virtual machines, physical machines or containers can exchange data with each other - perioidcally, if not always.

### Security

Swanling assumes its network traffic is across a secured, trusted, private network.
There are no active plans alter this. While PR's are welcome, our current view is that network security is not trivial, and pseudo-security, or security theater creates more risks than it eliminates, while imposing real performance costs. That being said the transport between Regatta processes/instances does use the [Noise protocol](https://noiseprotocol.org/) and public key cryptography as the basis of [peer identity](https://docs.libp2p.io/reference/glossary/#peerid).

## Features

In addition to these requirements/assumptions, using the Raft algorithm further simplifies configuration and setup, and Swanling adopts several conventions that still further eases configuration and management.

- Dynamic leadership: The `regatta` leader (hence followers) are elected automatically. There is no need to configure and track manager and worker nodes.
- Dynamic regatta size: The `regatta` size can dynamically grow or shrink.
- Distributed Storage: Within reason, data can survive network outages and cluster members crashing or exiting the cluster.
- Configurable storage precision/size trade-offs: High Dynamic Range Histogram (HdrHistogram) provide storage that maintains a fixed (but not optimal) cost in both space and time.   The memory footprint depends solely on the dynamic range and precision chosen, not the number of data values.
- Throughput and response distributions: Both distributions are recorded in HdrHistogram data structures.

## Data persistence

Data is persisted in memory, then synchronized among raft cluster members.
Data is synchronized across the cluster a configurable number of seconds (`sync_interval`, default: 60 seconds).

### Raft Libraries

While the Raft algorithm is relatively new, Diego Ongaro's Ph.D. [dissertation](https://github.com/ongardie/dissertation#readme) being awarded in 2014, there are [several implementations](https://raft.github.io/#implementations) and it has been widely adopted in distributed computing applications, e.g. Hashicorp's [Consul, Nomad and Vault](https://www.hashicorp.com/resources/raft-consul-consensus-protocol-explained) and the Kubernetes [etcd](https://etcd.io/).

- ( 626) [async-raft (was actix-raft)](https://github.com/async-raft/async-raft)
- (  45) [lol](https://github.com/akiradeveloper/lol)
- (1.7k) [raft-rs (1)](https://github.com/tikv/raft-rs): Need to build Log, State Machine and Transport components
- (   4) [raft-rs (2)](https://github.com/simple-raft-rs/raft-rs)
- ( 183) [riteraft](https://github.com/ritelabs/riteraft)
- (  19) [Replicated State Machine](https://github.com/opaugam/rsm)

<!--
References
- https://www.alexgarrett.tech/blog/article/observer-pattern-in-rust/
- https://github.com/najamelan/pharos/tree/master -->