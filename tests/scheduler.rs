use httpmock::{Method::GET, MockRef, MockServer};
use serial_test::serial;
use tokio::time::{sleep, Duration};

mod common;

use swanling::prelude::*;
use swanling::SwanlingConfiguration;

// Paths used in load tests performed during these tests.
const ONE_PATH: &str = "/one";
const TWO_PATH: &str = "/two";
const THREE_PATH: &str = "/three";
const START_ONE_PATH: &str = "/start/one";
const STOP_ONE_PATH: &str = "/stop/one";

// Indexes to the above paths.
const ONE_KEY: usize = 0;
const TWO_KEY: usize = 1;
const THREE_KEY: usize = 2;
const START_ONE_KEY: usize = 3;
const STOP_ONE_KEY: usize = 4;

// Load test configuration.
const EXPECT_WORKERS: usize = 4;
// Users needs to be an even number.
const USERS: usize = 18;
const RUN_TIME: usize = 3;

// There are two test variations in this file.
#[derive(Clone)]
enum TestType {
    // Schedule multiple task sets.
    TaskSets,
    // Schedule multiple tasks.
    Tasks,
}

// Test task.
pub async fn one_with_delay(user: &SwanlingUser) -> SwanlingTaskResult {
    let _swanling = user.get(ONE_PATH).await?;

    // "Run out the clock" on the load test when this function runs. Sleep for
    // the total duration the test is to run plus 1 second to be sure no
    // additional tasks will run after this one.
    sleep(Duration::from_secs(RUN_TIME as u64 + 1)).await;

    Ok(())
}

// Test task.
pub async fn two_with_delay(user: &SwanlingUser) -> SwanlingTaskResult {
    let _swanling = user.get(TWO_PATH).await?;

    // "Run out the clock" on the load test when this function runs. Sleep for
    // the total duration the test is to run plus 1 second to be sure no
    // additional tasks will run after this one.
    sleep(Duration::from_secs(RUN_TIME as u64 + 1)).await;

    Ok(())
}

// Test task.
pub async fn three(user: &SwanlingUser) -> SwanlingTaskResult {
    let _swanling = user.get(THREE_PATH).await?;

    Ok(())
}

// Used as a test_start() function, which always runs one time.
pub async fn start_one(user: &SwanlingUser) -> SwanlingTaskResult {
    let _swanling = user.get(START_ONE_PATH).await?;

    Ok(())
}

// Used as a test_stop() function, which always runs one time.
pub async fn stop_one(user: &SwanlingUser) -> SwanlingTaskResult {
    let _swanling = user.get(STOP_ONE_PATH).await?;

    Ok(())
}

// All tests in this file run against common endpoints.
fn setup_mock_server_endpoints(server: &MockServer) -> Vec<MockRef> {
    vec![
        // First set up ONE_PATH, store in vector at ONE_KEY.
        server.mock(|when, then| {
            when.method(GET).path(ONE_PATH);
            then.status(200);
        }),
        // Next set up TWO_PATH, store in vector at TWO_KEY.
        server.mock(|when, then| {
            when.method(GET).path(TWO_PATH);
            then.status(200);
        }),
        // Next set up THREE_PATH, store in vector at THREE_KEY.
        server.mock(|when, then| {
            when.method(GET).path(THREE_PATH);
            then.status(200);
        }),
        // Next set up START_ONE_PATH, store in vector at START_ONE_KEY.
        server.mock(|when, then| {
            when.method(GET).path(START_ONE_PATH);
            then.status(200);
        }),
        // Next set up STOP_ONE_PATH, store in vector at STOP_ONE_KEY.
        server.mock(|when, then| {
            when.method(GET).path(STOP_ONE_PATH);
            then.status(200);
        }),
    ]
}

