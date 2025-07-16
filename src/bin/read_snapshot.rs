use arrow::array::{Float64Array, StringArray, UInt64Array};
use arrow::record_batch::RecordBatch;
use clap::Parser;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use serde_json::{Value, json};
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the parquet file
    #[arg(short, long)]
    file: PathBuf,

    /// Number of rows to display (0 for all)
    #[arg(short, long, default_value = "10")]
    rows: usize,
}

fn batch_to_json(batch: &RecordBatch) -> Vec<Value> {
    let num_rows = batch.num_rows();
    let mut result = Vec::with_capacity(num_rows);

    // Get column names
    let schema = batch.schema();
    let column_names: Vec<_> = schema.fields().iter().map(|f| f.name()).collect();

    // Convert each row to JSON
    for row_idx in 0..num_rows {
        let mut row = serde_json::Map::new();
        for (col_idx, col_name) in column_names.iter().enumerate() {
            let array = batch.column(col_idx);
            let value = match array.data_type() {
                arrow::datatypes::DataType::Float64 => {
                    let array = array.as_any().downcast_ref::<Float64Array>().unwrap();
                    json!(array.value(row_idx))
                }
                arrow::datatypes::DataType::UInt64 => {
                    let array = array.as_any().downcast_ref::<UInt64Array>().unwrap();
                    json!(array.value(row_idx))
                }
                arrow::datatypes::DataType::Utf8 => {
                    let array = array.as_any().downcast_ref::<StringArray>().unwrap();
                    json!(array.value(row_idx))
                }
                _ => json!(null),
            };
            row.insert(col_name.to_string(), value);
        }
        result.push(json!(row));
    }

    result
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Open the parquet file
    let file = File::open(&args.file)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;

    // Get the schema from the builder before building the reader
    let schema = builder.schema().clone();
    let mut reader = builder.build()?;

    println!("Reading from file: {}", args.file.display());
    println!("Schema:");
    println!("{}", schema);
    println!();

    let mut total_rows = 0;
    while let Some(batch) = reader.next() {
        let batch = batch?;
        let json_rows = batch_to_json(&batch);

        let num_rows = if args.rows == 0 {
            json_rows.len()
        } else {
            args.rows.min(json_rows.len())
        };

        for row in json_rows.iter().take(num_rows) {
            println!("{}", serde_json::to_string_pretty(row)?);
            total_rows += 1;
        }

        if args.rows > 0 && total_rows >= args.rows {
            break;
        }
    }

    println!("\nTotal rows displayed: {}", total_rows);
    Ok(())
}
