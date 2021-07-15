# Regatta

[Regatta (collective noun)](https://lenichoir.org/collective-nouns/)
: A regatta of swans.

[Raft](https://dictionary.cambridge.org/dictionary/english/raft)
: a small rubber or plastic boat that can be filled with air.
: a large number or range; a lot

[Regatta (noun)](https://dictionary.cambridge.org/dictionary/english/regatta)
: a boat race or series of races.

## Raft

Swanling treats the problem of recording the results of distributed load testing as a distributed storage problem.  Holding that opinion, it is natural to consider building on the [Raft (PDF)](https://raft.github.io/raft.pdf) algorithm, a consensus algorithm for distributed systems.  While the Raft algorithm is relatively new, Diego Ongaro's Ph.D. [dissertation](https://github.com/ongardie/dissertation#readme) being awarded in 2014, there are [several implementations](https://raft.github.io/#implementations) and it has been widely adopted in distributed computing applications, e.g. Hashicorp's [Consul, Nomad and Vault](https://www.hashicorp.com/resources/raft-consul-consensus-protocol-explained) and Kubernetes [etcd](https://etcd.io/).

## Network

By using Raft algorithm we simplify configuration and setup.  Of course the ability to send and receive data over a network is integral to the proper functionality of nodes within a Raft cluster.  Swanling assumes members of your cluster of virtual/physical machines, containers can exchange data with each other.  Swanling also assumes that DNS is used to resolve names to IP addresses.

Using those assumptions, Swanling provides several features that ease configuration and management:

- The regatta leader (hence followers) are elected automatically: No need to configure and track manager and worker nodes.
- The regatta size dynamically grows and shrinks.
- Within reason, data survives network outages and cluster members crashing or exiting the cluster.

### Libraries

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