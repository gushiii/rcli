use std::fs;

use anyhow::Result;
use serde_json::{Map, Value};

use crate::cli::OutputFormat;

fn create_csv_reader(
    input: &str,
    header: bool,
    delimiter: &char,
) -> Result<csv::Reader<std::fs::File>> {
    let reader = csv::ReaderBuilder::new()
        .has_headers(header)
        .delimiter(*delimiter as u8)
        .from_path(input)?;
    Ok(reader)
}

pub fn process_csv(
    input: &str,
    header: bool,
    delimiter: &char,
    output: String,
    format: OutputFormat,
) -> Result<()> {
    let mut reader = create_csv_reader(input, header, delimiter)?;

    let mut ret = Vec::with_capacity(128);
    let headers = reader.headers()?.clone();
    for result in reader.records() {
        let record = result?;
        let json_value = headers.iter().zip(record.iter()).collect::<Value>();
        ret.push(json_value);
    }

    let content = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&ret)?,
        OutputFormat::Yaml => serde_yaml::to_string(&ret)?,
        OutputFormat::Toml => {
            let mut root_map = Map::new();
            root_map.insert("data".to_string(), Value::Array(ret.clone()));
            let root_value = Value::Object(root_map);
            toml::to_string(&root_value)?
        }
    };

    fs::write(&output, content)?;
    println!("✓ CSV converted successfully to: {}", output);

    Ok(())
}

pub fn process_csv_show(input: &str, header: bool, delimiter: &char) -> Result<()> {
    let mut reader = create_csv_reader(input, header, delimiter)?;

    let mut all_rows: Vec<Vec<String>> = Vec::new();
    let headers = if header {
        Some(
            reader
                .headers()?
                .clone()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    for result in reader.records() {
        let record = result?;
        all_rows.push(record.iter().map(|s| s.to_string()).collect());
    }

    let col_count = headers
        .as_ref()
        .map_or(all_rows.first().map_or(0, |row| row.len()), |h| h.len());

    let mut col_widths = vec![0; col_count];

    if let Some(ref h) = headers {
        for (i, header) in h.iter().enumerate() {
            col_widths[i] = col_widths[i].max(header.len());
        }
    }

    for row in &all_rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    let separator = create_separator(&col_widths);
    println!("{}", separator);

    if let Some(h) = headers {
        print_row(&h, &col_widths);
        println!("{}", separator);
    }

    for row in all_rows {
        print_row(&row, &col_widths);
    }

    println!("{}", separator);

    Ok(())
}

fn create_separator(col_widths: &[usize]) -> String {
    let parts: Vec<String> = col_widths
        .iter()
        .map(|&width| "-".repeat(width + 2))
        .collect();
    format!("+{}+", parts.join("+"))
}

fn print_row(row: &[String], col_widths: &[usize]) {
    let parts: Vec<String> = row
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            if i < col_widths.len() {
                format!(" {:<width$} ", cell, width = col_widths[i])
            } else {
                format!(" {} ", cell)
            }
        })
        .collect();
    println!("|{}|", parts.join("|"));
}
