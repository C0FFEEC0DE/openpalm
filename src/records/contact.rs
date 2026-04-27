//! Contact record types for Palm OS
//!
//! This module provides extended contact record parsing and serialization.

use crate::error::{PilotError, Result};
use crate::types::PalmDateTime;

/// Contact record (extended address book)
#[derive(Debug, Clone)]
pub struct ContactRecord {
    /// Record ID
    pub id: u32,
    /// Category
    pub category: u8,
    /// Attributes
    pub attributes: ContactAttributes,
    /// Name
    pub name: ContactName,
    /// Company
    pub company: String,
    /// Phone numbers
    pub phones: Vec<PhoneNumber>,
    /// Email addresses
    pub emails: Vec<String>,
    /// Addresses
    pub addresses: Vec<PostalAddress>,
    /// Instant messaging
    pub im: Vec<ImAddress>,
    /// Web sites
    pub websites: Vec<String>,
    /// Custom fields
    pub custom: Vec<CustomField>,
    /// Note
    pub note: String,
    /// Birthday
    pub birthday: Option<PalmDateTime>,
    /// Anniversary
    pub anniversary: Option<PalmDateTime>,
}

impl Default for ContactRecord {
    fn default() -> Self {
        Self {
            id: 0,
            category: 0,
            attributes: ContactAttributes(0),
            name: ContactName::default(),
            company: String::new(),
            phones: Vec::new(),
            emails: Vec::new(),
            addresses: Vec::new(),
            im: Vec::new(),
            websites: Vec::new(),
            custom: Vec::new(),
            note: String::new(),
            birthday: None,
            anniversary: None,
        }
    }
}

/// Contact attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContactAttributes(u8);

impl ContactAttributes {
    pub const SECRET: u8 = 0x02;
    pub const BUSY: u8 = 0x20;
    pub const ARCHIVE: u8 = 0x10;

    pub fn is_secret(&self) -> bool { (self.0 & Self::SECRET) != 0 }
    pub fn is_busy(&self) -> bool { (self.0 & Self::BUSY) != 0 }
    pub fn is_archived(&self) -> bool { (self.0 & Self::ARCHIVE) != 0 }
}

/// Contact name
#[derive(Debug, Clone, Default)]
pub struct ContactName {
    pub title: String,
    pub first: String,
    pub middle: String,
    pub last: String,
    pub suffix: String,
    pub company: String,
}

impl ContactName {
    pub fn full_name(&self) -> String {
        let mut parts = Vec::new();
        if !self.title.is_empty() { parts.push(self.title.as_str()); }
        if !self.first.is_empty() { parts.push(self.first.as_str()); }
        if !self.middle.is_empty() { parts.push(self.middle.as_str()); }
        if !self.last.is_empty() { parts.push(self.last.as_str()); }
        if !self.suffix.is_empty() { parts.push(self.suffix.as_str()); }
        parts.join(" ")
    }

    /// Get sort name (last, first)
    pub fn sort_name(&self) -> String {
        if self.last.is_empty() {
            self.full_name()
        } else {
            format!("{}, {}", self.last, self.first)
        }
    }
}

/// Phone number
#[derive(Debug, Clone)]
pub struct PhoneNumber {
    pub label: PhoneLabel,
    pub number: String,
    pub is_primary: bool,
}

/// Phone labels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PhoneLabel {
    Mobile = 0,
    Phone = 1,
    Work = 2,
    Home = 3,
    Fax = 4,
    Other = 5,
    Email = 6,
    Main = 7,
    Pager = 8,
}

impl PhoneLabel {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => PhoneLabel::Mobile,
            1 => PhoneLabel::Phone,
            2 => PhoneLabel::Work,
            3 => PhoneLabel::Home,
            4 => PhoneLabel::Fax,
            5 => PhoneLabel::Other,
            6 => PhoneLabel::Email,
            7 => PhoneLabel::Main,
            8 => PhoneLabel::Pager,
            _ => PhoneLabel::Other,
        }
    }
    
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// Postal address
#[derive(Debug, Clone)]
pub struct PostalAddress {
    pub label: AddressLabel,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
    pub is_primary: bool,
}

