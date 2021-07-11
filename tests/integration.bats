# These loads do not work from `setup_file`
setup() {
  load 'support/bats-support/load' # this is required by bats-assert!
  load 'support/bats-assert/load'
}

@test "addition using bc" {
  result="$(echo 2+2 | bc)"
  [ "$result" -eq 4 ]
}