// Build appropriate configuration for these tests.
fn common_build_configuration(
    server: &MockServer,
    worker: Option<bool>,
    manager: Option<usize>,
) -> SwanlingConfiguration {
    if let Some(expect_workers) = manager {
        common::build_configuration(
            &server,
            vec![
                "--manager",
                "--expect-workers",
                &expect_workers.to_string(),
                "--users",
                &USERS.to_string(),
                "--hatch-rate",
                &USERS.to_string(),
                "--run-time",
                &RUN_TIME.to_string(),
                "--no-reset-metrics",
            ],
        )
    } else if worker.is_some() {
        common::build_configuration(&server, vec!["--worker"])
    } else {
        common::build_configuration(
            &server,
            vec![
                "--users",
                &USERS.to_string(),
                "--hatch-rate",
                &USERS.to_string(),
                "--run-time",
                &RUN_TIME.to_string(),
                "--no-reset-metrics",
            ],
        )
    }
}

// Helper to confirm all variations generate appropriate results.
fn validate_test(test_type: &TestType, scheduler: &SwanlingScheduler, mock_endpoints: &[MockRef]) {
    // START_ONE_PATH is loaded one and only one time on all variations.
    mock_endpoints[START_ONE_KEY].assert_hits(1);

    match test_type {
        TestType::TaskSets => {
            // Now validate scheduler-specific counters.
            match scheduler {
                SwanlingScheduler::RoundRobin => {
                    // We launch an equal number of each task set, so we call both endpoints
                    // an equal number of times.
                    mock_endpoints[TWO_KEY].assert_hits(mock_endpoints[ONE_KEY].hits());
                    mock_endpoints[ONE_KEY].assert_hits(USERS / 2);
                }
                SwanlingScheduler::Serial => {
                    // As we only launch as many users as the weight of the first task set, we only
                    // call the first endpoint, never the second endpoint.
                    mock_endpoints[ONE_KEY].assert_hits(USERS);
                    mock_endpoints[TWO_KEY].assert_hits(0);
                }
                SwanlingScheduler::Random => {
                    // When scheduling task sets randomly, we don't know how many of each will get
                    // launched, but we do now that added together they will equal the total number
                    // of users.
                    assert!(
                        mock_endpoints[ONE_KEY].hits() + mock_endpoints[TWO_KEY].hits() == USERS
                    );
                }
            }
        }
        TestType::Tasks => {
            // Now validate scheduler-specific counters.
            match scheduler {
                SwanlingScheduler::RoundRobin => {
                    // Tests are allocated round robin THREE, TWO, ONE. There's no delay
                    // in THREE, so the test runs THREE and TWO which then times things out
                    // and prevents ONE from running.
                    mock_endpoints[ONE_KEY].assert_hits(0);
                    mock_endpoints[TWO_KEY].assert_hits(USERS);
                    mock_endpoints[THREE_KEY].assert_hits(USERS);
                }
                SwanlingScheduler::Serial => {
                    // Tests are allocated sequentally THREE, TWO, ONE. There's no delay
                    // in THREE and it has a weight of 2, so the test runs THREE twice and
                    // TWO which then times things out and prevents ONE from running.
                    mock_endpoints[ONE_KEY].assert_hits(0);
                    mock_endpoints[TWO_KEY].assert_hits(USERS);
                    mock_endpoints[THREE_KEY].assert_hits(USERS * 2);
                }
                SwanlingScheduler::Random => {
                    // When scheduling task sets randomly, we don't know how many of each will get
                    // launched, but we do now that added together they will equal the total number
                    // of users (THREE_KEY isn't counted as there's no delay).
                    assert!(
                        mock_endpoints[ONE_KEY].hits() + mock_endpoints[TWO_KEY].hits() == USERS
                    );
                }
            }
        }
    }

    // STOP_ONE_PATH is loaded one and only one time on all variations.
    mock_endpoints[STOP_ONE_KEY].assert_hits(1);
}

// Returns the appropriate taskset, start_task and stop_task needed to build these tests.
fn get_tasksets() -> (SwanlingTaskSet, SwanlingTaskSet, SwanlingTask, SwanlingTask) {
    (
        taskset!("TaskSetOne")
            .register_task(task!(one_with_delay))
            .set_weight(USERS)
            .unwrap(),
        taskset!("TaskSetTwo")
            .register_task(task!(two_with_delay))
            // Add one to the weight to avoid this getting reduced by gcd.
            .set_weight(USERS + 1)
            .unwrap(),
        // Start runs before all other tasks, regardless of where defined.
        task!(start_one),
        // Stop runs after all other tasks, regardless of where defined.
        task!(stop_one),
    )
}