impl Default for PostalAddress {
    fn default() -> Self {
        Self {
            label: AddressLabel::Home,
            street: String::new(),
            city: String::new(),
            state: String::new(),
            zip: String::new(),
            country: String::new(),
            is_primary: false,
        }
    }
}

impl PostalAddress {
    /// Get formatted address
    pub fn format(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if !self.street.is_empty() { parts.push(&self.street); }
        
        let mut line2 = String::new();
        if !self.city.is_empty() {
            line2.push_str(&self.city);
            if !self.state.is_empty() { line2.push(' '); }
        }
        if !self.state.is_empty() { line2.push_str(&self.state); }
        if !self.zip.is_empty() { 
            if !line2.is_empty() { line2.push(' '); }
            line2.push_str(&self.zip);
        }
        if !line2.is_empty() { parts.push(&line2); }
        
        if !self.country.is_empty() { parts.push(&self.country); }
        
        parts.join("\n")
    }
}

/// Address labels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AddressLabel {
    Home = 0,
    Work = 1,
    Other = 2,
}

impl AddressLabel {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => AddressLabel::Home,
            1 => AddressLabel::Work,
            _ => AddressLabel::Other,
        }
    }
}

/// Instant messaging address
#[derive(Debug, Clone)]
pub struct ImAddress {
    pub service: ImService,
    pub username: String,
}

impl Default for ImAddress {
    fn default() -> Self {
        Self {
            service: ImService::Other,
            username: String::new(),
        }
    }
}

/// IM services
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ImService {
    AIM = 0,
    Yahoo = 1,
    MSN = 2,
    ICQ = 3,
    Jabber = 4,
    Other = 5,
}

impl ImService {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => ImService::AIM,
            1 => ImService::Yahoo,
            2 => ImService::MSN,
            3 => ImService::ICQ,
            4 => ImService::Jabber,
            _ => ImService::Other,
        }
    }
}

/// Custom field
#[derive(Debug, Clone)]
pub struct CustomField {
    pub id: u8,
    pub name: String,
    pub value: String,
}

