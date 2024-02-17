/* use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
};

use tokio::sync::Notify;

#[derive(Debug)]
pub struct WriteGuard<'a, T> {
    data: &'a mut T,
}

impl<'a, T> WriteGuard<'a, T> {
    pub fn new(data: &'a mut T) -> Self {
        Self { data }
    }
}

impl<'a, T> Deref for WriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> DerefMut for WriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Debug, Clone)]
pub struct ReadGuard<'a, T> {
    data: &'a T,
}

impl<'a, T> ReadGuard<'a, T> {
    pub fn new(data: &'a T) -> Self {
        Self { data }
    }
}

impl<'a, T> Deref for ReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub enum K {
    Writer(usize),
    Readers(HashSet<usize>),
}

pub struct RwLock<T> {
    data: T,
    locked_for_writer: Arc<AtomicBool>,
    await_read: Arc<Notify>,
    ledger: parking_lot::Mutex<Option<K>>,
    next_uid: Arc<AtomicUsize>,
}

impl<T> RwLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            locked_for_writer: Arc::new(AtomicBool::new(false)),
            await_read: Arc::new(Notify::new()),
            ledger: parking_lot::Mutex::new(None),
            next_uid: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub async fn read(&self) -> ReadGuard<T> {
        loop {
            if self.locked_for_writer.load(Ordering::SeqCst) {
                self.await_read.notified().await;
            } else {
                break;
            }
        }

        ReadGuard::new(&self.data)
    }

    pub async fn write(&mut self) -> WriteGuard<T> {
        WriteGuard::new(&mut self.data)
    }
}
 */