// Returns a single SwanlingTaskSet with two SwanlingTasks, a start_task, and a stop_task.
fn get_tasks() -> (SwanlingTaskSet, SwanlingTask, SwanlingTask) {
    (
        taskset!("TaskSet")
            .register_task(task!(three).set_weight(USERS * 2).unwrap())
            .register_task(task!(two_with_delay).set_weight(USERS).unwrap())
            .register_task(task!(one_with_delay).set_weight(USERS).unwrap()),
        // Start runs before all other tasks, regardless of where defined.
        task!(start_one),
        // Stop runs after all other tasks, regardless of where defined.
        task!(stop_one),
    )
}

// Helper to run all standalone tests.
fn run_standalone_test(test_type: &TestType, scheduler: &SwanlingScheduler) {
    // Start the mock server.
    let server = MockServer::start();

    // Setup the mock endpoints needed for this test.
    let mock_endpoints = setup_mock_server_endpoints(&server);

    // Build common configuration.
    let configuration = common_build_configuration(&server, None, None);

    let swanling_attack;
    match test_type {
        TestType::TaskSets => {
            // Get the tasksets, start and stop tasks to build a load test.
            let (taskset1, taskset2, start_task, stop_task) = get_tasksets();
            // Set up the common base configuration.
            swanling_attack = crate::SwanlingAttack::initialize_with_config(configuration)
                .unwrap()
                .register_taskset(taskset1)
                .register_taskset(taskset2)
                .test_start(start_task)
                .test_stop(stop_task)
                .set_scheduler(scheduler.clone());
        }
        TestType::Tasks => {
            // Get the taskset, start and stop tasks to build a load test.
            let (taskset1, start_task, stop_task) = get_tasks();
            // Set up the common base configuration.
            swanling_attack = crate::SwanlingAttack::initialize_with_config(configuration)
                .unwrap()
                .register_taskset(taskset1)
                .test_start(start_task)
                .test_stop(stop_task)
                .set_scheduler(scheduler.clone());
        }
    }

    // Run the Swanling Attack.
    common::run_load_test(swanling_attack, None);

    // Confirm the load test ran correctly.
    validate_test(test_type, &scheduler, &mock_endpoints);
}

// Helper to run all gaggle tests.
fn run_gaggle_test(test_type: &TestType, scheduler: &SwanlingScheduler) {
    // Start the mock server.
    let server = MockServer::start();

    // Setup the mock endpoints needed for this test.
    let mock_endpoints = setup_mock_server_endpoints(&server);

    // Build common configuration.
    let worker_configuration = common_build_configuration(&server, Some(true), None);

    let swanling_attack;
    match test_type {
        TestType::TaskSets => {
            // Get the tasksets, start and stop tasks to build a load test.
            let (taskset1, taskset2, start_task, stop_task) = get_tasksets();
            // Set up the common base configuration.
            swanling_attack = crate::SwanlingAttack::initialize_with_config(worker_configuration)
                .unwrap()
                .register_taskset(taskset1)
                .register_taskset(taskset2)
                .test_start(start_task)
                .test_stop(stop_task)
                .set_scheduler(scheduler.clone());
        }
        TestType::Tasks => {
            // Get the taskset, start and stop tasks to build a load test.
            let (taskset1, start_task, stop_task) = get_tasks();
            // Set up the common base configuration.
            swanling_attack = crate::SwanlingAttack::initialize_with_config(worker_configuration)
                .unwrap()
                .register_taskset(taskset1)
                .test_start(start_task)
                .test_stop(stop_task)
                .set_scheduler(scheduler.clone());
        }
    }

    // Workers launched in own threads, store thread handles.
    let worker_handles = common::launch_gaggle_workers(swanling_attack, EXPECT_WORKERS);

    // Build Manager configuration.
    let manager_configuration = common_build_configuration(&server, None, Some(EXPECT_WORKERS));

    let manager_swanling_attack;
    match test_type {
        TestType::TaskSets => {
            // Get the tasksets, start and stop tasks to build a load test.
            let (taskset1, taskset2, start_task, stop_task) = get_tasksets();
            // Build the load test for the Manager.
            manager_swanling_attack =
                crate::SwanlingAttack::initialize_with_config(manager_configuration)
                    .unwrap()
                    .register_taskset(taskset1)
                    .register_taskset(taskset2)
                    .test_start(start_task)
                    .test_stop(stop_task)
                    .set_scheduler(scheduler.clone());
        }
        TestType::Tasks => {
            // Get the taskset, start and stop tasks to build a load test.
            let (taskset1, start_task, stop_task) = get_tasks();
            // Build the load test for the Manager.
            manager_swanling_attack =
                crate::SwanlingAttack::initialize_with_config(manager_configuration)
                    .unwrap()
                    .register_taskset(taskset1)
                    .test_start(start_task)
                    .test_stop(stop_task)
                    .set_scheduler(scheduler.clone());
        }
    }

    // Run the Swanling Attack.
    common::run_load_test(manager_swanling_attack, Some(worker_handles));

    // Confirm the load test ran correctly.
    validate_test(test_type, &scheduler, &mock_endpoints);
}

