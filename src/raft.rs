//! Developers wanting to understand the Raft algorithm will be aided by the [Raft Consensus Simulator](https://observablehq.com/@stwind/raft-consensus-simulator)
//! Developing will be aided by having three mental models
//! 1. the [Raft type](https://docs.rs/async-raft/0.5.0/async_raft/raft/struct.Raft.html) these are all of the interfaces, methods etc.
//! 2. the [RaftStorage trait](https://docs.rs/async-raft/0.5.0/async_raft/storage/trait.RaftStorage.html) the [memstore]() crate is a great example of this, except it is in-memory only.
//! 3. the [RaftNetwork trait](https://docs.rs/async-raft/0.5.0/async_raft/network/trait.RaftNetwork.html)
//!