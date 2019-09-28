use core::cell::UnsafeCell;

// TODO
// - no real locking going on here, just a place holder
// - named single core to remind me to fix it later
pub struct SingleCoreLock<T> {
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for SingleCoreLock<T> {}

impl<T> SingleCoreLock<T> {
    pub const fn new(data: T) -> SingleCoreLock<T> {
        SingleCoreLock {
            data: UnsafeCell::new(data),
        }
    }
}

impl<T> SingleCoreLock<T> {
    pub fn lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        // In a real lock, there would be code around this line that ensures
        // that this mutable reference will ever only be given out one at a
        // time.
        f(unsafe { &mut *self.data.get() })
    }
}
