/// Parse command-line arguments for input CSV file path
pub fn parse_file_path_from_cli_args() -> String {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <transactions.csv>", args[0]);
        std::process::exit(1);
    }

    args[1].clone()
}
