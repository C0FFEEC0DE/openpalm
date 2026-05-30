//! PiBuffer - Variable size buffer management
//!
//! Provides reliable and easy to use variable size buffer management.

use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ptr;
use std::slice;
use std::fmt;

/// Buffer error type
#[derive(Debug, Clone)]
pub struct BufferError;

impl fmt::Display for BufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Buffer error: memory allocation failed")
    }
}

impl std::error::Error for BufferError {}

/// Result type alias for buffer operations
pub type BufferResult<T> = std::result::Result<T, BufferError>;

/// Minimum allocation size
const MIN_BUFFER_SIZE: usize = 16;

/// Default initial capacity
const DEFAULT_BUFFER_CAPACITY: usize = 256;

/// Buffer growth factor when resizing
const BUFFER_GROWTH_FACTOR: usize = 2;

/// Minimum growth amount
const BUFFER_MIN_GROWTH: usize = 16;

/// A variable-size buffer for storing binary data
#[derive(Debug)]
pub struct PiBuffer {
    data: *mut u8,
    capacity: usize,
    used: usize,
}

impl Clone for PiBuffer {
    fn clone(&self) -> Self {
        if self.data.is_null() || self.used == 0 {
            return Self::new(0);
        }
        let mut buf = Self::new(self.used);
        unsafe {
            ptr::copy_nonoverlapping(self.data, buf.data, self.used);
            buf.used = self.used;
        }
        buf
    }
}

impl PiBuffer {
    /// Create a new buffer with the specified initial capacity
    pub fn new(capacity: usize) -> Self {
        if capacity == 0 {
            return PiBuffer {
                data: ptr::null_mut(),
                capacity: 0,
                used: 0,
            };
        }

        let layout = Layout::from_size_align(capacity, 1).expect("Invalid layout");
        let data = unsafe { alloc(layout) };

        if data.is_null() {
            panic!("Failed to allocate buffer");
        }

        PiBuffer { data, capacity, used: 0 }
    }

    /// Create a buffer from existing data
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut buf = Self::new(data.len());
        let _ = buf.append(data);
        buf
    }

    /// Create a buffer with default capacity
    pub fn default() -> Self {
        Self::new(DEFAULT_BUFFER_CAPACITY)
    }

    /// Append data to the buffer
    pub fn append(&mut self, data: &[u8]) -> BufferResult<()> {
        self.reserve_more(data.len())?;
        
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.data.add(self.used), data.len());
            self.used += data.len();
        }
        Ok(())
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.used = 0;
    }

    /// Reserve capacity
    pub fn reserve(&mut self, new_capacity: usize) -> BufferResult<()> {
        if new_capacity <= self.capacity {
            return Ok(());
        }
        let new_cap = Self::compute_new_capacity(new_capacity);
        self.reallocate(new_cap)
    }

    /// Reserve more capacity
    fn reserve_more(&mut self, additional: usize) -> BufferResult<()> {
        let required = self.used.saturating_add(additional);
        self.reserve(required)
    }

    /// Get the current length
    #[inline]
    pub fn len(&self) -> usize {
        self.used
    }

    /// Check if the buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.used == 0
    }

    /// Get the total capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get a reference to the underlying data as a slice
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data, self.used) }
    }

    /// Get a mutable reference to the underlying data as a slice
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data, self.used) }
    }

    fn compute_new_capacity(requested: usize) -> usize {
        let double = DEFAULT_BUFFER_CAPACITY * BUFFER_GROWTH_FACTOR;
        let add_min = DEFAULT_BUFFER_CAPACITY + BUFFER_MIN_GROWTH;
        std::cmp::max(std::cmp::max(double, add_min), requested)
    }

    fn reallocate(&mut self, new_capacity: usize) -> BufferResult<()> {
        if new_capacity == 0 {
            self.reset();
            return Ok(());
        }

        let new_layout = Layout::from_size_align(new_capacity, 1).map_err(|_| BufferError)?;
        
        let new_data = if self.data.is_null() {
            unsafe { alloc(new_layout) }
        } else {
            let old_layout = Layout::from_size_align(self.capacity, 1).map_err(|_| BufferError)?;
            unsafe { realloc(self.data, old_layout, new_capacity) }
        };

        if new_data.is_null() {
            return Err(BufferError);
        }

        self.data = new_data;
        self.capacity = new_capacity;
        Ok(())
    }

    fn reset(&mut self) {
        if !self.data.is_null() {
            let layout = Layout::from_size_align(self.capacity, 1).unwrap();
            unsafe { dealloc(self.data, layout) };
            self.data = ptr::null_mut();
            self.capacity = 0;
            self.used = 0;
        }
    }

    /// Compare the buffer contents with another slice
    pub fn cmp(&self, other: &[u8]) -> std::cmp::Ordering {
        self.as_slice().cmp(other)
    }

    /// Check if the buffer starts with a given prefix
    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        self.as_slice().starts_with(prefix)
    }

    /// Find a byte sequence in the buffer
    pub fn find(&self, needle: &[u8]) -> Option<usize> {
        self.as_slice().windows(needle.len()).position(|window| window == needle)
    }

    /// Extract a subset of the buffer
    pub fn slice(&self, start: usize, end: usize) -> Option<PiBuffer> {
        if start >= end || end > self.used {
            return None;
        }
        let mut buf = Self::new(end - start);
        let _ = buf.append(&self.as_slice()[start..end]);
        Some(buf)
    }

    /// Truncate the buffer
    pub fn truncate(&mut self, new_len: usize) {
        if new_len < self.used {
            self.used = new_len;
        }
    }
}

