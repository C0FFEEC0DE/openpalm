//! Money record types for Palm OS
//!
//! This module provides money/financial record parsing and serialization.

use crate::error::{PilotError, Result};
use crate::types::PalmDateTime;

/// Money record (financial tracking)
#[derive(Debug, Clone)]
pub struct MoneyRecord {
    /// Record ID
    pub id: u32,
    /// Category
    pub category: u8,
    /// Attributes
    pub attributes: MoneyAttributes,
    /// Date
    pub date: PalmDateTime,
    /// Amount
    pub amount: i32,
    /// Currency code
    pub currency: [u8; 4],
    /// Account
    pub account: String,
    /// Description
    pub description: String,
    /// Payee
    pub payee: String,
    /// Check number
    pub check_number: Option<u32>,
    /// Memo
    pub memo: String,
    /// Split items
    pub splits: Vec<MoneySplit>,
    /// Reconciled flag
    pub reconciled: bool,
}

/// Money attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoneyAttributes(u8);

impl MoneyAttributes {
    pub const SECRET: u8 = 0x02;
    pub const BUSY: u8 = 0x20;
    pub const ARCHIVE: u8 = 0x10;
    pub const DIRTY: u8 = 0x40;

    pub fn is_secret(&self) -> bool {
        (self.0 & Self::SECRET) != 0
    }
    pub fn is_busy(&self) -> bool {
        (self.0 & Self::BUSY) != 0
    }
    pub fn is_archived(&self) -> bool {
        (self.0 & Self::ARCHIVE) != 0
    }
    pub fn is_dirty(&self) -> bool {
        (self.0 & Self::DIRTY) != 0
    }
}

/// Transaction types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MoneyType {
    Check = 0,
    Deposit = 1,
    Transfer = 2,
    ATM = 3,
    Charge = 4,
    Payment = 5,
    Fee = 6,
    Interest = 7,
    Other = 8,
}

impl MoneyType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => MoneyType::Check,
            1 => MoneyType::Deposit,
            2 => MoneyType::Transfer,
            3 => MoneyType::ATM,
            4 => MoneyType::Charge,
            5 => MoneyType::Payment,
            6 => MoneyType::Fee,
            7 => MoneyType::Interest,
            _ => MoneyType::Other,
        }
    }
}

/// Money split item
#[derive(Debug, Clone)]
pub struct MoneySplit {
    /// Category
    pub category: String,
    /// Amount
    pub amount: i32,
    /// Memo
    pub memo: String,
}

impl Default for MoneyRecord {
    fn default() -> Self {
        Self {
            id: 0,
            category: 0,
            attributes: MoneyAttributes(0),
            date: PalmDateTime::now(),
            amount: 0,
            currency: [0x55, 0x53, 0x44, 0x00],
            account: String::new(),
            description: String::new(),
            payee: String::new(),
            check_number: None,
            memo: String::new(),
            splits: Vec::new(),
            reconciled: false,
        }
    }
}

impl MoneyRecord {
    /// Parse from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 20 {
            return Err(PilotError::InvalidData("Money record too short".into()));
        }

        let mut record = MoneyRecord::default();
        let mut offset = 0;

        // Parse date
        let date_val = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        record.date = PalmDateTime::from_palm(date_val);

        // Amount
        record.amount = i32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Currency
        record.currency = [
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ];
        offset += 4;

        // Reconciled flag
        record.reconciled = data[offset] != 0;
        offset += 1;

        // Skip some bytes
        offset += 2;

        // Parse strings
        let (account, new_offset) = Self::parse_string(data, offset)?;
        record.account = account;
        offset = new_offset;

        let (description, new_offset) = Self::parse_string(data, offset)?;
        record.description = description;
        offset = new_offset;

        let (payee, new_offset) = Self::parse_string(data, offset)?;
        record.payee = payee;
        offset = new_offset;

        let (memo, _) = Self::parse_string(data, offset)?;
        record.memo = memo;

        Ok(record)
    }

    /// Pack to bytes
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Date
        data.extend_from_slice(&self.date.to_palm().to_be_bytes());

        // Amount
        data.extend_from_slice(&self.amount.to_be_bytes());

        // Currency
        data.extend_from_slice(&self.currency);

        // Reconciled
        data.push(if self.reconciled { 1 } else { 0 });

        // Reserved
        data.push(0);
        data.push(0);

        // Strings
        data.extend_from_slice(&Self::pack_string(&self.account));
        data.extend_from_slice(&Self::pack_string(&self.description));
        data.extend_from_slice(&Self::pack_string(&self.payee));
        data.extend_from_slice(&Self::pack_string(&self.memo));

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

    /// Get amount as floating point
    pub fn amount_float(&self) -> f64 {
        (self.amount as f64) / 100.0
    }

    /// Set amount from floating point
    pub fn set_amount_float(&mut self, value: f64) {
        self.amount = (value * 100.0).round() as i32;
    }
}

