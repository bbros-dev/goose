# Distributed Load Test

Swanling also supports distributed load testing. A Regatta is one Swanling process running in Manager mode, and 1 or more Swanling processes running in Worker mode. The Manager coordinates starting and stopping the Workers, and collects aggregated metrics. Regatta support is a cargo feature that must be enabled at compile-time as documented below. To launch a Regatta, you must copy your load test application to all servers from which you wish to generate load.

It is strongly recommended that the same load test application be copied to all servers involved in a Regatta. By default, Swanling will verify that the load test is identical by comparing a hash of all load test rules. Telling it to skip this check can cause the load test to panic (for example, if a Worker defines a different number of tasks or task sets than the Manager).

## Regatta Compile-time Feature

Regatta support is a compile-time Cargo feature that must be enabled. Swanling uses the [`nng`](https://docs.rs/nng/) library to manage network connections, and compiling `nng` requires that `cmake` be available.

The `gaggle` feature can be enabled from the command line by adding `--features gaggle` to your cargo command.

When writing load test applications, you can default to compiling in the Regatta feature in the `dependencies` section of your `Cargo.toml`, for example:

```toml
[dependencies]
swanling = { version = "^0.12", features = ["gaggle"] }
```

## Regatta Manager

To launch a Regatta, you first must start a Swanling application in Manager mode. All configuration happens in the Manager. To start, add the `--manager` flag and the `--expect-workers` flag, the latter necessary to tell the Manager process how many Worker processes it will be coordinating. For example:

```
cargo run --features gaggle --example simple -- --manager --expect-workers 2 --host http://local.dev/ -v
```

This configures a Swanling Manager to listen on all interfaces on the default port (0.0.0.0:5115) for 2 Swanling Worker processes.

## Regatta Worker

At this time, a Swanling process can be either a Manager or a Worker, not both. Therefor, it usually makes sense to launch your first Worker on the same server that the Manager is running on. If not otherwise configured, a Swanling Worker will try to connect to the Manager on the localhost. This can be done as follows:

```
cargo run --features gaggle --example simple -- --worker -v
```

In our above example, we expected 2 Workers. The second Swanling process should be started on a different server. This will require telling it the host where the Swanling Manager process is running. For example:

```
cargo run --example simple -- --worker --manager-host 192.168.1.55 -v
```

Once all expected Workers are running, the distributed load test will automatically start. We set the `-v` flag so Swanling provides verbose output indicating what is happening. In our example, the load test will run until it is canceled. You can cancel the Manager or either of the Worker processes, and the test will stop on all servers.

## Regatta Run-time Flags

* `--manager`: starts a Swanling process in Manager mode. There currently can only be one Manager per Regatta.
* `--worker`: starts a Swanling process in Worker mode. How many Workers are in a given Regatta is defined by the `--expect-workers` option, documented below.
* `--no-hash-check`: tells Swanling to ignore if the load test application doesn't match between Worker(s) and the Manager. This is not recommended, and can cause the application to panic.

The `--no-metrics`, `--only-summary`, `--no-reset-metrics`, `--status-codes`, and `--no-hash-check` flags must be set on the Manager. Workers inherit these flags from the Manager

## Regatta Run-time Options

* `--manager-bind-host <manager-bind-host>`: configures the host that the Manager listens on. By default Swanling will listen on all interfaces, or `0.0.0.0`.
* `--manager-bind-port <manager-bind-port>`: configures the port that the Manager listens on. By default Swanling will listen on port `5115`.
* `--manager-host <manager-host>`: configures the host that the Worker will talk to the Manager on. By default, a Swanling Worker will connect to the localhost, or `127.0.0.1`. In a distributed load test, this must be set to the IP of the Swanling Manager.
* `--manager-port <manager-port>`: configures the port that a Worker will talk to the Manager on. By default, a Swanling Worker will connect to port `5115`.

The `--users`, `--hatch-rate`, `--host`, and `--run-time` options must be set on the Manager. Workers inherit these options from the Manager.

The `--throttle-requests` option must be configured on each Worker, and can be set to a different value on each Worker if desired.

## Technical Details

Swanling uses [`nng`](https://docs.rs/nng/) to send network messages between the Manager and all Workers. [Serde](https://docs.serde.rs/serde/index.html) and [Serde CBOR](https://github.com/pyfisch/cbor) are used to serialize messages into [Concise Binary Object Representation](https://tools.ietf.org/html/rfc7049).

Workers initiate all network connections, and push metrics to the Manager process.