#[test]
// Load test with multiple tasks allocating SwanlingTaskSets in round robin order.
fn test_round_robin_taskset() {
    run_standalone_test(&TestType::TaskSets, &SwanlingScheduler::RoundRobin);
}

#[test]
#[cfg_attr(not(feature = "gaggle"), ignore)]
#[serial]
// Load test with multiple tasks allocating SwanlingTaskSets in round robin order, in
// Regatta mode.
fn test_round_robin_taskset_gaggle() {
    run_gaggle_test(&TestType::TaskSets, &SwanlingScheduler::RoundRobin);
}

#[test]
// Load test with multiple SwanlingTasks allocated in round robin order.
fn test_round_robin_task() {
    run_standalone_test(&TestType::Tasks, &SwanlingScheduler::RoundRobin);
}

#[test]
#[cfg_attr(not(feature = "gaggle"), ignore)]
#[serial]
// Load test with multiple SwanlingTasks allocated in round robin order, in
// Regatta mode.
fn test_round_robin_task_gaggle() {
    run_gaggle_test(&TestType::Tasks, &SwanlingScheduler::RoundRobin);
}

#[test]
// Load test with multiple tasks allocating SwanlingTaskSets in serial order.
fn test_serial_taskset() {
    run_standalone_test(&TestType::TaskSets, &SwanlingScheduler::Serial);
}

#[test]
#[cfg_attr(not(feature = "gaggle"), ignore)]
#[serial]
// Load test with multiple tasks allocating SwanlingTaskSets in serial order, in
// Regatta mode.
fn test_serial_taskset_gaggle() {
    run_gaggle_test(&TestType::TaskSets, &SwanlingScheduler::Serial);
}

#[test]
// Load test with multiple SwanlingTasks allocated in serial order.
fn test_serial_tasks() {
    run_standalone_test(&TestType::Tasks, &SwanlingScheduler::Serial);
}

#[test]
// Load test with multiple tasks allocating SwanlingTaskSets in random order.
fn test_random_taskset() {
    run_standalone_test(&TestType::TaskSets, &SwanlingScheduler::Random);
}

#[test]
#[cfg_attr(not(feature = "gaggle"), ignore)]
#[serial]
// Load test with multiple tasks allocating SwanlingTaskSets in random order, in
// Regatta mode.
fn test_random_taskset_gaggle() {
    run_gaggle_test(&TestType::TaskSets, &SwanlingScheduler::Random);
}

#[test]
// Load test with multiple tasks allocating SwanlingTaskSets in random order.
fn test_random_tasks() {
    run_standalone_test(&TestType::Tasks, &SwanlingScheduler::Random);
}
