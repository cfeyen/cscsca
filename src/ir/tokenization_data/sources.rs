use std::ptr::NonNull;



#[derive(Debug, Default, PartialEq, Eq)]
pub struct Sources(Vec<NonNull<str>>);

impl Sources {
    #[inline(always)]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[inline(always)]
    pub fn push(&mut self, source: &str) {
        self.0.push(NonNull::from_ref(source));
    }

    #[inline(always)]
    pub fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0);
    }

    /// Frees all the sources
    /// 
    /// # Safety
    /// All references to the sources must be dropped
    /// #[inline(always)]
    pub unsafe fn free(self) {
        for source in self.0 {
            let ptr = source.as_ptr();
            unsafe { drop(Box::from_raw(ptr)); }
        }
    }
}

// Safety:
// It is safe to share the sources since the pointers are read-only through references
unsafe impl Sync for Sources {}

// Safety:
// It is safe to send sources since there is no shared mutable state
unsafe impl Send for Sources {}