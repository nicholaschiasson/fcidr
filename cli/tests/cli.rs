use assert_cmd::Command;
use predicates::prelude::*;

// Helper to get the binary and prepare a Command
fn bin() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("fcidr")
}

#[test]
fn difference_from_stdin_yields_expected() {
    let mut cmd = bin();
    cmd.arg("difference")
        .arg("10.0.0.0/9")
        .write_stdin("10.0.0.0/8\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("10.128.0.0/9\n"));
}

#[test]
fn union_normalizes_output() {
    // Adding a subnet to an already included supernet shouldn't duplicate output
    let mut cmd = bin();
    cmd.arg("union")
        .arg("10.0.128.0/24")
        .write_stdin("10.0.0.0/8\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("10.0.0.0/8\n"));
}

#[test]
fn alias_minus_works() {
    let mut cmd = bin();
    cmd.arg("-").arg("10.0.0.0/9").write_stdin("10.0.0.0/8\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("10.128.0.0/9\n"));
}

#[test]
fn alias_plus_works() {
    // Union using '+' alias should produce both disjoint ranges
    let mut cmd = bin();
    cmd.arg("+").arg("127.0.0.0/16").write_stdin("10.0.0.0/8\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("10.0.0.0/8\n127.0.0.0/16\n"));
}

#[test]
fn alias_bang_complement_works() {
    // Complement using '!' alias should exclude the input range
    let mut cmd = bin();
    cmd.arg("!").write_stdin("10.0.0.0/8\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0.0.0.0/5\n"))
        .stdout(predicate::str::contains("128.0.0.0/1\n"))
        .stdout(predicate::str::contains("10.0.0.0/8").not());
}

#[test]
fn complement_with_positional_cidr_works() {
    // Provide CIDR as positional argument and run complement subcommand
    let mut cmd = bin();
    cmd.arg("10.0.0.0/8").arg("complement");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0.0.0.0/5\n"))
        .stdout(predicate::str::contains("128.0.0.0/1\n"))
        .stdout(predicate::str::contains("10.0.0.0/8").not());
}

#[test]
fn difference_with_positional_cidrs_works() {
    let mut cmd = bin();
    cmd.arg("10.0.0.0/8").arg("difference").arg("10.0.0.0/9");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("10.128.0.0/9\n"));
}

#[test]
fn union_with_positional_cidrs_normalizes() {
    // Union of a subnet with its supernet should still output only the supernet
    let mut cmd = bin();
    cmd.arg("10.0.0.0/8").arg("union").arg("10.0.128.0/24");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("10.0.0.0/8\n"));
}

#[test]
fn superset_with_positional_cidrs_success() {
    let mut cmd = bin();
    cmd.arg("255.0.0.0/16").arg("superset").arg("255.0.1.2/32");
    cmd.assert().success().stdout(predicate::str::is_empty());
}

#[test]
fn superset_with_positional_cidrs_failure() {
    let mut cmd = bin();
    cmd.arg("255.0.0.0/16").arg("superset").arg("255.1.1.2/32");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not a superset of 255.1.1.2/32"));
}

#[test]
fn superset_alias_gt_success() {
    let mut cmd = bin();
    cmd.arg("255.0.0.0/16").arg(">").arg("255.0.1.2/32");
    cmd.assert().success().stdout(predicate::str::is_empty());
}

#[test]
fn superset_alias_gt_failure() {
    let mut cmd = bin();
    cmd.arg("255.0.0.0/16").arg(">").arg("255.1.1.2/32");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not a superset of 255.1.1.2/32"));
}

#[test]
fn complement_with_multiline_stdin_full_range_becomes_empty() {
    // Two halves of the full IPv4 space; complement should yield no output
    let mut cmd = bin();
    cmd.arg("complement")
        .write_stdin("0.0.0.0/1\n128.0.0.0/1\n");
    cmd.assert().success().stdout(predicate::str::is_empty());
}

#[test]
fn complement_with_multiline_stdin_produces_expected_output() {
    // Two disjoint quarters; complement should output the other two quarters
    let mut cmd = bin();
    cmd.arg("complement")
        .write_stdin("0.0.0.0/2\n128.0.0.0/2\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("64.0.0.0/2\n"))
        .stdout(predicate::str::contains("192.0.0.0/2\n"))
        .stdout(predicate::str::contains("0.0.0.0/2").not())
        .stdout(predicate::str::contains("128.0.0.0/2").not());
}

#[test]
fn difference_with_multiline_stdin_preserves_other_ranges() {
    // Start with two ranges; remove the left half of 10.0.0.0/8
    let mut cmd = bin();
    cmd.arg("difference")
        .arg("10.0.0.0/9")
        .write_stdin("10.0.0.0/8\n127.0.0.0/16\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("10.128.0.0/9\n"))
        .stdout(predicate::str::contains("127.0.0.0/16\n"))
        .stdout(predicate::str::contains("10.0.0.0/9").not());
}

#[test]
fn union_with_multiline_stdin_normalizes() {
    // Adding a subnet already covered should not change output
    let mut cmd = bin();
    cmd.arg("union")
        .arg("10.0.128.0/24")
        .write_stdin("10.0.0.0/8\n127.0.0.0/16\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("10.0.0.0/8\n127.0.0.0/16\n"));
}

#[test]
fn superset_with_multiline_stdin_success() {
    let mut cmd = bin();
    cmd.arg("superset")
        .arg("255.0.1.2/32")
        .write_stdin("255.0.0.0/16\n127.0.0.0/8\n");
    cmd.assert().success().stdout(predicate::str::is_empty());
}

#[test]
fn superset_with_multiline_stdin_failure() {
    let mut cmd = bin();
    cmd.arg("superset")
        .arg("255.1.1.2/32")
        .write_stdin("255.0.0.0/16\n127.0.0.0/8\n");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not a superset of 255.1.1.2/32"));
}

#[test]
fn complement_without_stdin_yields_full_range_in_harness() {
    // In test harness, stdin is a pipe (non-terminal), so default -> complement prints full range
    let mut cmd = bin();
    cmd.arg("complement");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("0.0.0.0/0\n"));
}

#[test]
fn superset_success_exits_zero_and_prints_nothing() {
    let mut cmd = bin();
    cmd.arg("superset")
        .arg("255.0.1.2/32")
        .write_stdin("255.0.0.0/16\n");
    cmd.assert().success().stdout(predicate::str::is_empty());
}

#[test]
fn superset_failure_exits_nonzero_and_errors() {
    let mut cmd = bin();
    cmd.arg("superset")
        .arg("255.1.1.2/32")
        .write_stdin("255.0.0.0/16\n");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not a superset of 255.1.1.2/32"));
}
