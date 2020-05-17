use std::ops;

pub const BUCKET_SIZE: usize = 4;

#[derive(Default, Copy, Clone)]
pub struct Bucket {
    data: [u8; BUCKET_SIZE],
}

impl Bucket {
    pub fn new() -> Self {
        Bucket {
            data: [0; BUCKET_SIZE],
        }
    }

    pub fn insert(&mut self, finger: u8) -> bool {
        for fp in self.data.iter_mut() {
            if *fp == 0 {
                *fp = finger;
                return true;
            }
        }
        false
    }

    pub fn delete(&mut self, finger: u8) -> bool {
        for fp in self.data.iter_mut() {
            if *fp == finger {
                *fp = 0;
                return true;
            }
        }
        false
    }

    pub fn get_fingerprint_index(self, finger: u8) -> Option<usize> {
        for (i, fp) in self.data.iter().enumerate() {
            if *fp == finger {
                return Some(i);
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        for fp in self.data.iter_mut() {
            *fp = 0;
        }
    }
}

impl ops::Index<usize> for Bucket {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl ops::IndexMut<usize> for Bucket {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}
