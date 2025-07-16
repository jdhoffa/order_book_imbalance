use crate::models::OrderBookSnapshot;
use arrow::array::{Float64Array, StringArray, UInt64Array};
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::error::Error;
use std::fs::File;
use std::sync::Arc;

pub struct ParquetWriter;

impl ParquetWriter {
    pub async fn save_snapshot(
        snapshot: &OrderBookSnapshot,
        symbol: &str,
    ) -> Result<String, Box<dyn Error>> {
        let (arrays, schema) = Self::prepare_arrays(snapshot, symbol)?;
        let filename = Self::create_filename(symbol);
        Self::write_parquet(&filename, schema, arrays)?;
        Ok(filename)
    }

    fn prepare_arrays(
        snapshot: &OrderBookSnapshot,
        symbol: &str,
    ) -> Result<
        (
            Vec<Arc<dyn arrow::array::Array>>,
            Arc<arrow::datatypes::Schema>,
        ),
        Box<dyn Error>,
    > {
        let mut bid_prices = Vec::new();
        let mut bid_quantities = Vec::new();
        let mut ask_prices = Vec::new();
        let mut ask_quantities = Vec::new();
        let mut update_ids = Vec::new();
        let mut symbols = Vec::new();

        for bid in &snapshot.bids {
            bid_prices.push(bid[0].parse::<f64>()?);
            bid_quantities.push(bid[1].parse::<f64>()?);
            update_ids.push(snapshot.last_update_id);
            symbols.push(symbol);
        }

        for ask in &snapshot.asks {
            ask_prices.push(ask[0].parse::<f64>()?);
            ask_quantities.push(ask[1].parse::<f64>()?);
        }

        let arrays = vec![
            Arc::new(StringArray::from(symbols)) as Arc<dyn arrow::array::Array>,
            Arc::new(UInt64Array::from(update_ids)),
            Arc::new(Float64Array::from(bid_prices)),
            Arc::new(Float64Array::from(bid_quantities)),
            Arc::new(Float64Array::from(ask_prices)),
            Arc::new(Float64Array::from(ask_quantities)),
        ];

        let schema = Arc::new(arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("symbol", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("update_id", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("bid_price", arrow::datatypes::DataType::Float64, false),
            arrow::datatypes::Field::new(
                "bid_quantity",
                arrow::datatypes::DataType::Float64,
                false,
            ),
            arrow::datatypes::Field::new("ask_price", arrow::datatypes::DataType::Float64, false),
            arrow::datatypes::Field::new(
                "ask_quantity",
                arrow::datatypes::DataType::Float64,
                false,
            ),
        ]));

        Ok((arrays, schema))
    }

    fn create_filename(symbol: &str) -> String {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        format!("orderbook_snapshot_{}_{}.parquet", symbol, timestamp)
    }

    fn write_parquet(
        filename: &str,
        schema: Arc<arrow::datatypes::Schema>,
        arrays: Vec<Arc<dyn arrow::array::Array>>,
    ) -> Result<(), Box<dyn Error>> {
        let batch = RecordBatch::try_new(schema.clone(), arrays)?;
        let file = File::create(filename)?;
        let props = WriterProperties::builder().build();
        let mut writer = ArrowWriter::try_new(file, schema, Some(props))?;
        writer.write(&batch)?;
        writer.close()?;
        Ok(())
    }
}
