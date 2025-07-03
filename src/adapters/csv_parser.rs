use std::fs::File;

/// Build CSV reader with desired configuration, Sets the capacity 32k for the buffer used in the CSV reader
pub fn build_csv_reader(path: &str) -> csv::Reader<File> {
    csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .buffer_capacity(32 * 1024)
        .from_path(path)
        .unwrap_or_else(|e| {
            eprintln!("Failed to open input file: {}", e);
            std::process::exit(1);
        })
}
