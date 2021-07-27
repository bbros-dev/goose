# Developers

[Developer documentation (external link)](https://docs.rs/swanling/)

## Install Dependencies

```bash
cargo build
```

## Run Benchmarks

### P2P

```bash
cargo criterion
DATE_TAG=$(date --utc --iso-8601=date)
RUSTFLAGS="-C target-cpu=native" cargo bench -- p2p
find ./target -wholename "*/new/raw.csv" -print0 | \
  xargs -0 xsv cat rows > book/assets/benchmark-data_${DATE_TAG}.csv
```

```bash
pushd regatta
  cargo bench --bench p2p -- --save-baseline before
  critcmp --export before > ./benches/data/before.json
  # Change branches
  cargo bench --bench p2p -- --save-baseline after
  critcmp --export after > after.json
  critcmp before.json after.json
popd
```
