//! Location/GPS record types for Palm OS
//!
//! This module provides location and GPS-related record parsing and serialization.

use std::fmt;
use crate::error::{PilotError, Result};
use crate::types::PalmDateTime;

/// Location record
#[derive(Debug, Clone)]
pub struct LocationRecord {
    /// Record ID
    pub id: u32,
    /// Category
    pub category: u8,
    /// Attributes
    pub attributes: LocationAttributes,
    /// Place name
    pub name: String,
    /// Address
    pub address: String,
    /// City
    pub city: String,
    /// State
    pub state: String,
    /// ZIP code
    pub zip: String,
    /// Country
    pub country: String,
    /// Phone
    pub phone: String,
    /// Latitude
    pub latitude: GpsCoordinate,
    /// Longitude
    pub longitude: GpsCoordinate,
    /// Altitude
    pub altitude: Option<i32>,
    /// Position date
    pub position_date: PalmDateTime,
    /// Note
    pub note: String,
}

/// Location attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocationAttributes(u8);

impl LocationAttributes {
    pub const SECRET: u8 = 0x02;
    pub const BUSY: u8 = 0x20;
    pub const ARCHIVE: u8 = 0x10;

    pub fn is_secret(&self) -> bool { (self.0 & Self::SECRET) != 0 }
    pub fn is_busy(&self) -> bool { (self.0 & Self::BUSY) != 0 }
    pub fn is_archived(&self) -> bool { (self.0 & Self::ARCHIVE) != 0 }
}

/// GPS coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpsCoordinate {
    /// Degrees
    pub degrees: i32,
    /// Minutes
    pub minutes: u32,
    /// Seconds
    pub seconds: u32,
    /// Direction (N/S/E/W)
    pub direction: GpsDirection,
}

impl GpsCoordinate {
    /// Create new coordinate
    pub fn new(degrees: i32, minutes: u32, seconds: u32, direction: GpsDirection) -> Self {
        Self { degrees, minutes, seconds, direction }
    }

    /// Convert to decimal degrees
    pub fn to_decimal(&self) -> f64 {
        let mut dec = self.degrees as f64;
        dec += (self.minutes as f64) / 60.0;
        dec += (self.seconds as f64) / 3600.0;
        
        match self.direction {
            GpsDirection::South | GpsDirection::West => -dec,
            _ => dec,
        }
    }

    /// Create from decimal degrees
    pub fn from_decimal(decimal: f64) -> Self {
        let abs = decimal.abs();
        let deg = abs.trunc() as i32;
        let min_f = (abs - deg as f64) * 60.0;
        let min = min_f.trunc() as u32;
        let sec = ((min_f - min as f64) * 60.0 * 1000.0).round() as u32;
        
        let dir = if decimal < 0.0 { GpsDirection::South } else { GpsDirection::North };
        
        Self { degrees: deg, minutes: min, seconds: sec, direction: dir }
    }

    /// Format as string (DD°MM'SS"N)
    pub fn format(&self) -> String {
        format!("{}°{}'{}''{}", self.degrees, self.minutes, self.seconds, self.direction)
    }
}

/// GPS direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpsDirection {
    North,
    South,
    East,
    West,
}

impl fmt::Display for GpsDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpsDirection::North => write!(f, "N"),
            GpsDirection::South => write!(f, "S"),
            GpsDirection::East => write!(f, "E"),
            GpsDirection::West => write!(f, "W"),
        }
    }
}

impl GpsDirection {
    pub fn from_char(c: char) -> Self {
        match c.to_ascii_uppercase() {
            'S' => GpsDirection::South,
            'E' => GpsDirection::East,
            'W' => GpsDirection::West,
            _ => GpsDirection::North,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            GpsDirection::North => 'N',
            GpsDirection::South => 'S',
            GpsDirection::East => 'E',
            GpsDirection::West => 'W',
        }
    }
}

/// Position data
#[derive(Debug, Clone)]
pub struct Position {
    /// Latitude
    pub lat: GpsCoordinate,
    /// Longitude
    pub lon: GpsCoordinate,
    /// Altitude (meters)
    pub altitude: Option<f64>,
    /// Accuracy (meters)
    pub accuracy: Option<f64>,
    /// Speed (m/s)
    pub speed: Option<f64>,
    /// Heading (degrees from north)
    pub heading: Option<f64>,
    /// Timestamp
    pub timestamp: PalmDateTime,
}

