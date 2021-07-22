use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::io::{self, Write};
use std::process::Command; // Run programs
use tempfile::NamedTempFile;

