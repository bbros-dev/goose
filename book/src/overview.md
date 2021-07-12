# Overview

Swanling is a HTTP load testing tool inspired by [Locust](https://locust.io/) and [Taurus](https://github.com/Blazemeter/taurus). User behavior is defined with standard Rust code. Load tests are applications that have a dependency on the Swanling library. Web requests are made with the [Reqwest](https://docs.rs/reqwest) HTTP Client.

## Rust Requirements

Minimum required `rustc` version is `1.49.0`.
`swanling` depends on [`flume`](https://docs.rs/flume) for communication between threads, which in turn depends on [`spinning_top`](https://docs.rs/spinning_top) which uses `hint::spin_loop` which stabilized in `rustc` version `1.49.0`.
More detail in [Rust issue #55002](https://github.com/rust-lang/rust/issues/55002).

## Road-map

These features are incomplete or off by default.

- [Robust to Coordinated Omission](coordinated-ommission.md)
  - Initially, CO is mitigated by backfill. Raw (biased) and CO-adjusted (less biased) data is provided.
  - Subsequently, CO will by explicitly guarded against. Raw data will be unbiased.
- [Histogram storage](histograms.md)
  - Initially, ([uncorrected]()) [HDRHistogram]()
  - Subsequently, [DynaHist]()
- [Request models]() that guard against CO
  - Initially, deterministic and uniform.
  - Subsequently, configurable random distributions.

## History

Swanling started as a fork of [Goose](https://github.com/tag1consulting/goose).
Many thanks to Jeremy Andrews and the other Goose contributors:

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
- [HttpPerf](https://github.com/httperf/httperf)
- [Jmeter](https://github.com/apache/jmeter) (Java)
- [K6](https://github.com/k6io/k6) (Go)
- [Locust](https://github.com/locustio/locust) (Python)
- [Siege](https://github.com/JoeDog/siege) (C)
- [SlowHTTPTest](https://github.com/shekyan/slowhttptest)
- [Taurus](https://gettaurus.org/docs/Index/)
- [Tsung](https://github.com/processone/tsung) (Erlang)
- [Vegeta](https://github.com/tsenart/vegeta) (Go)
- [Wrk2](https://github.com/giltene/wrk2) (C)
- [Wrk](https://github.com/wg/wrk) (C)