impl Drop for PiBuffer {
    fn drop(&mut self) {
        self.reset();
    }
}

impl Default for PiBuffer {
    fn default() -> Self {
        Self::new(DEFAULT_BUFFER_CAPACITY)
    }
}

impl AsRef<[u8]> for PiBuffer {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsMut<[u8]> for PiBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl PartialEq for PiBuffer {
    fn eq(&self, other: &PiBuffer) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for PiBuffer {}

impl PartialEq<[u8]> for PiBuffer {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}

impl From<&[u8]> for PiBuffer {
    fn from(slice: &[u8]) -> Self {
        Self::from_bytes(slice)
    }
}

impl From<&str> for PiBuffer {
    fn from(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }
}

impl From<String> for PiBuffer {
    fn from(s: String) -> Self {
        Self::from_bytes(s.as_bytes())
    }
}

impl std::ops::Deref for PiBuffer {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl std::ops::DerefMut for PiBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl std::ops::Index<std::ops::Range<usize>> for PiBuffer {
    type Output = [u8];
    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl std::ops::Index<std::ops::RangeFull> for PiBuffer {
    type Output = [u8];
    fn index(&self, _: std::ops::RangeFull) -> &Self::Output {
        self.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buf = PiBuffer::new(100);
        assert!(buf.capacity() >= 100);
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_from_bytes() {
        let buf = PiBuffer::from_bytes(b"test data");
        assert_eq!(buf.len(), 9);
    }

    #[test]
    fn test_append() {
        let mut buf = PiBuffer::new(16);
        assert!(buf.append(b"hello").is_ok());
        assert_eq!(buf.len(), 5);
        
        assert!(buf.append(b" world").is_ok());
        assert_eq!(buf.len(), 11);
    }

    #[test]
    fn test_clear() {
        let mut buf = PiBuffer::from_bytes(b"hello");
        buf.clear();
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_slice() {
        let buf = PiBuffer::from_bytes(b"hello world");
        let slice = buf.slice(0, 5);
        assert!(slice.is_some());
        assert_eq!(slice.unwrap().as_slice(), b"hello");
    }

    #[test]
    fn test_clone_is_deep_copy() {
        let buf = PiBuffer::from_bytes(b"original");
        let cloned = buf.clone();
        // Both should have same content
        assert_eq!(buf.as_slice(), cloned.as_slice());
        // After dropping the original, clone must still be valid
        drop(buf);
        assert_eq!(cloned.as_slice(), b"original");
    }
}
