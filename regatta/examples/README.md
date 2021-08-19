# Examples

```bash
cargo run --example 01-pages-hello-ok
```

## Profiling

Guidance:

* [Hotspot](https://github.com/KDAB/hotspot)
* [The Performance Book - Profiling](https://nnethercote.github.io/perf-book/profiling.html)
* [Top-Down methodology](https://easyperf.net/blog/2019/02/09/Top-Down-performance-analysis-methodology)

## Call Stack

Install the utility `cargo +stable install cargo-call-stack`

Configure Cargo to generate required bit codes.  Without this you will observe:

```rust
error: failed to get bitcode from object file for LTO (Bitcode section not found in object file)
```

Generate a GraphViz dot file (must use `+nightly`):

```bash
RUSTFLAGS = "-C embed-bitcode" cargo +nightly call-stack --example 01-pages-hello-ok-c > 01-pages-hello-ok-c.dot
dot -Tsvg:cairo 01-pages-hello-ok-c.dot >01-pages-hello-ok-c.svg
```
