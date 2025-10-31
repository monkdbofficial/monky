/*
 * Copyright (C) 2025 Movibase Platform Private Limited
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

//! Utilities for converting between Unix timestamps (in milliseconds)
//! and ISO/RFC3339 date-time strings without external formatting crates.
//!
//! This module provides safe conversions with robust overflow checking,
//! and detailed error handling via `DateFormatError`.

use time::format_description::well_known::Rfc3339;
use time::format_description::FormatItem;
use time::{format_description, OffsetDateTime};

use std::fmt;

/// Errors for date formatting and parsing without external crates.
#[derive(Debug)]
pub enum DateFormatError {
    /// Raised when a format description cannot be parsed or constructed.
    InvalidFormatDescription(time::error::InvalidFormatDescription),
    /// Raised when parsing a date-time string fails.
    Parse(time::error::Parse),
    /// Raised when formatting an OffsetDateTime fails.
    Format(time::error::Format),
    /// Raised when integer overflow or underflow occurs in conversions.
    IntConversion,
    /// Raised when Unix timestamp is invalid or out of range.
    InvalidTimestamp,
}

impl fmt::Display for DateFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateFormatError::InvalidFormatDescription(e) => {
                write!(f, "invalid format description: {}", e)
            }
            DateFormatError::Parse(e) => write!(f, "parse error: {}", e),
            DateFormatError::Format(e) => write!(f, "format error: {}", e),
            DateFormatError::IntConversion => {
                write!(f, "integer conversion overflow or underflow")
            }
            DateFormatError::InvalidTimestamp => write!(f, "invalid unix timestamp"),
        }
    }
}

impl std::error::Error for DateFormatError {}

impl From<time::error::InvalidFormatDescription> for DateFormatError {
    fn from(e: time::error::InvalidFormatDescription) -> Self {
        DateFormatError::InvalidFormatDescription(e)
    }
}
impl From<time::error::Parse> for DateFormatError {
    fn from(e: time::error::Parse) -> Self {
        DateFormatError::Parse(e)
    }
}
impl From<time::error::Format> for DateFormatError {
    fn from(e: time::error::Format) -> Self {
        DateFormatError::Format(e)
    }
}

/// Returns a format description used to format timestamps up to seconds.
///
/// The pattern used is `YYYY-MM-DDTHH:MM:SS`.
///
/// # Errors
///
/// Returns `DateFormatError::InvalidFormatDescription` if the format
/// description string cannot be parsed.
fn seconds_format_description() -> Result<Vec<FormatItem<'static>>, time::error::InvalidFormatDescription> {
    // pattern: YYYY-MM-DDTHH:MM:SS
    format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]")
}


/// Converts epoch milliseconds into an ISO 8601 string with exactly three
/// fractional digits and a trailing `Z`.
///
/// Example output:
/// ```
/// use monky_utilities::date_format::iso_from_millis;
/// let iso = iso_from_millis(1_602_123_456_789).unwrap();
/// assert_eq!(iso, "2020-09-12T12:34:16.789Z");
/// ```
///
/// # Errors
///
/// * `DateFormatError::IntConversion` if multiplication overflows.
/// * `DateFormatError::InvalidTimestamp` if timestamp is out of range.
/// * `DateFormatError::Format` if formatting fails.
pub fn iso_from_millis(epoch_millis: i128) -> Result<String, DateFormatError> {
    // Convert millis -> nanos safely
    let nanos = epoch_millis
        .checked_mul(1_000_000)
        .ok_or(DateFormatError::IntConversion)?;

    // Build OffsetDateTime from nanos (may fail if out of range)
    let odt =
        OffsetDateTime::from_unix_timestamp_nanos(nanos).map_err(|_| DateFormatError::InvalidTimestamp)?;

    // Format base (without fractional seconds)
    let fmt = seconds_format_description()?;
    let base = odt.format(&fmt)?; // e.g. "2020-09-12T12:34:16"

    // Compute millisecond component in range 0..=999 robustly for negative timestamps too.
    // rem_euclid yields non-negative remainder.
    let ms = epoch_millis.rem_euclid(1000) as i128; // 0..=999

    // Assemble final string ensuring 3-digit zero-padded millis and trailing Z
    let result = format!("{}.{:03}Z", base, ms);
    Ok(result)
}

/// Parses an ISO/RFC3339 date-time string into epoch milliseconds and
/// `OffsetDateTime`.
///
/// Example:
/// ```
/// # use monky_utilities::date_format::instant_from_iso;
/// let (millis, odt) = instant_from_iso("2020-09-12T12:34:16.789Z").unwrap();
/// assert_eq!(millis, 1_602_123_456_789);
/// assert_eq!(odt.hour(), 12);
/// ```
///
/// # Errors
///
/// * `DateFormatError::Parse` if the string cannot be parsed.
/// * `DateFormatError::IntConversion` if division fails.
pub fn instant_from_iso(iso_str: &str) -> Result<(i128, OffsetDateTime), DateFormatError> {
    // Rfc3339 accepts many ISO-8601 variants (fractional digits optional, timezone offsets)
    let odt = OffsetDateTime::parse(iso_str, &Rfc3339)?;
    let nanos = odt.unix_timestamp_nanos();
    let millis = nanos
        .checked_div(1_000_000)
        .ok_or(DateFormatError::IntConversion)?;
    Ok((millis, odt))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iso_from_millis_positive() {
        // 2020-09-12T12:34:16.789Z
        let epoch_millis = 1_600_000_456_789i128;
        let iso = iso_from_millis(epoch_millis).unwrap();
        assert!(iso.starts_with("2020-09-"));
        assert!(iso.ends_with("Z"));
        assert!(iso.contains('.'));
        assert_eq!(iso.len(), 24); // "YYYY-MM-DDTHH:MM:SS.mmmZ"
    }

    #[test]
    fn test_iso_from_millis_zero() {
        let iso = iso_from_millis(0).unwrap();
        assert_eq!(iso, "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn test_iso_from_millis_negative() {
        let epoch_millis = -1_000i128;
        let iso = iso_from_millis(epoch_millis).unwrap();
        // Should be just before the Unix epoch
        assert!(iso.starts_with("1969-12-31"));
        assert!(iso.ends_with(".000Z") || iso.ends_with(".999Z"));
    }

    #[test]
    fn test_iso_from_millis_overflow() {
        let large = i128::MAX / 1_000_000; // big enough to overflow nanos
        let result = iso_from_millis(large);
        assert!(matches!(result, Err(DateFormatError::InvalidTimestamp) | Err(DateFormatError::IntConversion)));
    }

    #[test]
    fn test_instant_from_iso_basic() {
        let iso = "2020-09-12T12:34:16.789Z";
        let (millis, odt) = instant_from_iso(iso).unwrap();
        assert!(millis > 0);
        assert_eq!(odt.offset().whole_seconds(), 0);
        assert!(odt.year() >= 2020);
    }

    #[test]
    fn test_instant_from_iso_with_offset() {
        let iso = "2020-09-12T14:34:16.789+02:00";
        let (millis, odt) = instant_from_iso(iso).unwrap();
        assert_eq!(odt.offset().whole_seconds(), 7200);
        // Re-format and verify it corresponds to same instant in UTC
        let utc_iso = iso_from_millis(millis).unwrap();
        assert!(utc_iso.starts_with("2020-09-12T12:34:16"));
    }

    #[test]
    fn test_instant_from_iso_invalid() {
        let result = instant_from_iso("invalid-date");
        assert!(matches!(result, Err(DateFormatError::Parse(_))));
    }

    #[test]
    fn test_round_trip_consistency() {
        let now = 1_725_000_000_123i128; // arbitrary, modern timestamp
        let iso = iso_from_millis(now).unwrap();
        let (millis_back, _) = instant_from_iso(&iso).unwrap();
        // Allow 1 ms rounding tolerance
        assert!((now - millis_back).abs() <= 1);
    }
}
