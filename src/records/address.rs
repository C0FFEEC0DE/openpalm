//! Address book record parsing
//!
//! This module implements parsing for Palm OS Address DB records.
//! Based on pilot-link's address.c

use crate::error::Result;

/// Address entry indices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AddressEntry {
    /// Last name
    LastName = 0,
    /// First name
    FirstName = 1,
    /// Company
    Company = 2,
    /// Phone 1
    Phone1 = 3,
    /// Phone 2
    Phone2 = 4,
    /// Phone 3
    Phone3 = 5,
    /// Phone 4
    Phone4 = 6,
    /// Phone 5
    Phone5 = 7,
    /// Main phone
    MainPhone = 8,
    /// Other phone
    OtherPhone = 9,
    /// Email 1
    Email1 = 10,
    /// Email 2
    Email2 = 11,
    /// Email 3
    Email3 = 12,
    /// Address
    Address = 13,
    /// City
    City = 14,
    /// State
    State = 15,
    /// ZIP code
    ZipCode = 16,
    /// Country
    Country = 17,
    /// Title (job title)
    Title = 18,
    /// Custom field 1
    Custom1 = 19,
    /// Custom field 2
    Custom2 = 20,
    /// Custom field 3
    Custom3 = 21,
    /// Custom field 4
    Custom4 = 22,
    /// Note/Comments
    Note = 23,
}

/// Address record
#[derive(Debug, Clone)]
pub struct AddressRecord {
    /// Show phone (which phone to highlight)
    pub show_phone: u8,
    /// Phone labels
    pub phone_labels: [u8; 5],
    /// Entries (up to 24)
    pub entry: [Option<String>; 24],
}

impl Default for AddressRecord {
    fn default() -> Self {
        Self {
            show_phone: 0,
            phone_labels: [0; 5],
            entry: [const { None }; 24],
        }
    }
}

impl AddressRecord {
    /// Create a new empty address record
    pub fn new() -> Self {
        Self::default()
    }

    /// Unpack from record data (address_v1 format)
    pub fn unpack(data: &[u8]) -> Result<Self> {
        if data.len() < 9 {
            return Err(crate::error::PilotError::DlpBufSize);
        }

        let mut record = Self::default();

        // Parse phone flag bytes
        let byte1 = data[1];
        let byte2 = data[2];
        let byte3 = data[3];

        record.show_phone = (byte1 >> 4) & 0x0F;
        record.phone_labels[4] = byte1 & 0x0F;
        record.phone_labels[3] = (byte2 >> 4) & 0x0F;
        record.phone_labels[2] = byte2 & 0x0F;
        record.phone_labels[1] = (byte3 >> 4) & 0x0F;
        record.phone_labels[0] = byte3 & 0x0F;

        // Parse contents bitmask
        let contents = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        // Offset to start of string data
        let mut offset = 9usize;

        // Parse each entry if corresponding bit is set
        for i in 0..24 {
            if contents & (1 << i) != 0 {
                if offset >= data.len() {
                    break;
                }

                // Find null terminator
                let end = data[offset..]
                    .iter()
                    .position(|&b| b == 0)
                    .map(|p| offset + p)
                    .unwrap_or(data.len());

                if offset < end {
                    let s = crate::utils::decode_palm_string(&data[offset..end]);
                    record.entry[i] = Some(s);
                }

                offset = end + 1;
            }
        }

        Ok(record)
    }

