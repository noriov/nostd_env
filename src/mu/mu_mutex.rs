//
// Micro Mutex - A Mutual Exclusion Primitive using Spin Lock
//

use core::{
    cell::UnsafeCell,
    hint::spin_loop,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};


pub struct MuMutex<T> {
    value: UnsafeCell<T>,
    atomic: AtomicBool,
}

unsafe impl<T: Send> Send for MuMutex<T> {}
unsafe impl<T: Send> Sync for MuMutex<T> {}

impl<T> MuMutex<T> {
    pub const fn new(value: T) -> Self {
	Self {
	    value: UnsafeCell::new(value),
	    atomic: AtomicBool::new(false),
	}
    }

    pub fn lock(&self) -> MuMutexGuard<T> {
	self.spin_lock();
	MuMutexGuard::<T> { locked: self }
    }

    fn spin_lock(&self) {
	while self.atomic.compare_exchange_weak(false,
						true,
						Ordering::Acquire,
						Ordering::Relaxed).is_err() {
	    while self.atomic.load(Ordering::Relaxed) {
		spin_loop();
	    }
	}
    }

    fn spin_unlock(&self) {
	self.atomic.store(false, Ordering::Release);
    }
}


#[must_use = "If not used, immediately unlocked"]
pub struct MuMutexGuard<'a, T> {
    locked: &'a MuMutex<T>,
}

impl<'a, T> Drop for MuMutexGuard<'a, T> {
    fn drop(&mut self) {
	self.locked.spin_unlock();
    }
}

impl<'a, T> Deref for MuMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
	unsafe {
	    &*self.locked.value.get()
	}
    }
}

impl<'a, T> DerefMut for MuMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
	unsafe {
	    &mut *self.locked.value.get()
	}
    }
}
