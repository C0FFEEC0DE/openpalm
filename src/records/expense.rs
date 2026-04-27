//! Expense record types for Palm OS
//!
//! This module provides expense tracking record parsing and serialization.

use crate::error::{PilotError, Result};
use crate::types::{FourCharCode, PalmDateTime};

/// Expense record
#[derive(Debug, Clone)]
pub struct ExpenseRecord {
    /// Record ID
    pub id: u32,
    /// Category
    pub category: u8,
    /// Attributes
    pub attributes: u8,
    /// Expense type
    pub expense_type: ExpenseType,
    /// Amount (in cents/currency units)
    pub amount: i32,
    /// Currency code (ISO 4217)
    pub currency: [u8; 4],
    /// Date
    pub date: PalmDateTime,
    /// Date paid
    pub date_paid: PalmDateTime,
    /// Vendor name
    pub vendor: String,
    /// Description
    pub description: String,
    /// City
    pub city: String,
    /// Attendees
    pub attendees: Vec<String>,
    /// Note
    pub note: String,
    /// Payment type
    pub payment_type: PaymentType,
    /// Billable flag
    pub billable: bool,
}

/// Expense types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExpenseType {
    None = 0,
    Airfare = 1,
    Ground = 2,
    Lodging = 3,
    Meal = 4,
    Beverage = 5,
    Entertainment = 6,
    Conference = 7,
    Gift = 8,
    Postage = 9,
    Taxi = 10,
    Rental = 11,
    Fuel = 12,
    Parking = 13,
    Tips = 14,
    Other = 15,
}

impl ExpenseType {
    /// Parse from byte
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(ExpenseType::None),
            1 => Some(ExpenseType::Airfare),
            2 => Some(ExpenseType::Ground),
            3 => Some(ExpenseType::Lodging),
            4 => Some(ExpenseType::Meal),
            5 => Some(ExpenseType::Beverage),
            6 => Some(ExpenseType::Entertainment),
            7 => Some(ExpenseType::Conference),
            8 => Some(ExpenseType::Gift),
            9 => Some(ExpenseType::Postage),
            10 => Some(ExpenseType::Taxi),
            11 => Some(ExpenseType::Rental),
            12 => Some(ExpenseType::Fuel),
            13 => Some(ExpenseType::Parking),
            14 => Some(ExpenseType::Tips),
            15 => Some(ExpenseType::Other),
            _ => None,
        }
    }
}

/// Payment types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PaymentType {
    Cash = 0,
    Credit = 1,
    Debit = 2,
    Check = 3,
    ATM = 4,
    Wire = 5,
    Other = 6,
}

impl PaymentType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(PaymentType::Cash),
            1 => Some(PaymentType::Credit),
            2 => Some(PaymentType::Debit),
            3 => Some(PaymentType::Check),
            4 => Some(PaymentType::ATM),
            5 => Some(PaymentType::Wire),
            6 => Some(PaymentType::Other),
            _ => None,
        }
    }
}

/// Expense application info
#[derive(Debug, Clone)]
pub struct ExpenseAppInfo {
    /// Categories
    pub categories: Vec<String>,
    /// Currency list
    pub currencies: Vec<CurrencyInfo>,
    /// Payment types
    pub payment_types: Vec<String>,
    /// Version
    pub version: u16,
}

/// Currency information
#[derive(Debug, Clone)]
pub struct CurrencyInfo {
    /// Currency symbol
    pub symbol: [u8; 4],
    /// Currency name
    pub name: String,
    /// Decimal places
    pub decimals: u8,
}