impl Position {
    /// Calculate distance to another position (Haversine formula)
    pub fn distance_to(&self, other: &Position) -> f64 {
        let lat1 = self.lat.to_decimal().to_radians();
        let lat2 = other.lat.to_decimal().to_radians();
        let dlat = (other.lat.to_decimal() - self.lat.to_decimal()).to_radians();
        let dlon = (other.lon.to_decimal() - self.lon.to_decimal()).to_radians();

        let a = (dlat / 2.0).sin().powi(2) 
            + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();
        
        // Earth radius in meters
        6371000.0 * c
    }
}

/// Location application info
#[derive(Debug, Clone)]
pub struct LocationAppInfo {
    /// Categories
    pub categories: Vec<String>,
    /// Default category
    pub default_category: u8,
    /// Distance units
    pub use_metric: bool,
    /// Version
    pub version: u16,
}

impl Default for LocationAppInfo {
    fn default() -> Self {
        Self {
            categories: vec!["Unfiled".to_string()],
            default_category: 0,
            use_metric: false,
            version: 1,
        }
    }
}

impl Default for LocationRecord {
    fn default() -> Self {
        Self {
            id: 0,
            category: 0,
            attributes: LocationAttributes(0),
            name: String::new(),
            address: String::new(),
            city: String::new(),
            state: String::new(),
            zip: String::new(),
            country: String::new(),
            phone: String::new(),
            latitude: GpsCoordinate::new(0, 0, 0, GpsDirection::North),
            longitude: GpsCoordinate::new(0, 0, 0, GpsDirection::West),
            altitude: None,
            position_date: PalmDateTime::now(),
            note: String::new(),
        }
    }
}

impl LocationRecord {
    /// Parse from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 30 {
            return Err(PilotError::InvalidData("Location record too short".into()));
        }

        let mut record = LocationRecord::default();
        let mut offset = 0;

        // Parse latitude (7 bytes)
        let lat_deg = i32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
        offset += 4;
        let lat_min = u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
        offset += 4;
        let lat_sec = u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
        offset += 4;
        let lat_dir = match data[offset] {
            b'S' => GpsDirection::South,
            _ => GpsDirection::North,
        };
        offset += 1;
        
        record.latitude = GpsCoordinate::new(lat_deg, lat_min, lat_sec, lat_dir);

        // Parse longitude (7 bytes)
        let lon_deg = i32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
        offset += 4;
        let lon_min = u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
        offset += 4;
        let lon_sec = u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
        offset += 4;
        let lon_dir = match data[offset] {
            b'W' => GpsDirection::West,
            b'E' => GpsDirection::East,
            _ => GpsDirection::West,
        };
        offset += 1;
        
        record.longitude = GpsCoordinate::new(lon_deg, lon_min, lon_sec, lon_dir);

        // Parse altitude if present
        if offset + 4 <= data.len() {
            let alt = i32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
            if alt != 0 {
                record.altitude = Some(alt);
            }
            offset += 4;
        }

        // Parse strings
        let (name, new_offset) = Self::parse_string(data, offset)?;
        record.name = name;
        offset = new_offset;

        let (address, new_offset) = Self::parse_string(data, offset)?;
        record.address = address;
        offset = new_offset;

        let (city, new_offset) = Self::parse_string(data, offset)?;
        record.city = city;
        offset = new_offset;

        let (state, new_offset) = Self::parse_string(data, offset)?;
        record.state = state;
        offset = new_offset;

        let (zip, new_offset) = Self::parse_string(data, offset)?;
        record.zip = zip;
        offset = new_offset;

        let (country, new_offset) = Self::parse_string(data, offset)?;
        record.country = country;
        offset = new_offset;

        let (phone, new_offset) = Self::parse_string(data, offset)?;
        record.phone = phone;
        offset = new_offset;

        let (note, _) = Self::parse_string(data, offset)?;
        record.note = note;

        Ok(record)
    }

    /// Pack to bytes
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Latitude
        data.extend_from_slice(&self.latitude.degrees.to_le_bytes());
        data.extend_from_slice(&self.latitude.minutes.to_le_bytes());
        data.extend_from_slice(&self.latitude.seconds.to_le_bytes());
        data.push(self.latitude.direction.to_char() as u8);

        // Longitude
        data.extend_from_slice(&self.longitude.degrees.to_le_bytes());
        data.extend_from_slice(&self.longitude.minutes.to_le_bytes());
        data.extend_from_slice(&self.longitude.seconds.to_le_bytes());
        data.push(self.longitude.direction.to_char() as u8);

        // Altitude
        let alt = self.altitude.unwrap_or(0i32);
        data.extend_from_slice(&alt.to_le_bytes());

        // Strings
        data.extend_from_slice(&Self::pack_string(&self.name));
        data.extend_from_slice(&Self::pack_string(&self.address));
        data.extend_from_slice(&Self::pack_string(&self.city));
        data.extend_from_slice(&Self::pack_string(&self.state));
        data.extend_from_slice(&Self::pack_string(&self.zip));
        data.extend_from_slice(&Self::pack_string(&self.country));
        data.extend_from_slice(&Self::pack_string(&self.phone));
        data.extend_from_slice(&Self::pack_string(&self.note));

        data
    }

    fn parse_string(data: &[u8], offset: usize) -> Result<(String, usize)> {
        let mut end = offset;
        while end < data.len() && data[end] != 0 {
            end += 1;
        }
        let s = String::from_utf8_lossy(&data[offset..end]).to_string();
        Ok((s, end + 1))
    }

    fn pack_string(s: &str) -> Vec<u8> {
        let mut bytes = s.as_bytes().to_vec();
        bytes.push(0);
        bytes
    }

    /// Get full address
    pub fn full_address(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if !self.address.is_empty() { parts.push(&self.address); }
        if !self.city.is_empty() { parts.push(&self.city); }
        if !self.state.is_empty() { parts.push(&self.state); }
        if !self.zip.is_empty() { parts.push(&self.zip); }
        parts.join(", ")
    }
}

