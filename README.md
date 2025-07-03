# Payments Engine

## Overview

This project implements a simple payments engine that processes a stream of transactions from a CSV input and outputs the resulting account balances as a CSV. It is designed to be **asynchronous** and handle large inputs gracefully.

### Features

- Processes deposits, withdrawals, disputes, resolves, and chargebacks.
- Maintains accurate available, held, and total balances per client.
- Locks accounts upon chargebacks (preventing further transactions).
- Reads and processes transactions in a streaming (chunked) fashion to efficiently handle large CSV inputs.
- Employs asynchronous, non-blocking processing via the Tokio runtime for maximum throughput.
- Outputs final account states to `stdout` in CSV format.

---

## Problem Summary

The engine:

1. Reads a CSV file with transaction records (`type`, `client`, `tx`, `amount`).
2. Applies each transaction to the respective client account according to the business rules.
3. Outputs the final state of all client accounts in CSV format with columns:
   - `client`, `available`, `held`, `total`, `locked`.

For example, an output could look like:

```csv
client,available,held,total,locked
1,1.5000,0.0000,1.5000,false
2,2.0000,0.0000,2.0000,false
```

Example input snippet:

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

Where `transactions.csv` is your input file containing transactions, and the output is written to `accounts.csv`. You can also run the release binary directly for better performance:

```bash
./target/release/payments_engine transactions.csv > accounts.csv
```

---

## Dependencies

| Crate                                                 | Purpose                                     |
| ----------------------------------------------------- | ------------------------------------------- |
| [csv](https://crates.io/crates/csv)                   | Efficient CSV parsing and writing           |
| [serde](https://crates.io/crates/serde)               | Serialization/deserialization of records    |
| [rust_decimal](https://crates.io/crates/rust_decimal) | Precise decimal arithmetic for money values |
| [tokio](https://crates.io/crates/tokio)               | Asynchronous runtime for non-blocking tasks |
| [bytes](https://crates.io/crates/bytes)               | Efficient byte buffer management for I/O    |

---

## Design Decisions

- **Precision:** Uses `rust_decimal` to handle monetary values precisely, avoiding floating-point rounding errors.
- **Streaming:** Transactions are processed in a streaming fashion (line-by-line) using the `csv` crate, which allows the engine to handle large datasets efficiently without loading everything into memory at once.
- **Data Structures:** Internally uses Rust collections (`HashMap` for client accounts and transaction records) for quick lookups and updates.
- **Concurrency:** Utilizes the Tokio asynchronous runtime (Rust’s _green threads_) to avoid blocking operations. The engine’s state (client accounts and pending transactions) is owned by a dedicated async task. The main thread (or an input task) reads the CSV and sends each transaction to this state-owning task via message passing (e.g., channels). This design ensures the main thread remains free (performing I/O and other tasks) and that heavy processing is done off the main thread. The cooperative scheduling in Tokio means tasks yield when waiting on I/O or computation, preventing any single task from monopolizing the CPU.
- **Chunked I/O:** The engine handles input in chunks to manage large files (for example, a CSV with tens of thousands of rows). It reads and processes data in segments rather than loading the entire file into memory. This is facilitated by internal buffering and can leverage the `bytes` crate for efficient byte handling. By processing the CSV in small portions and yielding between chunks (e.g., via `tokio::task::yield_now()`), the engine minimizes memory usage and avoids long blocking periods.
- **Safety:** All logic is implemented in safe Rust with thorough error handling (e.g., gracefully handling invalid transactions). No `unsafe` code is used in this project.

---

## Assumptions

- Only **deposit** transactions can be disputed (i.e., a dispute references a previous deposit).
- Transactions in the input CSV are provided in chronological order.
- Invalid dispute, resolve, or chargeback operations (e.g., referencing a non-existent or already resolved transaction) are gracefully ignored.
- Once an account is locked due to a chargeback, it remains locked and ignores any subsequent transactions.
- The engine processes one input stream/file per run (no concurrent multi-file processing in the current scope).

---

## Testing

Unit and integration tests are provided in the `tests/` directory to verify correct handling of all transaction types and edge cases.

Run tests with:

```bash
cargo test
```

The tests cover scenarios like normal transactions, disputes and resolutions, chargeback effects, and ensure that the final account outputs match expected results.

---

## Future Improvements

- **CLI Enhancements:** Add more robust command-line argument parsing (using a crate like `clap`) to support specifying input/output files, logging levels, etc.
- **Performance Tuning:** Investigate optimizations such as batch processing of transactions or using memory-mapped files for extremely large datasets.
- **Extended Integrations:** Integrate with persistent storage (e.g. a database) for transaction records, enabling the engine to recover state or handle transactions beyond a single run.
- **Service Mode:** Evolve the engine into an async service (REST/gRPC) that can ingest transactions in real-time, building on the current Tokio-based design.

_(The need for handling multiple input files or streams concurrently is not in scope for now, but the architecture could be extended to handle that if required.)_

---

## Notes

The current implementation uses streaming CSV parsing via the `csv` crate, reading input line by line to ensure low memory usage even with large datasets. Combined with asynchronous processing using Tokio, the engine can process large volumes of transactions without blocking the main thread. These design choices fulfill the project requirements efficiently. In the future, this architecture can serve as a foundation for a fully concurrent server-based payments system if needed.

---

## Author

Aidin – 2025
