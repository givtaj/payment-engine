use crate::{
    adapters::output::output_accounts,
    engine::state::State,
    models::{command::Command, transaction::TransactionInput},
};

use std::{fs::File, io};
use tokio::sync::mpsc;

/// Run the engine event loop to receive and handle commands, and then output results.
pub async fn run(mut rx: mpsc::Receiver<Command>) {
    let mut state = State::new();

    // Process incoming commands
    while let Some(cmd) = rx.recv().await {
        state.process_single_command(cmd);
    }

    // All commands processed, output final state of accounts as CSV
    output_accounts(&state.accounts, io::stdout());
}

/// Set up engine task and return its handle along with command sender
pub fn setup_engine() -> (mpsc::Sender<Command>, tokio::task::JoinHandle<()>) {
    let (cmd_tx, cmd_rx) = mpsc::channel(1000);

    let handle = tokio::spawn(async move {
        run(cmd_rx).await;
    });

    (cmd_tx, handle)
}

/// Read CSV, parse to commands, and send to engine
pub async fn send_commands_to_engine(
    csv_reader: &mut csv::Reader<File>,
    cmd_tx: mpsc::Sender<Command>,
) {
    let deserialize_iter = csv_reader.deserialize::<TransactionInput>();
    let mut record_count: usize = 0;

    for result in deserialize_iter {
        let input: TransactionInput = match result {
            Ok(rec) => rec,
            Err(e) => {
                eprintln!("CSV parsing error: {}", e);
                std::process::exit(1);
            }
        };

        let cmd = match input.to_command() {
            Ok(cmd) => cmd,
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        };

        if cmd_tx.send(cmd).await.is_err() {
            break;
        }

        record_count += 1;

        if record_count % 1000 == 0 {
            tokio::task::yield_now().await;
        }
    }

    // Close the channel to signal engine no more commands will arrive
    drop(cmd_tx);
}

/// Wait for engine task to finish processing and handle result
pub async fn finalize_engine(handle: tokio::task::JoinHandle<()>) {
    if let Err(e) = handle.await {
        eprintln!("Engine task error: {:?}", e);
        std::process::exit(1);
    }
}