/// Location constants
pub mod constants {
    use crate::types::FourCharCode;

    /// Location database type
    pub const LOCATION_TYPE: FourCharCode = FourCharCode { 0: 0x4C6F6361 }; // "Loca"
    
    /// Location database creator
    pub const LOCATION_CREATOR: FourCharCode = FourCharCode { 0: 0x4C6F6361 }; // "Loca"

    /// Earth radius in meters
    pub const EARTH_RADIUS_M: f64 = 6371000.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gps_direction() {
        assert_eq!(GpsDirection::from_char('N'), GpsDirection::North);
        assert_eq!(GpsDirection::from_char('s'), GpsDirection::South);
        assert_eq!(GpsDirection::to_char(&GpsDirection::East), 'E');
    }

    #[test]
    fn test_gps_coordinate_decimal() {
        // 40°42'46"N = 40.7128
        let coord = GpsCoordinate::new(40, 42, 46, GpsDirection::North);
        let decimal = coord.to_decimal();
        assert!((decimal - 40.7128).abs() < 0.01);
        
        // Round trip
        let coord2 = GpsCoordinate::from_decimal(40.7128);
        assert!((coord2.degrees - 40).abs() < 1);
    }

    #[test]
    fn test_position_distance() {
        let pos1 = Position {
            lat: GpsCoordinate::new(40, 42, 46, GpsDirection::North),
            lon: GpsCoordinate::new(74, 0, 0, GpsDirection::West),
            altitude: None,
            accuracy: None,
            speed: None,
            heading: None,
            timestamp: PalmDateTime::now(),
        };
        
        let pos2 = Position {
            lat: GpsCoordinate::new(40, 43, 0, GpsDirection::North),
            lon: GpsCoordinate::new(74, 0, 0, GpsDirection::West),
            altitude: None,
            accuracy: None,
            speed: None,
            heading: None,
            timestamp: PalmDateTime::now(),
        };
        
        // About 270 meters apart - test distance calculation works
        let dist = pos1.distance_to(&pos2);
        assert!(dist > 100.0 && dist < 1000.0, "Distance should be reasonable: {}", dist);
    }

    #[test]
    fn test_location_record_pack_parse() {
        let mut record = LocationRecord::default();
        record.name = "Central Park".to_string();
        record.city = "New York".to_string();
        record.state = "NY".to_string();
        record.latitude = GpsCoordinate::new(40, 46, 56, GpsDirection::North);
        record.longitude = GpsCoordinate::new(73, 57, 55, GpsDirection::West);

        let packed = record.pack();
        let parsed = LocationRecord::parse(&packed).unwrap();
        
        assert_eq!(parsed.name, "Central Park");
        assert_eq!(parsed.city, "New York");
    }
}
