use crate::bucket::Bucket;
use crate::bucket::BUCKET_SIZE;
use crate::util::{get_alt_index, get_indices_and_fingerprint, upper_power2};
use rand::{random, Rng};
use std::cmp::max;
use std::mem;
use std::{iter, result};

// Maximum number of cuckoo kicks before claiming failure
const MAX_CUCKOO_COUNT: usize = 500;

const DE_BRUIJN64_TAB: [usize; 64] = [
    0, 1, 56, 2, 57, 49, 28, 3, 61, 58, 42, 50, 38, 29, 17, 4, 62, 47, 59, 36, 45, 43, 51, 22, 53,
    39, 33, 30, 24, 18, 12, 5, 63, 55, 48, 27, 60, 41, 37, 16, 46, 35, 44, 21, 52, 32, 23, 11, 54,
    26, 40, 15, 34, 20, 31, 10, 25, 14, 19, 9, 13, 8, 7, 6,
];
const DE_BRUIJN64: u64 = 0x03f79d71b4ca8b09;

pub type CResult<E> = result::Result<(), E>;

#[allow(clippy::enum_variant_names)]
pub enum CuckooError {
    NotFound,
    NotEnoughSpace,
    NotSupported,
}

pub struct CuckooFilter {
    buckets: Box<[Bucket]>,
    size: usize,
    pow: usize,
}

fn gen_size(max_num_keys: u64) -> u64 {
    let mut num_buckets = upper_power2(max(1, max_num_keys / BUCKET_SIZE as u64));
    let frac = max_num_keys as f64 / num_buckets as f64 / BUCKET_SIZE as f64;
    if frac > 0.96 {
        num_buckets <<= 1;
    }
    num_buckets
}

impl CuckooFilter {
    /// # Example
    /// ```
    /// use dakv_cuckoo::CuckooFilter;
    /// let cuckoo = CuckooFilter::new(100);
    /// ```
    pub fn new(max_num_keys: u64) -> Self {
        Self::with_capacity(gen_size(max_num_keys) as usize)
    }

    /// # Example
    /// ```
    /// use dakv_cuckoo::CuckooFilter;
    /// let cuckoo = CuckooFilter::with_capacity(100);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        let buck = iter::repeat(Bucket::new())
            .take(capacity)
            .collect::<Vec<_>>();
        CuckooFilter {
            size: 0,
            buckets: buck.into_boxed_slice(),
            pow: trailing_zeros(capacity),
        }
    }

    /// # Example
    /// ```
    /// use dakv_cuckoo::CuckooFilter;
    ///
    /// let mut cf = CuckooFilter::default();
    /// cf.add(b"test");
    /// ```
    pub fn add(&mut self, item: &[u8]) -> CResult<CuckooError> {
        let finger = get_indices_and_fingerprint(item, self.pow);
        if self.insert(finger.fp, finger.i1) || self.insert(finger.fp, finger.i2) {
            return Ok(());
        }
        self.reinsert(finger.fp, rand_index(finger.i1, finger.i2))
    }

    fn insert(&mut self, fp: u8, i: u64) -> bool {
        let index = i as usize % self.buckets.len();
        if self.buckets[index].insert(fp) {
            self.size += 1;
            true
        } else {
            false
        }
    }

    fn reinsert(&mut self, mut fp: u8, mut i: u64) -> CResult<CuckooError> {
        let mut rng = rand::thread_rng();
        for _ in 0..MAX_CUCKOO_COUNT {
            let j = rng.gen_range(0, BUCKET_SIZE);
            mem::swap(&mut fp, &mut self.buckets[i as usize][j]);

            i = get_alt_index(fp, i, self.pow);
            if self.insert(fp, i) {
                return Ok(());
            }
        }
        Err(CuckooError::NotEnoughSpace)
    }

    /// # Example
    /// ```
    /// use dakv_cuckoo::CuckooFilter;
    /// let mut cf = CuckooFilter::default();
    /// cf.add(b"test");
    /// assert!(cf.contains(b"test"));
    /// ```
    pub fn contains(&self, data: &[u8]) -> bool {
        let finger = get_indices_and_fingerprint(data, self.pow);
        let b1 = self.buckets[finger.i1 as usize];
        let b2 = self.buckets[finger.i1 as usize];
        b1.get_fingerprint_index(finger.fp).is_some()
            || b2.get_fingerprint_index(finger.fp).is_some()
    }

    /// # Example
    /// ```
    /// use dakv_cuckoo::CuckooFilter;
    /// let mut cf = CuckooFilter::default();
    /// cf.add(b"test");
    /// assert!(cf.delete(b"test"));
    /// ```
    pub fn delete(&mut self, data: &[u8]) -> bool {
        let finger = get_indices_and_fingerprint(data, self.pow);
        self.remove(finger.fp, finger.i1) || self.remove(finger.fp, finger.i2)
    }

    fn remove(&mut self, fp: u8, i: u64) -> bool {
        if self.buckets[i as usize].delete(fp) {
            self.size -= 1;
            return true;
        }
        false
    }

    /// # Example
    /// ```
    /// use dakv_cuckoo::CuckooFilter;
    /// let cuckoo = CuckooFilter::default();
    ///
    /// println!("size: {}", cuckoo.size());
    /// ```
    pub fn size(&self) -> usize {
        self.size
    }
}

impl Default for CuckooFilter {
    fn default() -> Self {
        // About 16 million
        CuckooFilter::new(1 << 24)
    }
}

fn rand_index(i1: u64, i2: u64) -> u64 {
    if random() {
        i1
    } else {
        i2
    }
}

fn trailing_zeros(c: usize) -> usize {
    if c == 0 {
        return 64;
    }
    let cc = (c as u64 & (c as i64 * (-1)) as u64).wrapping_mul(DE_BRUIJN64);
    DE_BRUIJN64_TAB[(cc as usize).wrapping_shr(64 - 6)]
}

#[cfg(test)]
mod tests {
    use crate::cuckoo_filter::{gen_size, trailing_zeros};
    use crate::CuckooFilter;

    #[test]
    fn test_trailing_zeros() {
        assert_eq!(trailing_zeros(1 << 24), 24);
        assert_eq!(trailing_zeros(100), 2);
        assert_eq!(trailing_zeros(64), 6);
        assert_eq!(trailing_zeros(0), 64);
        assert_eq!(trailing_zeros(1), 0);
        assert_eq!(trailing_zeros(3), 0);
        assert_eq!(trailing_zeros(8), 3);
    }

    #[test]
    fn test_gen_size() {
        assert_eq!(gen_size(100), 32);
        assert_eq!(gen_size(64), 32);
    }

    #[test]
    fn test_add() {
        let mut cf = CuckooFilter::new(100);
        for _ in 0..8 {
            let result = cf.add(b"test");
            assert!(result.is_ok());
        }
        assert_eq!(cf.size(), 8);
        for _ in 0..8 {
            let result = cf.add(b"test");
            assert!(!result.is_ok());
        }
        assert_eq!(cf.size(), 8);
    }

    #[test]
    fn test_delete() {
        let mut cf = CuckooFilter::default();
        let _ = cf.add(b"test");
        assert_eq!(cf.size(), 1);
        assert!(cf.contains(b"test"));
        assert!(cf.delete(b"test"));
        assert_eq!(cf.size(), 0);
        assert!(!cf.contains(b"test"));
    }
}
