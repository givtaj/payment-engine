use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_sample_transactions() {
    let mut cmd = Command::cargo_bin("payments_engine").unwrap();

    cmd.arg("tests/data/sample_transactions.csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("1,5.5,0,5.5,false"));
}

#[test]
fn test_dispute_flow() {
    let mut cmd = Command::cargo_bin("payments_engine").unwrap();

    cmd.arg("tests/data/dispute_flow.csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("2,20.0,0.0,20.0,false"));
}

#[test]
fn test_chargeback_flow() {
    let mut cmd = Command::cargo_bin("payments_engine").unwrap();

    cmd.arg("tests/data/chargeback_flow.csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("3,0.0,0.0,0.0,true"));
}

#[test]
fn test_insufficient_funds() {
    let mut cmd = Command::cargo_bin("payments_engine").unwrap();

    cmd.arg("tests/data/insufficient_funds.csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("4,0.0,0,0.0,false").not());
}

#[test]
fn test_duplicate_tx_ids() {
    let mut cmd = Command::cargo_bin("payments_engine").unwrap();

    cmd.arg("tests/data/duplicate_tx_ids.csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("5,30.0,0,30.0,false"));
}

#[test]
fn test_deposit_withdraw_dispute_chargeback_flow() {
    // Prepare test input file
    let csv_content = "\
type,client,tx,amount
deposit,42,100,10.0
withdrawal,42,101,10.0
dispute,42,100,
chargeback,42,100,
";

    let input_path = "test_transactions.csv";
    std::fs::write(input_path, csv_content).unwrap();

    // Run the binary with the input file
    let output = Command::new("cargo")
        .args(["run", "--", input_path])
        .output()
        .expect("Failed to run payments_engine");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Engine output:\n{}", stdout);

    // Verify output contains locked account with correct balances
    assert!(stdout.contains("42"));
    assert!(stdout.contains("locked")); // Check locked column is present

    // Clean up test file
    std::fs::remove_file(input_path).unwrap();
}
