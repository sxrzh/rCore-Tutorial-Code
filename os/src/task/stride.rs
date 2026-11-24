use core::cmp::Ordering;

use crate::config::{BIG_STRIDE, INITIAL_PRIORITY};

#[derive(Clone, Copy)]
pub struct Stride(pub u64);

#[derive(Clone, Copy)]
pub struct StridePass(u64);

impl Default for Stride {
    fn default() -> Self {
        Self(0)
    }
}
impl Ord for Stride {
    fn cmp(&self, other: &Self) -> Ordering {
        // 134217728
        // 1073741824 
        11728481943552 as u64;
        const THRES: u64 = (BIG_STRIDE as u64 + 1) / 2;
        if other.0 < self.0 {
            // Ordering::Less
            if self.0 - other.0 <= THRES { Ordering::Less }
            else {
                panic!("{} - {} > THRES, not L", self.0, other.0);
                // Ordering::Greater
            } 
        }
        else if other.0 > self.0 {
            // Ordering::Greater
            if other.0 - self.0 <= THRES { Ordering::Greater }
            else {
                panic!("{} - {} > THRES, not G", other.0, self.0);
                // Ordering::Less
            }
        }
        else {
            Ordering::Equal
        }
    }
}
impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for Stride {}

impl Stride {
    pub fn add(&mut self, pass: StridePass) {
        // info!("{} += {} -> {}", self.0, pass.0, self.0 + pass.0);
        self.0 += pass.0
    }
}

impl StridePass {
    pub fn from_prio(prio: usize) -> Self {
        Self(BIG_STRIDE / (prio as u64))
    }
}

impl Default for StridePass {
    fn default() -> Self {
        Self::from_prio(INITIAL_PRIORITY)
    }
}