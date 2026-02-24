use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

fn main() {
    // Specify the path to your log file here
    let log_path = format!("{}/access.log", std::env::var("HOME").unwrap_or_default());

    // HashMap to store our aggregations: Key = (Minute, Internal IP), Value = Request Count
    let mut utilization_data: HashMap<(String, String), usize> = HashMap::new();

    if let Ok(lines) = read_lines(&log_path) {
        for line in lines.map_while(Result::ok) {
            // 1. Filter by access point
            if !line.contains("/sap/opu/odata/sap/HCMFAB") {
                continue;
            }

            // 2. Extract Minute and Internal IP
            let minute = extract_minute(&line);
            let ip = line.split_whitespace().nth(2);

            // 3. Aggregate if both fields were successfully extracted
            if let (Some(m), Some(ip)) = (minute, ip) {
                *utilization_data
                    .entry((m, ip.parse().unwrap()))
                    .or_insert(0) += 1;
            }
        }
    } else {
        eprintln!("Error: Could not read the file '{log_path}'. Please check the path.");
        return;
    }

    // Print the aggregated report
    print_report(&utilization_data);
}

// Helper to open and read the file line by line safely
fn read_lines(filename: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
    Ok(io::BufReader::new(File::open(filename)?).lines())
}

// Extracts the minute from the leading timestamp bracket
fn extract_minute(line: &str) -> Option<String> {
    if line.starts_with('[') {
        let end_idx = line.find(']')?;
        let timestamp_full = &line[1..end_idx];

        // timestamp_full example: "21/Feb/2026:19:48:21 +0100"
        let parts: Vec<&str> = timestamp_full.split(':').collect();
        if parts.len() >= 3 {
            // Reconstruct up-to-the-minute: "21/Feb/2026:19:48"
            return Some(format!("{}:{}:{}", parts[0], parts[1], parts[2]));
        }
    }
    None
}

// Sorts and formats the output into a clean table
fn print_report(data: &HashMap<(String, String), usize>) {
    println!("{:<20} | {:<15} | Requests", "Time (Minute)", "Internal IP");
    println!("{:-<20}-+-{:-<15}-+-{:-<10}", "", "", "");

    // Convert HashMap to a Vector so we can sort chronologically
    let mut sorted_results: Vec<_> = data.iter().collect();
    sorted_results.sort_by_key(|(k, _)| *k);

    for ((minute, ip), count) in sorted_results {
        println!("{minute:<20} | {ip:<15} | {count}");
    }
}
