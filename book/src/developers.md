# Developers

[Developer documentation (external link)](https://docs.rs/swanling/)

## Install Dependencies

```bash
cargo build
```

## Build Documentation Site

```
~/.cargo/bin/mdbook build
```

## Run Benchmarks

### P2P

```bash
cargo criterion
DATE_TAG=$(date --utc +"%Y%m%d")
RUSTFLAGS="-C target-cpu=native" cargo bench -- p2p
find ./target -wholename "*/new/raw.csv" -print0 | \
  xargs -0 xsv cat rows > book/assets/benchmark-data_${DATE_TAG}.csv
```

```bash
DATE_TAG=$(date --utc +"%Y%m%d")
BENCH=p2p
pushd regatta
  cargo bench --bench ${BENCH} -- --save-baseline ${BENCH}-${DATE_TAG}
  critcmp --export ${BENCH}-${DATE_TAG} > benches/data/${BENCH}-${DATE_TAG}.json
  # Change branches
  cargo bench --bench p2p -- --save-baseline ${BENCH}-${DATE_TAG}a
  critcmp --export ${BENCH}-${DATE_TAG}a > ./benches/data/${BENCH}-${DATE_TAG}a.json
  critcmp ./benches/data/${BENCH}-${DATE_TAG}.json ./benches/data/${BENCH}-${DATE_TAG}a.json
popd
```

## Road Map

### Tender

Sub-command `status all` print status of all `regatta` peers.

Sub-command `status peer [id]` print status of `regatta` peer with ID `[id]`.

Sub-command `exit` exit `tender`, leave`regatta` peers running.

Sub command `shutdown all` shutdown all `regatta` peers

Sub command `shutdown peer [id]` shutdown `regatta` peer with ID `[id]`

Sub-command `calibrate` returns, as req/s:

- Ceiling: The absolute maximum that could be made by the current code-base.
- Overhead: The overhead required by the request scheduling logic.
- Limit (Ceiling - Overhead): A req/s value above the limit produces an error.
- Drift statistics: The difference between the time a request was scheduled and
  the time stamp before the request was made.

### Conjectures

If you are inclined to benchmarking conjectures and the experimental; we are
interested in whether performance is improved by adopting ideas embodied in
the following crates:

- [bitter](https://github.com/nickbabcock/bitter): Likely in the context of HDR, and may require a fork until proven.
- [fixed-buffer](https://crates.io/crates/fixed-buffer): As above.
- [heapless](https://github.com/japaric/heapless): Storage of serialized histograms
- [sharded-slab](https://crates.io/crates/sharded-slab): As above.
