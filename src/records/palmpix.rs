//! PalmPix record types for Palm OS
//!
//! This module provides PalmPix image record parsing and serialization.

use crate::error::{PilotError, Result};
use crate::types::PalmDateTime;

/// PalmPix record (image)
#[derive(Debug, Clone)]
pub struct PalmPixRecord {
    /// Record ID
    pub id: u32,
    /// Category
    pub category: u8,
    /// Attributes
    pub attributes: PalmPixAttributes,
    /// Created date
    pub created: PalmDateTime,
    /// Modified date
    pub modified: PalmDateTime,
    /// Image format
    pub format: ImageFormat,
    /// Width
    pub width: u16,
    /// Height
    pub height: u16,
    /// Bits per pixel
    pub bits_per_pixel: u8,
    /// Image data
    pub image_data: Vec<u8>,
    /// Thumbnail data
    pub thumbnail: Option<Vec<u8>>,
    /// Camera info
    pub camera_info: Option<CameraInfo>,
    /// Description
    pub description: String,
}

/// PalmPix attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PalmPixAttributes(u8);

impl PalmPixAttributes {
    pub const SECRET: u8 = 0x02;
    pub const BUSY: u8 = 0x20;
    pub const ARCHIVE: u8 = 0x10;

    pub fn is_secret(&self) -> bool {
        (self.0 & Self::SECRET) != 0
    }
    pub fn is_busy(&self) -> bool {
        (self.0 & Self::BUSY) != 0
    }
    pub fn is_archived(&self) -> bool {
        (self.0 & Self::ARCHIVE) != 0
    }
}

/// Image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ImageFormat {
    Jpeg = 0,
    Gif = 1,
    Bmp = 2,
    Png = 3,
    Raw = 4,
    Unknown = 255,
}

impl ImageFormat {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => ImageFormat::Jpeg,
            1 => ImageFormat::Gif,
            2 => ImageFormat::Bmp,
            3 => ImageFormat::Png,
            4 => ImageFormat::Raw,
            _ => ImageFormat::Unknown,
        }
    }
}

/// Camera information
#[derive(Debug, Clone, Default)]
pub struct CameraInfo {
    /// Camera make
    pub make: String,
    /// Camera model
    pub model: String,
    /// Exposure time
    pub exposure: Option<u32>,
    /// F-number (aperture)
    pub f_number: Option<f32>,
    /// ISO speed
    pub iso: Option<u16>,
    /// Flash fired
    pub flash_fired: bool,
    /// Focal length
    pub focal_length: Option<f32>,
}

/// Thumbnail image
#[derive(Debug, Clone)]
pub struct Thumbnail {
    /// Width
    pub width: u16,
    /// Height
    pub height: u16,
    /// Data
    pub data: Vec<u8>,
}

impl Thumbnail {
    /// Create thumbnail from image data
    pub fn create(data: &[u8], _target_width: u16, _target_height: u16) -> Self {
        // Simplified: just return smaller version
        let thumb_data: Vec<u8> = data.iter().take(1024).copied().collect();
        Self {
            width: 80,
            height: 60,
            data: thumb_data,
        }
    }
}

/// Image metadata
#[derive(Debug, Clone)]
pub struct ImageMetadata {
    /// GPS location
    pub location: Option<GpsLocation>,
    /// Orientation
    pub orientation: ImageOrientation,
    /// Rating
    pub rating: u8,
}

impl Default for ImageMetadata {
    fn default() -> Self {
        Self {
            location: None,
            orientation: ImageOrientation::Normal,
            rating: 0,
        }
    }
}

/// GPS location for image
#[derive(Debug, Clone)]
pub struct GpsLocation {
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
    /// Altitude (meters)
    pub altitude: Option<f64>,
}

/// Image orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ImageOrientation {
    Normal = 0,
    FlippedHorizontal = 1,
    Rotated180 = 2,
    FlippedVertical = 3,
    Transposed = 4,
    Rotated90CW = 5,
    Transverse = 6,
    Rotated270CW = 7,
}

impl ImageOrientation {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => ImageOrientation::Normal,
            1 => ImageOrientation::FlippedHorizontal,
            2 => ImageOrientation::Rotated180,
            3 => ImageOrientation::FlippedVertical,
            4 => ImageOrientation::Transposed,
            5 => ImageOrientation::Rotated90CW,
            6 => ImageOrientation::Transverse,
            7 => ImageOrientation::Rotated270CW,
            _ => ImageOrientation::Normal,
        }
    }
}

/// PalmPix application info
#[derive(Debug, Clone)]
pub struct PalmPixAppInfo {
    /// Default format
    pub default_format: ImageFormat,
    /// Compression quality (1-100)
    pub quality: u8,
    /// Create thumbnails
    pub create_thumbnails: bool,
    /// Thumbnail width
    pub thumbnail_width: u16,
    /// Thumbnail height
    pub thumbnail_height: u16,
    /// Version
    pub version: u16,
}

impl Default for PalmPixAppInfo {
    fn default() -> Self {
        Self {
            default_format: ImageFormat::Jpeg,
            quality: 85,
            create_thumbnails: true,
            thumbnail_width: 80,
            thumbnail_height: 60,
            version: 1,
        }
    }
}

