use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::{sync::Notify, time::Instant};

pub struct MutexGuard<'a, T> {
    data: &'a Mutex<T>,
    notify_next: Arc<Notify>,
}
unsafe impl<T> Sync for MutexGuard<'_, T> where T: Send + Sync {}

impl<'a, T> MutexGuard<'a, T> {
    pub fn new(data: &'a Mutex<T>, notify_next: Arc<Notify>) -> Self {
        Self { data, notify_next }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.notify_next.notify_one();
    }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.data.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data.data.get() }
    }
}

pub struct Mutex<T> {
    data: UnsafeCell<T>,
    await_access: Arc<Notify>,
}
unsafe impl<T> Send for Mutex<T> where T: Send {}
unsafe impl<T> Sync for Mutex<T> where T: Send {}

impl<T> Mutex<T> {
    pub fn new(data: T) -> Self {
        let await_access: Arc<Notify> = Arc::new(Notify::new());
        await_access.notify_one();
        Self {
            data: UnsafeCell::new(data),
            await_access,
        }
    }

    #[inline]
    pub async fn lock(&self) -> MutexGuard<T> {
        let instant: Instant = Instant::now();
        self.await_access.notified().await;
        let time_taken = instant.elapsed().as_millis();
        if time_taken > 0 {
            println!("{time_taken}");
        }
        MutexGuard::new(self, self.await_access.to_owned())
    }
}
