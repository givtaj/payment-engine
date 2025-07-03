# Payments Engine

## Overview

This project implements a simple payments engine that processes a stream of transactions from a CSV input and outputs the resulting account balances as a CSV. It is designed to be **asynchronous** and handle large inputs efficiently.

### Features

- Processes deposits, withdrawals, disputes, resolves, and chargebacks.
- Maintains accurate available, held, and total balances per client.
- Locks accounts upon chargebacks, preventing any further transactions.
- Reads and processes transactions in a streaming fashion to efficiently handle large CSV inputs.
- Processes transactions sequentially to maintain correct ordering, using streaming CSV parsing for efficiency.
- Outputs final account states to `stdout` in CSV format.

---

## Problem Summary

The engine:

1. Reads a CSV file with transaction records (`type`, `client`, `tx`, `amount`).
2. Applies each transaction to the respective client account according to business rules.
3. Outputs the final state of all client accounts in CSV format with columns:
   - `client`, `available`, `held`, `total`, `locked`.

### Example Output

```csv
client,available,held,total,locked
1,1.5000,0.0000,1.5000,false
2,2.0000,0.0000,2.0000,false
```

### Example Input

```csv
type,client,tx,amount
deposit,1,1,1.5
deposit,2,2,2.0
withdrawal,1,3,0.5
dispute,1,1,
resolve,1,1,
chargeback,2,2,
```

---

## Build

Requires Rust (stable edition 2021 or later). Install via [rustup](https://rustup.rs).

```bash
cargo build --release
```

This will produce an optimized binary in the `target/release` directory.

---

## Run

```bash
cargo run -- transactions.csv > accounts.csv
```

Where `transactions.csv` is your input file containing transactions, and the output is written to `accounts.csv`.

---

## Goals

### Basics

- The application builds cleanly and uses idiomatic Rust (2021 edition).
- Reads input and writes output in the required CSV format.

### Completeness

- Handles **all transaction types**: deposit, withdrawal, dispute, resolve, chargeback.
- Enforces account locking after chargebacks.

### Correctness

- Transaction effects match specification rules precisely.
- All balances (available, held, total) computed correctly.
- Locked accounts ignore any further transactions.

### Safety and Robustness

- Uses `rust_decimal` for precise money calculations.
- Uses safe Rust throughout; no unsafe blocks.
- Graceful error handling: skips invalid transactions without crashing.

### Efficiency

- Processes input as a stream without loading the entire file into memory.
- Uses asynchronous tasks to overlap I/O and processing.

### Maintainability

- Clean module structure.
- Descriptive naming for enums, structs, and functions.
- Comprehensive unit tests covering edge cases.

---

## Assumptions

- Only **deposit** transactions can be disputed.
- Transactions occur chronologically as provided in the input CSV.
- Invalid dispute, resolve, or chargeback operations are ignored.
- **Chargeback only processes an active disputed transaction. If a transaction is resolved before chargeback, the chargeback has no effect.**
- Negative available balances are allowed if disputes move funds from already withdrawn deposits into held.
- Once an account is locked due to chargeback, it remains locked and ignores all subsequent transactions.
- Each run processes a single input file.

---

## Testing

Run tests with:

```bash
cargo test
```

Tests cover normal transaction flows, disputes and resolutions, chargebacks, and locked account behavior to ensure correctness.

---

## Notes

The implementation uses streaming CSV parsing to efficiently handle large datasets with minimal memory footprint. Transactions are processed sequentially to preserve ordering. Tokio is included but multi-task concurrency is not utilized in this version.

---

## Author

Aidin – 2025
