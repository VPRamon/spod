//! Parquet residual support provided by Siderust.
//!
//! The writer is re-exported unchanged so this service does not maintain a
//! second Parquet implementation.

pub use siderust::pod::product::residuals_csv::ResidualRecord;
pub use siderust::pod::product::residuals_parquet::ResidualParquetWriter;
