//! Hasher for unique values
pub struct UniqueHasher {
    result: u64,
}

impl UniqueHasher {
    pub const fn new() -> Self {
        Self {
            result: 0,
        }
    }

    #[inline]
    pub fn add(&mut self, val: u64) {
        self.result = self.result.wrapping_add(val);
    }
}

impl core::hash::Hasher for UniqueHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.result
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.add((*byte).into());
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add(i.into());
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add(i.into());
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add(i.into());
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add(i.into());
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add(i as u64)
    }
}

pub struct UniqueHasherBuilder;

impl core::hash::BuildHasher for UniqueHasherBuilder {
    type Hasher = UniqueHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        UniqueHasher::new()
    }
}
