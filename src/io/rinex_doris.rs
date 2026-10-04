//! RINEX-DORIS support provided by Siderust.
//!
//! The parser and record types are re-exported unchanged so this service
//! does not maintain a second DORIS implementation.

pub use siderust::formats::rinex::doris::{
    read_rinex_doris, DorisHeader, DorisObservation, RinexDoris, RinexDorisRecord,
};

#[cfg(test)]
mod tests {
    #[cfg(feature = "doris")]
    use super::read_rinex_doris;

    #[cfg(feature = "doris")]
    #[test]
    fn parses_minimal_doris_input() {
        let input = format!(
            "{:<60}RINEX VERSION / TYPE\n{:<60}END OF HEADER\n\
             > 2024 01 02 03 04 05.000\nD01 12345.678 678.9\n",
            "     3.00           OBSERVATION DATA    D", ""
        );
        let parsed = read_rinex_doris(input.as_bytes()).expect("minimal DORIS input is valid");

        assert_eq!(parsed.header.version, "3.00");
        assert_eq!(parsed.observations.len(), 1);
        assert_eq!(parsed.observations[0].satellite_id, "D01");
    }
}
