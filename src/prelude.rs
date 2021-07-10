//! A list of things that typically must be imported to write a Swanling load test.
//!
//! Instead of manually importing everything each time you write a Swanling load test,
//! you can simply import this prelude as follows:
//!
//! ```rust
//! use swanling::prelude::*;
//! ```

pub use crate::metrics::{SwanlingCoordinatedOmissionMitigation, SwanlingMetrics};
pub use crate::swanling::{
    SwanlingTask, SwanlingTaskError, SwanlingTaskFunction, SwanlingTaskResult, SwanlingTaskSet,
    SwanlingUser,
};
pub use crate::{
    task, taskset, SwanlingAttack, SwanlingDefault, SwanlingDefaultType, SwanlingError,
    SwanlingScheduler,
};