/// Money account
#[derive(Debug, Clone)]
pub struct MoneyAccount {
    /// Account name
    pub name: String,
    /// Account type
    pub account_type: AccountType,
    /// Initial balance
    pub initial_balance: i32,
    /// Currency
    pub currency: [u8; 4],
    /// Low balance alert
    pub low_balance_alert: Option<i32>,
    /// Closed flag
    pub closed: bool,
}

/// Account types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AccountType {
    Checking = 0,
    Savings = 1,
    CreditCard = 2,
    MoneyMarket = 3,
    CD = 4,
    Investment = 5,
    Other = 6,
}

impl AccountType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => AccountType::Checking,
            1 => AccountType::Savings,
            2 => AccountType::CreditCard,
            3 => AccountType::MoneyMarket,
            4 => AccountType::CD,
            5 => AccountType::Investment,
            _ => AccountType::Other,
        }
    }
}

/// Money application info
#[derive(Debug, Clone)]
pub struct MoneyAppInfo {
    /// Accounts
    pub accounts: Vec<MoneyAccount>,
    /// Categories
    pub categories: Vec<String>,
    /// Currency
    pub currency: [u8; 4],
    /// Last account ID
    pub last_account_id: u32,
    /// Version
    pub version: u16,
}

impl Default for MoneyAppInfo {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            categories: vec!["Uncategorized".to_string()],
            currency: [0x55, 0x53, 0x44, 0x00],
            last_account_id: 0,
            version: 1,
        }
    }
}

/// Money constants
pub mod constants {
    use crate::types::FourCharCode;

    /// Money database type
    pub const MONEY_TYPE: FourCharCode = FourCharCode(0x4D6F6E65); // "Mone"

    /// Money database creator
    pub const MONEY_CREATOR: FourCharCode = FourCharCode(0x4D6F6E65); // "Mone"

    /// Maximum accounts
    pub const MAX_ACCOUNTS: usize = 20;

    /// Maximum splits per transaction
    pub const MAX_SPLITS: usize = 20;

    /// Decimal places
    pub const DECIMAL_PLACES: i32 = 2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_attributes() {
        let attrs = MoneyAttributes(MoneyAttributes::SECRET | MoneyAttributes::DIRTY);
        assert!(attrs.is_secret());
        assert!(attrs.is_dirty());
        assert!(!attrs.is_archived());
    }

    #[test]
    fn test_money_type() {
        assert_eq!(MoneyType::from_u8(0), MoneyType::Check);
        assert_eq!(MoneyType::from_u8(2), MoneyType::Transfer);
        assert_eq!(MoneyType::from_u8(10), MoneyType::Other);
    }

    #[test]
    fn test_account_type() {
        assert_eq!(AccountType::from_u8(0), AccountType::Checking);
        assert_eq!(AccountType::from_u8(5), AccountType::Investment);
    }

    #[test]
    fn test_money_amount() {
        let mut record = MoneyRecord::default();
        assert_eq!(record.amount_float(), 0.0);

        record.set_amount_float(123.45);
        assert_eq!(record.amount, 12345);
        assert_eq!(record.amount_float(), 123.45);
    }

    #[test]
    fn test_money_record_pack_parse() {
        let mut record = MoneyRecord::default();
        record.account = "Checking".to_string();
        record.description = "Grocery shopping".to_string();
        record.payee = "Whole Foods".to_string();
        record.set_amount_float(56.78);

        let packed = record.pack();
        let parsed = MoneyRecord::parse(&packed).unwrap();

        assert_eq!(parsed.account, "Checking");
        assert_eq!(parsed.description, "Grocery shopping");
        assert_eq!(parsed.amount_float(), 56.78);
    }
}