impl ExpenseRecord {
    /// Parse from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 28 {
            return Err(PilotError::InvalidData("Expense record too short".into()));
        }

        let mut offset = 0;

        // Parse fixed fields
        let expense_type = ExpenseType::from_u8(data[offset])
            .unwrap_or(ExpenseType::Other);
        offset += 1;

        // Skip some bytes
        offset += 2;

        let amount = i32::from_le_bytes([
            data[offset], data[offset + 1],
            data[offset + 2], data[offset + 3],
        ]);
        offset += 4;

        let currency = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
        offset += 4;

        // Date (Palm format)
        let date_val = u32::from_le_bytes([
            data[offset], data[offset + 1],
            data[offset + 2], data[offset + 3],
        ]);
        offset += 4;
        let date = PalmDateTime::from_palm(date_val);

        // Date paid
        let date_paid_val = u32::from_le_bytes([
            data[offset], data[offset + 1],
            data[offset + 2], data[offset + 3],
        ]);
        offset += 4;
        let date_paid = PalmDateTime::from_palm(date_paid_val);

        // Payment type
        let payment_type = PaymentType::from_u8(data[offset]).unwrap_or(PaymentType::Other);
        offset += 1;

        // Skip some bytes
        offset += 3;

        // Billable flag
        let billable = (data[offset] & 0x01) != 0;
        offset += 1;

        // Parse strings
        let (vendor, new_offset) = Self::parse_string(data, offset)?;
        offset = new_offset;

        let (description, new_offset) = Self::parse_string(data, offset)?;
        offset = new_offset;

        let (city, new_offset) = Self::parse_string(data, offset)?;
        offset = new_offset;

        let (note, _) = Self::parse_string(data, offset)?;

        Ok(Self {
            id: 0, // Set by caller
            category: 0, // Set by caller
            attributes: 0,
            expense_type,
            amount,
            currency,
            date,
            date_paid,
            vendor,
            description,
            city,
            attendees: Vec::new(),
            note,
            payment_type,
            billable,
        })
    }

    /// Pack to bytes
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Expense type
        data.push(self.expense_type as u8);
        data.push(0); // Reserved
        data.push(0); // Reserved

        // Amount
        data.extend_from_slice(&self.amount.to_le_bytes());

        // Currency
        data.extend_from_slice(&self.currency);

        // Date
        data.extend_from_slice(&self.date.to_palm().to_le_bytes());

        // Date paid
        data.extend_from_slice(&self.date_paid.to_palm().to_le_bytes());

        // Payment type
        data.push(self.payment_type as u8);
        data.push(0); // Reserved
        data.push(0); // Reserved
        data.push(0); // Reserved

        // Billable flag
        data.push(if self.billable { 0x01 } else { 0x00 });

        // Strings
        data.extend_from_slice(&Self::pack_string(&self.vendor));
        data.extend_from_slice(&Self::pack_string(&self.description));
        data.extend_from_slice(&Self::pack_string(&self.city));
        data.extend_from_slice(&Self::pack_string(&self.note));

        data
    }

    /// Parse null-terminated string
    fn parse_string(data: &[u8], offset: usize) -> Result<(String, usize)> {
        let mut end = offset;
        while end < data.len() && data[end] != 0 {
            end += 1;
        }

        let s = String::from_utf8_lossy(&data[offset..end]).to_string();
        Ok((s, end + 1))
    }

    /// Pack string as null-terminated
    fn pack_string(s: &str) -> Vec<u8> {
        let mut bytes = s.as_bytes().to_vec();
        bytes.push(0);
        bytes
    }
}

/// Expense constants
pub mod constants {
    use crate::types::FourCharCode;

    /// Expense database type
    pub const EXPENSE_TYPE: FourCharCode = FourCharCode { 0: 0x45787073 };
    
    /// Expense database creator
    pub const EXPENSE_CREATOR: FourCharCode = FourCharCode { 0: 0x4578706E };
    
    /// Maximum expense amount
    pub const MAX_AMOUNT: i32 = 999_999_99;
    
    /// Currency codes
    pub const CURRENCY_USD: [u8; 4] = [0x55, 0x53, 0x44, 0x00]; // "USD"
    pub const CURRENCY_EUR: [u8; 4] = [0x45, 0x55, 0x52, 0x00]; // "EUR"
    pub const CURRENCY_GBP: [u8; 4] = [0x47, 0x42, 0x50, 0x00]; // "GBP"
    pub const CURRENCY_JPY: [u8; 4] = [0x4A, 0x50, 0x59, 0x00]; // "JPY";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expense_type() {
        assert_eq!(ExpenseType::from_u8(1), Some(ExpenseType::Airfare));
        assert_eq!(ExpenseType::from_u8(15), Some(ExpenseType::Other));
        assert_eq!(ExpenseType::from_u8(16), None);
    }

    #[test]
    fn test_payment_type() {
        assert_eq!(PaymentType::from_u8(0), Some(PaymentType::Cash));
        assert_eq!(PaymentType::from_u8(6), Some(PaymentType::Other));
    }

    #[test]
    fn test_expense_record_pack_parse() {
        let record = ExpenseRecord {
            id: 1,
            category: 0,
            attributes: 0,
            expense_type: ExpenseType::Meal,
            amount: 2500,
            currency: [0x55, 0x53, 0x44, 0x00],
            date: PalmDateTime::now(),
            date_paid: PalmDateTime::now(),
            vendor: "Restaurant".to_string(),
            description: "Business lunch".to_string(),
            city: "NYC".to_string(),
            attendees: Vec::new(),
            note: "With client".to_string(),
            payment_type: PaymentType::Credit,
            billable: true,
        };

        let packed = record.pack();
        let parsed = ExpenseRecord::parse(&packed).unwrap();
        
        assert_eq!(parsed.expense_type, ExpenseType::Meal);
        assert_eq!(parsed.amount, 2500);
        assert_eq!(parsed.vendor, "Restaurant");
    }
}