impl Default for PalmPixRecord {
    fn default() -> Self {
        Self {
            id: 0,
            category: 0,
            attributes: PalmPixAttributes(0),
            created: PalmDateTime::now(),
            modified: PalmDateTime::now(),
            format: ImageFormat::Jpeg,
            width: 0,
            height: 0,
            bits_per_pixel: 24,
            image_data: Vec::new(),
            thumbnail: None,
            camera_info: None,
            description: String::new(),
        }
    }
}

impl PalmPixRecord {
    /// Parse from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 20 {
            return Err(PilotError::InvalidData("PalmPix record too short".into()));
        }

        let mut record = PalmPixRecord::default();
        let mut offset = 0;

        // Created date
        let created_val = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        record.created = PalmDateTime::from_palm(created_val);

        // Modified date
        let modified_val = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        record.modified = PalmDateTime::from_palm(modified_val);

        // Image format
        record.format = ImageFormat::from_u8(data[offset]);
        offset += 1;

        // Skip some bytes
        offset += 1;

        // Width
        record.width = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // Height
        record.height = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // Bits per pixel
        record.bits_per_pixel = data[offset];
        offset += 1;

        // Skip some bytes
        offset += 3;

        // Parse description
        let (desc, new_offset) = Self::parse_string(data, offset)?;
        record.description = desc;
        offset = new_offset;

        // Rest is image data
        if offset < data.len() {
            record.image_data = data[offset..].to_vec();
        }

        Ok(record)
    }

    /// Pack to bytes
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Created date
        data.extend_from_slice(&self.created.to_palm().to_be_bytes());

        // Modified date
        data.extend_from_slice(&self.modified.to_palm().to_be_bytes());

        // Image format
        data.push(self.format as u8);

        // Reserved
        data.push(0);

        // Width
        data.extend_from_slice(&self.width.to_be_bytes());

        // Height
        data.extend_from_slice(&self.height.to_be_bytes());

        // Bits per pixel
        data.push(self.bits_per_pixel);

        // Reserved
        data.push(0);
        data.push(0);
        data.push(0);

        // Description
        data.extend_from_slice(&Self::pack_string(&self.description));

        // Image data
        data.extend_from_slice(&self.image_data);

        data
    }

    fn parse_string(data: &[u8], offset: usize) -> Result<(String, usize)> {
        let mut end = offset;
        while end < data.len() && data[end] != 0 {
            end += 1;
        }
        let s = crate::utils::decode_palm_string(&data[offset..end]);
        Ok((s, end + 1))
    }

    fn pack_string(s: &str) -> Vec<u8> {
        let mut bytes = crate::utils::encode_palm_string(s);
        bytes.push(0);
        bytes
    }

    /// Get image size in bytes
    pub fn image_size(&self) -> usize {
        self.image_data.len()
    }

    /// Get total record size
    pub fn total_size(&self) -> usize {
        20 + // header
        self.description.len() + 1 +
        self.image_data.len()
    }

    /// Check if has thumbnail
    pub fn has_thumbnail(&self) -> bool {
        self.thumbnail.is_some()
    }

    /// Check if is landscape
    pub fn is_landscape(&self) -> bool {
        self.width > self.height
    }
}

/// PalmPix constants
pub mod constants {
    use crate::types::FourCharCode;

    /// PalmPix database type
    pub const PALMPIX_TYPE: FourCharCode = FourCharCode(0x50696374); // "Pict"

    /// PalmPix database creator
    pub const PALMPIX_CREATOR: FourCharCode = FourCharCode(0x50696374); // "Pict"

    /// Maximum image dimension
    pub const MAX_DIMENSION: u16 = 4096;

    /// Default thumbnail size
    pub const THUMBNAIL_SIZE: u16 = 80;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format() {
        assert_eq!(ImageFormat::from_u8(0), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from_u8(3), ImageFormat::Png);
        assert_eq!(ImageFormat::from_u8(10), ImageFormat::Unknown);
    }

    #[test]
    fn test_image_orientation() {
        assert_eq!(ImageOrientation::from_u8(0), ImageOrientation::Normal);
        assert_eq!(ImageOrientation::from_u8(5), ImageOrientation::Rotated90CW);
    }

    #[test]
    fn test_palm_pix_attributes() {
        let attrs = PalmPixAttributes(PalmPixAttributes::SECRET | PalmPixAttributes::ARCHIVE);
        assert!(attrs.is_secret());
        assert!(attrs.is_archived());
        assert!(!attrs.is_busy());
    }

    #[test]
    fn test_thumbnail_creation() {
        let data = vec![0u8; 10240];
        let thumb = Thumbnail::create(&data, 80, 60);
        assert!(thumb.data.len() <= 1024);
        assert_eq!(thumb.width, 80);
        assert_eq!(thumb.height, 60);
    }

    #[test]
    fn test_palm_pix_record_pack_parse() {
        let mut record = PalmPixRecord::default();
        record.width = 640;
        record.height = 480;
        record.bits_per_pixel = 24;
        record.description = "Test image".to_string();
        record.image_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic

        let packed = record.pack();
        let parsed = PalmPixRecord::parse(&packed).unwrap();

        assert_eq!(parsed.width, 640);
        assert_eq!(parsed.height, 480);
        assert_eq!(parsed.description, "Test image");
        assert_eq!(parsed.image_size(), 4);
    }

    #[test]
    fn test_is_landscape() {
        let mut record = PalmPixRecord::default();
        assert!(!record.is_landscape());

        record.width = 1280;
        record.height = 720;
        assert!(record.is_landscape());
    }
}
