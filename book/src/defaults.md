# Defaults

All run-time options can be configured with custom defaults. For example, you may want to default to the the host name of your local development environment, only requiring that `--host` be set when running against a production environment. Assuming your local development environment is at "http://local.dev/" you can do this as follows:

```rust
    SwanlingAttack::initialize()?
        .register_taskset(taskset!("LoadtestTasks")
            .register_task(task!(loadtest_index))
        )
        .set_default(SwanlingDefault::Host, "http://local.dev/")?
        .execute()?
        .print();

    Ok(())
```

The following defaults can be configured with a `&str`:
 - host: `SwanlingDefault::Host`
 - log file name: `SwanlingDefault::LogFile`
 - html-formatted report file name: `SwanlingDefault::ReportFile`
 - requests log file name: `SwanlingDefault::RequestsFile`
 - requests log file format: `SwanlingDefault::RequestsFormat`
 - debug log file name: `SwanlingDefault::DebugFile`
 - debug log file format: `SwanlingDefault::DebugFormat`
 - host to bind telnet Controller to: `SwanlingDefault::TelnetHost`
 - host to bind WebSocket Controller to: `SwanlingDefault::WebSocketHost`
 - host to bind Manager to: `SwanlingDefault::ManagerBindHost`
 - host for Worker to connect to: `SwanlingDefault::ManagerHost`

The following defaults can be configured with a `usize` integer:
 - total users to start: `SwanlingDefault::Users`
 - users to start per second: `SwanlingDefault::HatchRate`
 - how often to print running metrics: `SwanlingDefault::RunningMetrics`
 - number of seconds for test to run: `SwanlingDefault::RunTime`
 - log level: `SwanlingDefault::LogLevel`
 - verbosity: `SwanlingDefault::Verbose`
 - maximum requests per second: `SwanlingDefault::ThrottleRequests`
 - number of Workers to expect: `SwanlingDefault::ExpectWorkers`
 - port to bind telnet Controller to: `SwanlingDefault::TelnetPort`
 - port to bind WebSocket Controller to: `SwanlingDefault::WebSocketPort`
 - port to bind Manager to: `SwanlingDefault::ManagerBindPort`
 - port for Worker to connect to: `SwanlingDefault::ManagerPort`

The following defaults can be configured with a `bool`:
 - do not reset metrics after all users start: `SwanlingDefault::NoResetMetrics`
 - do not track metrics: `SwanlingDefault::NoMetrics`
 - do not track task metrics: `SwanlingDefault::NoTaskMetrics`
 - do not start telnet Controller thread: `SwanlingDefault::NoTelnet`
 - do not start WebSocket Controller thread: `SwanlingDefault::NoWebSocket`
 - do not autostart load test, wait instead for a Controller to start: `SwanlingDefault::NoAutoStart`
 - track status codes: `SwanlingDefault::StatusCodes`
 - follow redirect of base_url: `SwanlingDefault::StickyFollow`
 - enable Manager mode: `SwanlingDefault::Manager`
 - ignore load test checksum: `SwanlingDefault::NoHashCheck`
 - enable Worker mode: `SwanlingDefault::Worker`

The following defaults can be configured with a `SwanlingCoordinatedOmissionMitigation`:
 - default Coordinated Omission Mitigation strategy: `SwanlingDefault::CoordinatedOmissionMitigation`

For example, without any run-time options the following load test would automatically run against `local.dev`, logging metrics to `swanling-metrics.log` and debug to `swanling-debug.log`. It will automatically launch 20 users in 4 seconds, and run the load test for 15 minutes. Metrics will be displayed every minute during the test and will include additional status code metrics. The order the defaults are set is not important.

```rust
    SwanlingAttack::initialize()?
        .register_taskset(taskset!("LoadtestTasks")
            .register_task(task!(loadtest_index))
        )
        .set_default(SwanlingDefault::Host, "local.dev")?
        .set_default(SwanlingDefault::RequestsFile, "swanling-requests.log")?
        .set_default(SwanlingDefault::DebugFile, "swanling-debug.log")?
        .set_default(SwanlingDefault::Users, 20)?
        .set_default(SwanlingDefault::HatchRate, 4)?
        .set_default(SwanlingDefault::RunTime, 900)?
        .set_default(SwanlingDefault::RunningMetrics, 60)?
        .set_default(SwanlingDefault::StatusCodes, true)?
        .execute()?
        .print();

    Ok(())
```
