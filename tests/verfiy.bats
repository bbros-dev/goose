# These loads do not work from `setup_file`
setup() {
  load 'support/bats-support/load' # this is required by bats-assert!
  load 'support/bats-assert/load'
}

@test "BATS is good to go" {
    run echo test
    assert_output "test"
}
