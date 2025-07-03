mod adapters;
mod engine;

mod models;

use engine::runner;

#[tokio::main]
async fn main() {
    let file_path = adapters::cli::parse_file_path_from_cli_args();

    let mut csv_reader = adapters::csv_parser::build_csv_reader(&file_path);

    let (cmd_tx, engine_handle) = runner::setup_engine();

    runner::send_commands_to_engine(&mut csv_reader, cmd_tx).await;

    runner::finalize_engine(engine_handle).await;
}
