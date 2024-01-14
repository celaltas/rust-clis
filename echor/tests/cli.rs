use std::fs;

use assert_cmd::Command;
use predicates::prelude::predicate;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn dies_no_args() -> TestResult {
    let mut cmd = Command::cargo_bin("echor")?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("USAGE"));
    Ok(())
}

fn run(outfile: &str, args: &[&str]) -> TestResult {
    let expected = fs::read_to_string(outfile)?;
    let mut cmd = Command::cargo_bin("echor")?;
    cmd.args(args).assert().success().stdout(expected);
    Ok(())
}

#[test]
fn hello1() -> TestResult {
    run("tests/expected/hello1.txt", &["Hello there"])
}

#[test]
fn hello2() -> TestResult {
    run("tests/expected/hello2.txt", &["Hello", "there"])
}
#[test]
fn hello1_no_newline() -> TestResult {
    run("tests/expected/hello1.n.txt", &["Hello  there", "-n"])
}
#[test]
fn hello2_no_newline() -> TestResult {
    run("tests/expected/hello2.n.txt", &["-n", "Hello", "there"])
}
