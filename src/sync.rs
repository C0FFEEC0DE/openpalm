//! Synchronization framework for Palm devices
//!
//! This module implements the synchronization logic for Palm OS devices.
//! Based on pilot-link's libpisync.

use crate::database::Record;
use std::collections::HashMap;

/// Sync direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    /// Sync from desktop to device
    DesktopToDevice,
    /// Sync from device to desktop
    DeviceToDesktop,
    /// Bidirectional sync
    Both,
    /// No sync (compare only)
    Compare,
}

/// Sync action for a record
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAction {
    /// Record should be added to the target
    Add,
    /// Record should be updated on target
    Update,
    /// Record should be deleted on target
    Delete,
    /// Record conflict detected
    Conflict,
    /// No action needed (records match)
    None,
}

/// Sync statistics
#[derive(Debug, Clone, Default)]
pub struct SyncStats {
    /// Number of records added
    pub added: u32,
    /// Number of records updated
    pub updated: u32,
    /// Number of records deleted
    pub deleted: u32,
    /// Number of conflicts
    pub conflicts: u32,
    /// Number of records skipped
    pub skipped: u32,
}

impl SyncStats {
    /// Add to stats
    pub fn add(&mut self, action: SyncAction) {
        match action {
            SyncAction::Add => self.added += 1,
            SyncAction::Update => self.updated += 1,
            SyncAction::Delete => self.deleted += 1,
            SyncAction::Conflict => self.conflicts += 1,
            SyncAction::None => self.skipped += 1,
        }
    }
    
    /// Total records processed
    pub fn total(&self) -> u32 {
        self.added + self.updated + self.deleted + self.conflicts + self.skipped
    }
}

/// Record for sync
#[derive(Debug, Clone)]
pub struct SyncRecord {
    /// Record ID
    pub id: u32,
    /// Category ID
    pub category: u8,
    /// Record attributes
    pub attributes: u8,
    /// Record data
    pub data: Vec<u8>,
    /// Modification number
    pub mod_num: u32,
}

impl SyncRecord {
    /// Create from database record
    pub fn from_record(record: &Record) -> Self {
        Self {
            id: record.id,
            category: record.category,
            attributes: record.attributes.bits(),
            data: record.data.clone(),
            mod_num: 0,
        }
    }
}

/// Sync handler trait
pub trait SyncHandler {
    /// Get the database name
    fn db_name(&self) -> &str;
    
    /// Get database type
    fn db_type(&self) -> u32;
    
    /// Get database creator
    fn db_creator(&self) -> u32;
    
    /// Get sync direction
    fn direction(&self) -> SyncDirection;
    
    /// Match desktop record to pilot record
    fn match_records(&self, desktop: &SyncRecord, pilot: Option<&SyncRecord>) -> bool;
    
    /// Merge records
    fn merge(&self, desktop: &mut SyncRecord, pilot: &SyncRecord) -> SyncAction;
    
    /// Free desktop match data
    fn free_match(&self, record: &mut SyncRecord);
}

/// Default sync handler
pub struct DefaultSyncHandler {
    db_name: String,
    db_type: u32,
    db_creator: u32,
    direction: SyncDirection,
}

impl DefaultSyncHandler {
    /// Create a new sync handler
    pub fn new(name: &str, db_type: u32, db_creator: u32, direction: SyncDirection) -> Self {
        Self {
            db_name: name.to_string(),
            db_type,
            db_creator,
            direction,
        }
    }
}

impl SyncHandler for DefaultSyncHandler {
    fn db_name(&self) -> &str {
        &self.db_name
    }
    
    fn db_type(&self) -> u32 {
        self.db_type
    }
    
    fn db_creator(&self) -> u32 {
        self.db_creator
    }
    
    fn direction(&self) -> SyncDirection {
        self.direction
    }
    
    fn match_records(&self, desktop: &SyncRecord, pilot: Option<&SyncRecord>) -> bool {
        // Simple match by ID
        if let Some(p) = pilot {
            desktop.id == p.id
        } else {
            false
        }
    }
    
    fn merge(&self, desktop: &mut SyncRecord, pilot: &SyncRecord) -> SyncAction {
        // Simple merge: desktop wins, unless it's empty
        if desktop.data.is_empty() && !pilot.data.is_empty() {
            desktop.data = pilot.data.clone();
            SyncAction::Update
        } else if pilot.data.is_empty() {
            SyncAction::Delete
        } else {
            SyncAction::None
        }
    }
    
    fn free_match(&self, _record: &mut SyncRecord) {
        // Nothing to free in simple implementation
    }
}

/// Sync processor
pub struct SyncProcessor {
    /// Stats
    stats: SyncStats,
    /// Handler
    handler: Box<dyn SyncHandler>,
}