    /// Pack to record data (address_v1 format)
    pub fn pack(&self) -> Vec<u8> {
        // First calculate total size
        let mut size = 9; // header
        for entry in &self.entry {
            if entry.is_some() {
                size += entry.as_ref().unwrap().len() + 1;
            }
        }

        let mut data = vec![0u8; size];

        // Build contents bitmask
        let mut contents: u32 = 0;
        let mut string_start: Option<usize> = None;

        for (i, entry) in self.entry.iter().enumerate() {
            if entry.is_some() {
                contents |= 1 << i;

                if string_start.is_none() {
                    string_start = Some(9);
                }
            }
        }

        // Phone flags
        let phone_flag: u32 = (self.phone_labels[0] as u32)
            | ((self.phone_labels[1] as u32) << 4)
            | ((self.phone_labels[2] as u32) << 8)
            | ((self.phone_labels[3] as u32) << 12)
            | ((self.phone_labels[4] as u32) << 16)
            | ((self.show_phone as u32) << 20);

        data[0..4].copy_from_slice(&phone_flag.to_be_bytes());
        data[4..8].copy_from_slice(&contents.to_be_bytes());
        data[8] = string_start.map(|s| s as u8).unwrap_or(9).saturating_sub(8);

        // Write strings
        let mut offset = 9;
        for s in self.entry.iter().flatten() {
            let bytes = s.as_bytes();
            data[offset..offset + bytes.len()].copy_from_slice(bytes);
            offset += bytes.len();
            data[offset] = 0;
            offset += 1;
        }

        data
    }

    /// Get entry by type
    pub fn get(&self, entry: AddressEntry) -> Option<&str> {
        self.entry[entry as usize].as_deref()
    }

    /// Set entry by type
    pub fn set(&mut self, entry: AddressEntry, value: Option<String>) {
        self.entry[entry as usize] = value;
    }

    /// Get full name
    pub fn full_name(&self) -> String {
        let parts: Vec<&str> = [
            self.entry[AddressEntry::FirstName as usize].as_deref(),
            self.entry[AddressEntry::LastName as usize].as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect();

        if parts.is_empty() {
            self.entry[AddressEntry::Company as usize]
                .as_deref()
                .unwrap_or("(no name)")
                .to_string()
        } else {
            parts.join(" ")
        }
    }
}

/// Phone label types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PhoneLabel {
    /// Phone
    Phone = 0,
    /// Work phone
    Work = 1,
    /// Home phone
    Home = 2,
    /// Fax
    Fax = 3,
    /// Other
    Other = 4,
    /// Main
    Main = 5,
    /// Mobile
    Mobile = 6,
    /// Palm phone
    Palm = 7,
    /// SMS
    Sms = 8,
    /// Email
    Email = 9,
}