impl ContactRecord {
    /// Parse from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 10 {
            return Err(PilotError::InvalidData("Contact record too short".into()));
        }

        let mut record = ContactRecord::default();
        let mut offset = 0;

        // Parse name
        let (first, new_offset) = Self::parse_string(data, offset)?;
        record.name.first = first;
        offset = new_offset;

        let (last, new_offset) = Self::parse_string(data, offset)?;
        record.name.last = last;
        offset = new_offset;

        let (company, new_offset) = Self::parse_string(data, offset)?;
        record.company = company;
        offset = new_offset;

        // Parse phones
        let phone_count = data[offset] as usize;
        offset += 1;
        
        for _ in 0..phone_count {
            let label = PhoneLabel::from_u8(data[offset]);
            offset += 1;
            let (number, new_offset) = Self::parse_string(data, offset)?;
            record.phones.push(PhoneNumber {
                label,
                number,
                is_primary: false,
            });
            offset = new_offset;
        }

        // Parse emails
        let email_count = data[offset] as usize;
        offset += 1;
        
        for _ in 0..email_count {
            let (email, new_offset) = Self::parse_string(data, offset)?;
            record.emails.push(email);
            offset = new_offset;
        }

        // Parse addresses
        let addr_count = data[offset] as usize;
        offset += 1;
        
        for _ in 0..addr_count {
            let label = AddressLabel::from_u8(data[offset]);
            offset += 1;
            
            let (street, new_offset) = Self::parse_string(data, offset)?;
            offset = new_offset;
            let (city, new_offset) = Self::parse_string(data, offset)?;
            offset = new_offset;
            let (state, new_offset) = Self::parse_string(data, offset)?;
            offset = new_offset;
            let (zip, new_offset) = Self::parse_string(data, offset)?;
            offset = new_offset;
            let (country, new_offset) = Self::parse_string(data, offset)?;
            offset = new_offset;
            
            record.addresses.push(PostalAddress {
                label,
                street,
                city,
                state,
                zip,
                country,
                is_primary: false,
            });
        }

        // Parse note (rest of data)
        if offset < data.len() {
            let (note, _) = Self::parse_string(data, offset)?;
            record.note = note;
        }

        Ok(record)
    }

    /// Pack to bytes
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Name
        data.extend_from_slice(&Self::pack_string(&self.name.first));
        data.extend_from_slice(&Self::pack_string(&self.name.last));
        data.extend_from_slice(&Self::pack_string(&self.company));

        // Phones
        data.push(self.phones.len() as u8);
        for phone in &self.phones {
            data.push(phone.label.as_u8());
            data.extend_from_slice(&Self::pack_string(&phone.number));
        }

        // Emails
        data.push(self.emails.len() as u8);
        for email in &self.emails {
            data.extend_from_slice(&Self::pack_string(email));
        }

        // Addresses
        data.push(self.addresses.len() as u8);
        for addr in &self.addresses {
            data.push(addr.label as u8);
            data.extend_from_slice(&Self::pack_string(&addr.street));
            data.extend_from_slice(&Self::pack_string(&addr.city));
            data.extend_from_slice(&Self::pack_string(&addr.state));
            data.extend_from_slice(&Self::pack_string(&addr.zip));
            data.extend_from_slice(&Self::pack_string(&addr.country));
        }

        // Note
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
}

/// Contact constants
pub mod constants {
    use crate::types::FourCharCode;

    /// Contact database type
    pub const CONTACT_TYPE: FourCharCode = FourCharCode { 0: 0x41444452 }; // "ADDR"
    
    /// Contact database creator
    pub const CONTACT_CREATOR: FourCharCode = FourCharCode { 0: 0x41444452 }; // "ADDR"

    /// Maximum phone numbers
    pub const MAX_PHONES: usize = 8;
    
    /// Maximum email addresses
    pub const MAX_EMAILS: usize = 4;
    
    /// Maximum addresses
    pub const MAX_ADDRESSES: usize = 3;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_name() {
        let name = ContactName {
            first: "John".to_string(),
            last: "Doe".to_string(),
            ..Default::default()
        };
        
        assert_eq!(name.full_name(), "John Doe");
        assert_eq!(name.sort_name(), "Doe, John");
    }

    #[test]
    fn test_phone_label() {
        assert_eq!(PhoneLabel::from_u8(0), PhoneLabel::Mobile);
        assert_eq!(PhoneLabel::from_u8(10), PhoneLabel::Other);
    }

    #[test]
    fn test_postal_address_format() {
        let addr = PostalAddress {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
            state: "IL".to_string(),
            zip: "12345".to_string(),
            country: "USA".to_string(),
            ..Default::default()
        };
        
        let formatted = addr.format();
        assert!(formatted.contains("123 Main St"));
        assert!(formatted.contains("Springfield"));
    }

    #[test]
    fn test_contact_record_pack_parse() {
        let mut record = ContactRecord::default();
        record.name.first = "Jane".to_string();
        record.name.last = "Smith".to_string();
        record.company = "Acme Corp".to_string();
        record.phones.push(PhoneNumber {
            label: PhoneLabel::Mobile,
            number: "555-1234".to_string(),
            is_primary: true,
        });
        record.emails.push("jane@example.com".to_string());

        let packed = record.pack();
        let parsed = ContactRecord::parse(&packed).unwrap();
        
        assert_eq!(parsed.name.first, "Jane");
        assert_eq!(parsed.name.last, "Smith");
        assert_eq!(parsed.phones.len(), 1);
        assert_eq!(parsed.emails.len(), 1);
    }
}
