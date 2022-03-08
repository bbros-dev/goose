# Debugging an async application

One novelty in Swanling is the calibration exercise, where in Swaling
tries to give you a reasonable upper bound on the number of requests per
second you might expect the current machine sustain.

In the course of developing the 'calibration' feature we noticed an intermittent 'hang'
of the Hyper client. Some Discord chats indicated we weren't alone.
Some existing reports (at the time of writing) of similar behavior ([#2312](https://github.com/hyperium/hyper/issues/2312), [#2419](https://github.com/hyperium/hyper/issues/2419)), and some previous reports ([#1358](https://github.com/hyperium/hyper/issues/1358)). However, the context and
use case didn't seem to exactly fit ours.

Which brings us to the topic of this post: how to go about debugging a Rust async application
that uses the Tokio runtime and the Hyper client.  Mostly this is a note to future self, so there will be lots of shell and Rust snippets.  If I have to do this again I'd like a less random walk to some useful information.

## History

1. We first observe the hang behavior in our code.  At this time it is not clear if the fault lies in out code or not.
1. We produce a standalone example using Criterion to generate samples, where the hang behavior always occurs at least once in 100 samples.  This excludes most of our code.  We still aren't sure if the issue lies with the Hyper client, Hyper server, Tokio runtime, Futures or Streams.

1. Remove the dependency on the Hyper server. Now we can replicate the Client hang using a 'simple' TCP server that always responds with HTTP 200 Hello World string. The string is stored in memory, eliminating any file IO.

1. 2021, Oct 17: After some effort the Criterion dependency is removed. Also remove the use of async-streams crate. We can produce the client hang on demand:

    ![Client hang, 17 Oct 2021](./images/hyper-client-hang-20211017.png)

    The CPU(blue) dive is the hang behavior, the Green(memory) indicates the process is still running.

## Objective

As hinted above, this will likely be a somewhat random walk to a, hopefully, productive end point: Identification of the root cause of some Hyper client hang behavior.  Ideally this leads to an open issue that is accepted.  Even better would be to identify a workaround.  Still better is some insight into what the fix might look like.

Lets begin. The following are our tools (in no particular order):

- `strace` (Linux utlity)
- strace-parser calibrate
- tracing and companion crates:
  - tracing-opentelemetry
  - Jaeger
  - Zipkin
- Jaeger (CNF incubated) container based analysis tool for opentelemtry data
- Zipkin container based analysis tool for opentelemtry data
- KDE's heap track and heap track gui.

Guidance, a.k.a prior art:

- [Hotspot](https://github.com/KDAB/hotspot)
- [The Performance Book - Profiling](https://nnethercote.github.io/perf-book/profiling.html)
- [Nick Babcock's Guidelines on Benchmarking and Rust]: https://nickb.dev/blog/guidelines-on-benchmarking-and-rust
- [Denis Bakhvalov's Top-Down performance analysis methodology]: https://easyperf.net/blog/2019/02/09/Top-Down-performance-analysis-methodology

We have a MRE of the hang behavior that we can trigger by setting a small enough value for the number of open files allowed: `ulimit -Sn 512`.

## Tracing: Jaeger & Zipkin

We instrument for both [Jaeger] and [Zipkin], not because you should but because it'll help us decide which tool we should adopt, or if we need both.  Hopefully this exercise will aid your decision making too.  There are multiple evaluations of these tools available from, in date order: [Thundera: Zipkin vs Jaeger (2020)], [Cisco (nee Epsagon): Zipkin vs Jaeger (2019)], [Bizety: Zipkin vs Jaeger (2019)], [Logz.io: Zipkin vs Jaeger (2018)].  None of those evaluations use each tool in an actual debugging example. So we'll do that.

Tracing (like logging and metrics) requires you insert additional code to trace the application behavior. However, to keep the MRE source 'uncluttered', we have created a separate example: `mre-tracing.rs`.  Different applications/organizations/teams will have different views on when tracing should be injected.  We leave that topic aside and are not suggesting the approach here should always be used.

We add OpenTelemetry tracing to the debug version of the MRE code `mre-tracing.rs`, and consume the output using both [Jaeger] and [Zipkin] containers, each parses and presents the traces produced.
Happily the Rust story around distributed tracing is mature enough that we can direct the data to two consumers (+ stdout when required) and then evaluate which provides the tools that best suit out needs.

Setting up mirrored tracing is common enough that it is an [example in the opentelemetry-rust project].  Better still, it is relatively simple.

There is one issue we need to be mindful about: we are trying to debug the behaviour of the hyper client, and distributed applications often use HTTP clients or libraries for data exchange.  Consequently, it is not surprising the Rust OpenTelemetry crate ecosystem make that client optional via feature flags, structs and traits. Unfortunately they do so in subtly different ways. We use the [Surf HTTP client] as the default OpenTelemetry collector HTTP client for the `opentelemetry`, `opentelemetry-jaeger`, and `opentelemetry-zipkin` crates. The primary reason for this choice is the fact Surf allows setting libcurl as its HTTP engine, and this eliminates any risk the Hyper client might become entangled in our debug tracing - the Reqwest client is built on the Hyper crate.  For this specific use case we prefer to brick over any possible confusion between the HTTP client used to trace and the HTTP client being traced.

Our first run produces errors from both Jaeger and Zipkin.  The Zipkin error is a more serious one from the container:

```bash
:: version 2.23.4 :: commit 3827477 ::2021-10-20 21:07:13.735  INFO [/] 1 --- [oss-http-*:9411] c.l.a.s.Server                           : Serving HTTP at /[0:0:0:0:0:0:0:0%0]:9411 - http://127.0.0.1:9411/
Terminating due to java.lang.OutOfMemoryError: Java heap space
```

There was no guidance on the Zipkin quick start page, yet it did look like we might be encountering a similar issue to that others report in [](https://github.com/openzipkin/zipkin-dependencies/issues/143), but we weren't sure and the Zipkin users on Gitter did not appear to know either, or at least were silent until the time we posted this. As an occasional Java users we kinda-sorta know the JVM memory arguments Java sometimes needs, but we also know Java is terrible at scaling down and making it happy is always an exercise that takes you two orders of magnitude more time than you expect. With Jaeger working we have no need for a slow start. Moving on.

The Jaeger three (3) types errors are logged (many times) to stdout:

```bash
Run Stream. Thread: ThreadId(2)
OpenTelemetry trace error occurred. Exporter jaeger encountered the following error(s): thrift agent failed with message too long
....<snip>....
OpenTelemetry trace error occurred. cannot send span to the batch span processor because the channel is full
...<snip>...
OpenTelemetry trace error occurred. ConnectFailed: failed to connect to the server
```

The Thrift `messages too long` error is reported in [opentelemetry-rust/issue/648], and the solution suggested works here: `OTEL_BSP_MAX_EXPORT_BATCH_SIZE=2` and we are able to proceed. We'll iterate on the optimal batch size when we have a more concrete idea of the message size the spans generate.
With the revised batch size the other two errors are resolved and we have successfully setup distributed tracing for our single file problem - now that is overkill, but we can use stable Rust, and can set this up on a developer laptop/desktop.
Next, set `JAEGER_DISABLED=true` to prevent traced Jaeger queries (`jaeger-query`) from cluttering the Jaeger UI.

The initial output is from:

```bash
OTEL_BSP_MAX_EXPORT_BATCH_SIZE=2 \
JAEGER_DISABLED=true \
RUST_BACKTRACE=1 \
RUST_LOG=trace \
HYPER=trace \
RUSTFLAGS="--cfg tokio_unstable" \
cargo run --example mre-tracing \
          -Z unstable-options \
          --profile dev \
          -- --nocapture &> mre-tracing.log
```

[Debugging Rust]: https://bitshifter.github.io/rr+rust/index.html#1
[Using Rust with rr]: https://gist.github.com/spacejam/15f27007c0b1bcc1d6b4c9169b18868c
[Bizety: Zipkin vs Jaeger (2019)]: https://www.bizety.com/2019/01/14/distributed-tracing-for-microservices-jaeger-vs-zipkin/
[Cisco (nee Epsagon): Zipkin vs Jaeger (2019)]: https://epsagon.com/observability/zipkin-or-jaeger-the-best-open-source-tools-for-distributed-tracing/
[Isahc HTTP client]: https://github.com/sagebind/isahc
[Logz: Zipkin vs Jaeger (2018)]: https://logz.io/blog/zipkin-vs-jaeger/
[Thundera: Zipkin vs Jaeger (2020)]: https://blog.thundra.io/decision-making-between-jaeger-and-zipkin
[Zipkin]: https://zipkin.io/
[example in the opentelemetry-rust project]: https://github.com/open-telemetry/opentelemetry-rust/tree/main/examples/multiple-span-processors


## Zipkin

```bash
podman run -d -p 9411:9411 openzipkin/zipkin
```

## Top-Down methodology

While Bakhvalov's "Top-Down performance analysis methodology" post deals with performance bottlenecks, we can cast indefinite hang behavior as the one performance bottleneck to rule them all.
With that in mind, as well as the different context and different tools, we can still adopt the key elements of the methodology:

1. Characterize our application

### Valgrind

```BASH
RUSTFLAGS="-g" cargo bench --norun --package regatta --bench reqs -- --nocapture
BENCH="./../target/release/reqs"
T_ID="Calibrate/calibrate-limit/10000"
valgrind --tool=callgrind \
         --dump-instr=yes \
         --collect-jumps=yes \
         --simulate-cache=yes \
         $BENCH --bench --profile-time 10 $T_ID
kcachegrind
```

#### pprof (Cargo Criterion)

The [pprof-rs] crate can generate profiles from Criterion benchmarks, and the output
data can be explored with the tooling developed around Go, `go tool pprof ...`
as well as the [SpeedScope] project.
These two features won us over.

The criterion option, `--profile-time 130`, is required to generate the pprof output
`profile.pb`.

```bash
cargo bench --bench reqs -- calibrate-limit --nocapture --profile-time 130
go tool pprof -http=:8080 ./../target/criterion/Calibrate/calibrate-limit/10000/profile/profie.pb
go tool pprof -svg profile300.gz ./../target/criterion/Calibrate/calibrate-limit/10000/profile/profie.pb
```

## Conclusion

If the tracing exercise whet your appetite, you might like our blog post on distributed tracing with the Consul service mesh, using Rust, of course.

https://github.com/hashicorp/consul/issues/8503
https://github.com/nicholasjackson/demo-consul-service-mesh/tree/master/metrics_tracing
https://github.com/hashicorp/consul-demo-tracing/tree/master/jaeger

https://academy.broadcom.com/blog/aiops/what-are-zipkin-and-jaeger-and-how-does-istio-use-them


[pprof-rs]: https://github.com/tikv/pprof-rs
[SpeedScope]: https://www.speedscope.app/