impl SyncProcessor {
    /// Create a new sync processor
    pub fn new(handler: Box<dyn SyncHandler>) -> Self {
        Self {
            stats: SyncStats::default(),
            handler,
        }
    }
    
    /// Perform sync
    pub fn sync(&mut self, desktop_records: &[SyncRecord], pilot_records: &[SyncRecord]) -> SyncResult {
        let direction = self.handler.direction();
        let mut result = SyncResult::default();
        
        // Build lookup maps
        let mut pilot_map: HashMap<u32, &SyncRecord> = HashMap::new();
        for r in pilot_records {
            pilot_map.insert(r.id, r);
        }
        
        // Process desktop records
        for drec in desktop_records {
            let pilot_rec = pilot_map.get(&drec.id).copied();
            
            if self.handler.match_records(drec, pilot_rec) {
                // Records match - check for conflicts
                if let Some(prec) = pilot_rec {
                    if drec.data != prec.data {
                        match direction {
                            SyncDirection::DesktopToDevice => {
                                // Desktop wins
                                result.desktop_actions.push(SyncAction::Update);
                                result.pilot_actions.push(SyncAction::Delete);
                            }
                            SyncDirection::DeviceToDesktop => {
                                // Device wins
                                result.desktop_actions.push(SyncAction::Update);
                                result.pilot_actions.push(SyncAction::Delete);
                            }
                            _ => {
                                // Conflict
                                result.desktop_actions.push(SyncAction::Conflict);
                                result.pilot_actions.push(SyncAction::Conflict);
                                self.stats.add(SyncAction::Conflict);
                            }
                        }
                    } else {
                        result.desktop_actions.push(SyncAction::None);
                        result.pilot_actions.push(SyncAction::None);
                    }
                }
            } else {
                // No match - add to target
                if direction == SyncDirection::DesktopToDevice || direction == SyncDirection::Both {
                    result.pilot_actions.push(SyncAction::Add);
                    self.stats.add(SyncAction::Add);
                }
                result.desktop_actions.push(SyncAction::None);
            }
        }
        
        // Find records only on device
        let mut desktop_ids: HashMap<u32, bool> = HashMap::new();
        for r in desktop_records {
            desktop_ids.insert(r.id, true);
        }
        
        for prec in pilot_records {
            if !desktop_ids.contains_key(&prec.id) {
                // Record only on device
                if direction == SyncDirection::DeviceToDesktop || direction == SyncDirection::Both {
                    result.desktop_actions.push(SyncAction::Add);
                    self.stats.add(SyncAction::Add);
                }
            }
        }
        
        result
    }
    
    /// Get sync statistics
    pub fn stats(&self) -> &SyncStats {
        &self.stats
    }
}

/// Sync result
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    /// Actions for desktop records
    pub desktop_actions: Vec<SyncAction>,
    /// Actions for pilot records
    pub pilot_actions: Vec<SyncAction>,
}

/// Sync session
pub struct SyncSession {
    /// Active
    active: bool,
    /// Last sync time
    last_sync: u32,
    /// Sync stats
    stats: SyncStats,
}

impl SyncSession {
    /// Create a new sync session
    pub fn new() -> Self {
        Self {
            active: false,
            last_sync: 0,
            stats: SyncStats::default(),
        }
    }
    
    /// Start sync session
    pub fn start(&mut self) {
        self.active = true;
    }
    
    /// End sync session
    pub fn end(&mut self) {
        self.active = false;
    }
    
    /// Check if active
    pub fn is_active(&self) -> bool {
        self.active
    }
    
    /// Get last sync time
    pub fn last_sync_time(&self) -> u32 {
        self.last_sync
    }
    
    /// Update stats
    pub fn update_stats(&mut self, action: SyncAction) {
        self.stats.add(action);
    }
    
    /// Get stats
    pub fn stats(&self) -> &SyncStats {
        &self.stats
    }
}

impl Default for SyncSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Sync strategy alias for SyncDirection
pub type SyncStrategy = SyncDirection;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_stats() {
        let mut stats = SyncStats::default();
        stats.add(SyncAction::Add);
        stats.add(SyncAction::Update);
        stats.add(SyncAction::Conflict);
        
        assert_eq!(stats.added, 1);
        assert_eq!(stats.updated, 1);
        assert_eq!(stats.conflicts, 1);
        assert_eq!(stats.total(), 3);
    }

    #[test]
    fn test_sync_session() {
        let mut session = SyncSession::new();
        assert!(!session.is_active());
        
        session.start();
        assert!(session.is_active());
        
        session.update_stats(SyncAction::Add);
        assert_eq!(session.stats().added, 1);
    }
}
