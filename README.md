# Swan

Swan is a tool for load testing and benchmarking websites and HTTP based micro-services.

The [Swan road-map](#road-map) envisions features that are not typically found in benchmarking or load testing tools.

[![crates.io](https://img.shields.io/crates/v/swanling.svg)](https://crates.io/crates/swanling)
[![Documentation](https://docs.rs/swanling/badge.svg)](https://docs.rs/swanling)
[![Apache-2.0 licensed](https://img.shields.io/crates/l/swanling.svg)](./LICENSE)
[![CI](https://github.com/BegleyBrothers/swanling/workflows/CI/badge.svg)](https://github.com/begleybrothers/swanling/actions?query=workflow%3ACI)
[![Docker Repository on Quay](https://quay.io/repository/begleybrothers/swanling/status "Docker Repository on Quay")](https://quay.io/repository/begleybrothers/swanling)

## Features

See [Road-map](#road-map) for forthcoming features.
These features can be relied on in the current major release series (v0):

- [nanomsg-NG](https://https://nng.nanomsg.org) based distributed workers.
- Coordinated omission mitigation

## Documentation

- [User documentation](https://swanling.io)
- [Developer documentation](https://docs.rs/swanling/)

## Road-map

These features are incomplete or off by default.

- [Robust to Coordinated omission]()
- [Histogram storage]()
- [Request models]()

## History

Swanling started as a fork of [Swanling](https://github.com/begleybrothers/swanling).
Many thanks to Jeremy Andrews and the other Swanling contributors:

- Jeremy Andrews
- Fabian Franz
- Narayan Newton
- Bernardo Uriarte Blanco
- David Laban
- jcarres-mdsol
- Alexander Liesenfeld
- nnewton
- Igor Davydenko
- Michael Edwards
- Vladislav Chugunov

## Alternatives

- [Apachebench](https://svn.apache.org/viewvc/httpd/httpd/branches/2.4.x/support/) (C)
- [Artillery](https://github.com/artilleryio/artillery) (NodeJS)
- [Autocannon](https://github.com/mcollina/autocannon) (NodeJS)
- [Bombardier](https://github.com/codesenberg/bombardie) (Go)
- [Cassowary](https://github.com/rogerwelin/cassowary) (Go)
- [Drill](https://github.com/fcsonline/drill) (Rust)
- [Gatling](https://github.com/gatling/gatling)
- [Swanling](https://github.com/begleybrothers/swanling) (Rust)
- [h2load](https://github.com/nghttp2/nghttp2) (C)
- [Hey](https://github.com/rakyll/hey) (Go)
- [Jmeter](https://github.com/apache/jmeter) (Java)
- [K6](https://github.com/k6io/k6) (Go)
- [Locust](https://github.com/locustio/locust) (Python)
- [Siege](https://github.com/JoeDog/siege) (C)
- [SlowHTTPTest](https://github.com/shekyan/slowhttptest)
- [Tsung](https://github.com/processone/tsung) (Erlang)
- [Vegeta](https://github.com/tsenart/vegeta) (Go)
- [Wrk2](https://github.com/giltene/wrk2) (C)
- [Wrk](https://github.com/wg/wrk) (C)
