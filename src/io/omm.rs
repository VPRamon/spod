//! # OMM reader/writer bridging `siderust-tle`
//!
//! Delegates KVN and XML parsing/writing to [`crate::tle::omm::kvn`] and
//! [`crate::tle::omm::xml`], exposing them under the unified
//! [`PodIoError`] error type.
//!
//! ## References
//!
//! - CCSDS 502.0-B-2: Orbit Mean-Elements Message (OMM).

use super::PodIoError;
use crate::tle::omm::{kvn, xml};
use std::io::{Read, Write};

/// Re-export of the `siderust-tle` OMM record.
///
/// See [`crate::tle::omm::Omm`] for field documentation.
///
/// # Examples
///
/// ```
/// use spod::io::omm::OmmMessage;
/// use spod::tle::{parse_3le, omm::Omm};
/// let l1 = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927";
/// let l2 = "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";
/// let tle = parse_3le("ISS (ZARYA)", l1, l2).unwrap();
/// let _omm: OmmMessage = Omm::from_tle(&tle);
/// ```
pub use crate::tle::omm::Omm as OmmMessage;

/// Read an OMM KVN message from a byte source.
///
/// # Errors
///
/// Returns [`PodIoError::Io`] on read failure or [`PodIoError::Format`]
/// on parse error.
///
/// # Examples
///
/// ```
/// use spod::io::omm::read_omm_kvn;
///
/// let data = b"CCSDS_OMM_VERS = 2.0\nOBJECT_NAME = TEST\nOBJECT_ID = 1998-067A\n\
///              MEAN_ELEMENT_THEORY = SGP4\nEPOCH = 2008-09-20T12:25:40.104192\n\
///              MEAN_MOTION = 15.72125391\nECCENTRICITY = 0.0006703\n\
///              INCLINATION = 51.6416\nRA_OF_ASC_NODE = 247.4627\n\
///              ARG_OF_PERICENTER = 130.5360\nMEAN_ANOMALY = 325.0288\n\
///              EPHEMERIS_TYPE = 0\nCLASSIFICATION_TYPE = U\nNORAD_CAT_ID = 25544\n\
///              ELEMENT_SET_NO = 292\nREV_AT_EPOCH = 56353\n\
///              BSTAR = -1.1606e-5\nMEAN_MOTION_DOT = -2.182e-5\nMEAN_MOTION_DDOT = 0.0\n";
/// let omm = read_omm_kvn(&data[..]).unwrap();
/// assert_eq!(omm.norad_id.0, 25544);
/// ```
pub fn read_omm_kvn<R: Read>(mut reader: R) -> Result<OmmMessage, PodIoError> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf).map_err(PodIoError::Io)?;
    kvn::read(&buf).map_err(|e| PodIoError::Format(e.to_string()))
}

/// Write an OMM KVN message to a writer.
///
/// # Errors
///
/// Returns [`PodIoError::Format`] if serialisation fails or
/// [`PodIoError::Io`] on write failure.
///
/// # Examples
///
/// ```
/// use spod::io::omm::{write_omm_kvn, OmmMessage};
/// use spod::tle::{parse_3le, omm::Omm};
///
/// let l1 = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927";
/// let l2 = "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";
/// let tle = parse_3le("ISS (ZARYA)", l1, l2).unwrap();
/// let omm = Omm::from_tle(&tle);
/// let mut buf = Vec::new();
/// write_omm_kvn(&mut buf, &omm).unwrap();
/// assert!(!buf.is_empty());
/// ```
pub fn write_omm_kvn<W: Write>(w: &mut W, omm: &OmmMessage) -> Result<(), PodIoError> {
    let text = kvn::write(omm).map_err(|e| PodIoError::Format(e.to_string()))?;
    w.write_all(text.as_bytes()).map_err(PodIoError::Io)
}

/// Read an OMM XML message from a byte source.
///
/// # Errors
///
/// Returns [`PodIoError::Io`] on read failure or [`PodIoError::Format`]
/// on parse error.
///
/// # Examples
///
/// ```
/// use spod::io::omm::{read_omm_xml, write_omm_xml, OmmMessage};
/// use spod::tle::{parse_3le, omm::Omm};
///
/// let l1 = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927";
/// let l2 = "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";
/// let tle = parse_3le("ISS (ZARYA)", l1, l2).unwrap();
/// let omm = Omm::from_tle(&tle);
/// let mut buf = Vec::new();
/// write_omm_xml(&mut buf, &omm).unwrap();
/// let omm2 = read_omm_xml(&buf[..]).unwrap();
/// assert_eq!(omm2.norad_id, omm.norad_id);
/// ```
pub fn read_omm_xml<R: Read>(mut reader: R) -> Result<OmmMessage, PodIoError> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf).map_err(PodIoError::Io)?;
    xml::read(&buf).map_err(|e| PodIoError::Format(e.to_string()))
}

/// Write an OMM XML message to a writer.
///
/// # Errors
///
/// Returns [`PodIoError::Format`] if serialisation fails or
/// [`PodIoError::Io`] on write failure.
///
/// # Examples
///
/// ```
/// use spod::io::omm::{write_omm_xml, OmmMessage};
/// use spod::tle::{parse_3le, omm::Omm};
///
/// let l1 = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927";
/// let l2 = "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";
/// let tle = parse_3le("ISS (ZARYA)", l1, l2).unwrap();
/// let omm = Omm::from_tle(&tle);
/// let mut buf = Vec::new();
/// write_omm_xml(&mut buf, &omm).unwrap();
/// assert!(!buf.is_empty());
/// ```
pub fn write_omm_xml<W: Write>(w: &mut W, omm: &OmmMessage) -> Result<(), PodIoError> {
    let text = xml::write(omm).map_err(|e| PodIoError::Format(e.to_string()))?;
    w.write_all(text.as_bytes()).map_err(PodIoError::Io)
}
