//! Parquet residual support provided by Siderust.
//!
//! The writer is re-exported unchanged so this service does not maintain a
//! second Parquet implementation.

pub use siderust::pod::product::residuals_csv::ResidualRecord;
pub use siderust::pod::product::residuals_parquet::ResidualParquetWriter;

#[cfg(test)]
mod tests {
    use super::ResidualParquetWriter;
    use crate::products::ResidualRecord;
    use std::io::Cursor;

    #[test]
    fn public_residual_record_is_accepted_by_parquet_writer() {
        let mut writer = ResidualParquetWriter::new(Cursor::new(Vec::<u8>::new()));
        writer.push(ResidualRecord {
            epoch_jd_tt: 2_451_545.0,
            obs_type: "C1C".into(),
            satellite: "G01".into(),
            residual_m: -0.15,
            sigma_m: 0.30,
            rejected: false,
        });

        let output = writer.finish().expect("valid residual record");
        assert!(!output.into_inner().is_empty());
    }
}