impl PhoneLabel {
    pub fn from_u8(val: u8) -> Self {
        match val & 0x0F {
            0 => PhoneLabel::Phone,
            1 => PhoneLabel::Work,
            2 => PhoneLabel::Home,
            3 => PhoneLabel::Fax,
            4 => PhoneLabel::Other,
            5 => PhoneLabel::Main,
            6 => PhoneLabel::Mobile,
            7 => PhoneLabel::Palm,
            8 => PhoneLabel::Sms,
            9 => PhoneLabel::Email,
            _ => PhoneLabel::Phone,
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

/// Address app info (category information)
#[derive(Debug, Clone, Default)]
pub struct AddressAppInfo {
    /// Category assignments
    pub categories: Vec<crate::database::Category>,
    /// Last unique ID
    pub last_unique_id: u16,
    /// Country code
    pub country: u16,
    /// Reserved
    pub reserved: Vec<u8>,
}

impl AddressAppInfo {
    /// Parse from app info data
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 277 {
            return Err(crate::error::PilotError::DlpBufSize);
        }

        let (categories, last_uniq_id, rest) = crate::database::parse_categories(data)?;
        let country = u16::from_be_bytes([rest[0], rest[1]]);

        Ok(Self {
            categories,
            last_unique_id: last_uniq_id as u16,
            country,
            reserved: rest[2..].to_vec(),
        })
    }

    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(277 + self.reserved.len());
        // renamedCategories bitfield
        let mut renamed: u16 = 0;
        for (i, cat) in self.categories.iter().enumerate() {
            if cat.flags != 0 {
                renamed |= 1 << i;
            }
        }
        data.extend_from_slice(&renamed.to_be_bytes());
        // Category labels
        for cat in &self.categories {
            data.extend_from_slice(&cat.name);
        }
        // Category unique IDs
        for cat in &self.categories {
            data.push(cat.reserved);
        }
        data.push(self.last_unique_id as u8);
        // Country code
        data.extend_from_slice(&self.country.to_be_bytes());
        data.extend_from_slice(&self.reserved);
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_record_new() {
        let record = AddressRecord::new();
        assert_eq!(record.show_phone, 0);
        assert_eq!(record.phone_labels, [0; 5]);
    }

    #[test]
    fn test_address_record_pack_unpack() {
        let mut record = AddressRecord::new();
        record.entry[AddressEntry::LastName as usize] = Some("Doe".to_string());
        record.entry[AddressEntry::FirstName as usize] = Some("John".to_string());
        record.entry[AddressEntry::Email1 as usize] = Some("john@example.com".to_string());

        let packed = record.pack();
        let unpacked = AddressRecord::unpack(&packed).unwrap();

        assert_eq!(unpacked.get(AddressEntry::LastName), Some("Doe"));
        assert_eq!(unpacked.get(AddressEntry::FirstName), Some("John"));
        assert_eq!(unpacked.get(AddressEntry::Email1), Some("john@example.com"));
    }

    #[test]
    fn test_address_phone_flags_roundtrip() {
        let mut record = AddressRecord::new();
        record.show_phone = 3;
        record.phone_labels = [1, 2, 3, 4, 5];
        record.entry[AddressEntry::LastName as usize] = Some("Smith".to_string());
        record.entry[AddressEntry::Phone1 as usize] = Some("555-1234".to_string());

        let packed = record.pack();
        let unpacked = AddressRecord::unpack(&packed).unwrap();

        assert_eq!(unpacked.show_phone, 3, "show_phone round-trip failed");
        assert_eq!(
            unpacked.phone_labels,
            [1, 2, 3, 4, 5],
            "phone_labels round-trip failed"
        );
        assert_eq!(unpacked.get(AddressEntry::LastName), Some("Smith"));
        assert_eq!(unpacked.get(AddressEntry::Phone1), Some("555-1234"));
    }

    #[test]
    fn test_address_app_info_categories() {
        // Build a Palm OS category AppInfo block (275 bytes)
        let mut data = vec![0u8; 275];
        // renamedCategories bitfield (2 bytes, big-endian)
        data[0..2].copy_from_slice(&0x0003u16.to_be_bytes()); // categories 0 and 1 renamed
                                                              // Category 0 label: "Personal" at offset 2
        data[2..10].copy_from_slice(b"Personal");
        // Category 1 label: "Business" at offset 18
        data[18..26].copy_from_slice(b"Business");
        // Category unique IDs at offset 258
        data[258] = 1; // cat 0 uniq id
        data[259] = 2; // cat 1 uniq id
                       // lastUniqID at offset 274
        data[274] = 3;
        // Country code follows at offset 275 (2 bytes)
        data.extend_from_slice(&0x0001u16.to_be_bytes());

        let info = AddressAppInfo::from_bytes(&data).unwrap();
        assert_eq!(info.categories.len(), 16);
        assert_eq!(info.categories[0].id, 0);
        assert_eq!(
            String::from_utf8_lossy(&info.categories[0].name).trim_end_matches('\0'),
            "Personal"
        );
        assert_eq!(info.categories[1].id, 1);
        assert_eq!(
            String::from_utf8_lossy(&info.categories[1].name).trim_end_matches('\0'),
            "Business"
        );
        assert_eq!(info.last_unique_id, 3);
        assert_eq!(info.country, 1);
    }

    #[test]
    fn test_full_name() {
        let mut record = AddressRecord::new();
        record.entry[AddressEntry::LastName as usize] = Some("Doe".to_string());
        record.entry[AddressEntry::FirstName as usize] = Some("John".to_string());

        assert_eq!(record.full_name(), "John Doe");
    }
}
