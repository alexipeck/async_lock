use crate::mutex::Mutex;
use std::{
    cell::UnsafeCell,
    collections::VecDeque,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::sync::{
    mpsc::{self, Sender, UnboundedSender},
    Notify,
};

pub struct WriteGuard<'a, T: Send + Sync> {
    data: &'a RwLock<T>,
    inner: Arc<RwLockInner<T>>,
}
unsafe impl<T: Send + Sync> Send for WriteGuard<'_, T> {}
unsafe impl<T: Send + Sync> Sync for WriteGuard<'_, T> {}

impl<'a, T: Send + Sync> WriteGuard<'a, T> {
    pub fn new(data: &'a RwLock<T>, inner: Arc<RwLockInner<T>>) -> Self {
        Self { data, inner }
    }
}

impl<'a, T: Send + Sync> Drop for WriteGuard<'a, T> {
    fn drop(&mut self) {
        self.inner
            .runtime_data
            .async_drop_tx
            .send(())
            .expect("Failed to send tick.");
    }
}

impl<'a, T: Send + Sync> Deref for WriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.inner.data.get() }
    }
}

impl<'a, T: Send + Sync> DerefMut for WriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data.inner.data.get() }
    }
}

pub struct ReadGuard<'a, T: Send + Sync> {
    data: &'a RwLock<T>,
    inner: Arc<RwLockInner<T>>,
}
unsafe impl<T: Send + Sync> Send for ReadGuard<'_, T> {}
unsafe impl<T: Send + Sync> Sync for ReadGuard<'_, T> {}

impl<'a, T: Send + Sync> ReadGuard<'a, T> {
    pub fn new(data: &'a RwLock<T>, inner: Arc<RwLockInner<T>>) -> Self {
        Self { data, inner }
    }
}

impl<'a, T: Send + Sync> Drop for ReadGuard<'a, T> {
    fn drop(&mut self) {
        self.inner
            .runtime_data
            .async_drop_tx
            .send(())
            .expect("Failed to send tick.");
    }
}

impl<'a, T: Send + Sync> Deref for ReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.inner.data.get() }
    }
}

#[derive(Debug)]
enum Current {
    None,
    Read(Arc<Notify>),
    Write(Arc<Notify>),
}

#[derive(Debug)]
struct LockGroup {
    is_writer: bool,
    notify: Arc<Notify>,
}

impl LockGroup {
    pub fn new(is_writer: bool, notify: Arc<Notify>) -> Self {
        Self { is_writer, notify }
    }
}

#[derive(Debug)]
pub struct RwLockRuntimeData {
    current: Arc<Mutex<Current>>,
    queue: Arc<Mutex<VecDeque<LockGroup>>>,
    async_drop_tx: UnboundedSender<()>,
    stop_tx: Sender<()>,
}

impl RwLockRuntimeData {
    pub fn new(async_drop_tx: UnboundedSender<()>, stop_tx: Sender<()>) -> Self {
        Self {
            current: Arc::new(Mutex::new(Current::None)),
            queue: Arc::new(Mutex::new(VecDeque::new())),
            async_drop_tx,
            stop_tx,
        }
    }
    async fn tick(&self) {
        let mut mutex_guard = self.current.lock().await;
        match mutex_guard.deref() {
            Current::Write(notify) => {
                if Arc::strong_count(&notify) != 1 {
                    notify.notify_one();
                    return;
                }
            }
            Current::Read(notify) => {
                if Arc::strong_count(&notify) != 1 {
                    return;
                }
            }
            _ => return,
        }
        if let Some(lock_group) = self.queue.lock().await.pop_front() {
            *mutex_guard.deref_mut() = if lock_group.is_writer {
                lock_group.notify.notify_one();
                Current::Write(lock_group.notify)
            } else {
                lock_group.notify.notify_waiters();
                Current::Read(lock_group.notify)
            }
        } else {
            *mutex_guard.deref_mut() = Current::None;
        }
    }
}

pub struct RwLockInner<T: Send + Sync> {
    data: UnsafeCell<T>,
    runtime_data: Arc<RwLockRuntimeData>,
}
unsafe impl<T: Send + Sync> Send for RwLockInner<T> {}
unsafe impl<T: Send + Sync> Sync for RwLockInner<T> {}

impl<'a, T: Send + Sync> Drop for RwLockInner<T> {
    fn drop(&mut self) {
        let stop_tx = self.runtime_data.stop_tx.clone();
        tokio::spawn(async move {
            if let Err(err) = stop_tx.send(()).await {
                panic!("Error sending stop signal: {err}");
            }
        });
    }
}

//uses internal Arc<> to allow cloning safely.
#[derive(Clone)]
pub struct RwLock<T: Send + Sync> {
    inner: Arc<RwLockInner<T>>,
}

unsafe impl<T: Send + Sync> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T: Send + Sync> RwLock<T> {
    pub fn new(data: T) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<()>();
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
        let inner = Arc::new(RwLockInner {
            data: UnsafeCell::new(data),
            runtime_data: Arc::new(RwLockRuntimeData::new(tx, stop_tx)),
        });
        let _ = tokio::spawn({
            let runtime_data = inner.runtime_data.clone();
            async move {
                loop {
                    tokio::select! {
                        _ = rx.recv() => runtime_data.tick().await,
                        _ = stop_rx.recv() => break,
                    }
                }
            }
        });
        Self { inner }
    }

    async fn join_queue(&self, is_writer: bool) -> Arc<Notify> {
        let mut mutex_guard = self.inner.runtime_data.queue.lock().await;
        if let Some(lock_group) = mutex_guard.iter().last() {
            if is_writer && lock_group.is_writer {
                return lock_group.notify.clone();
            }
        }
        let notify = Arc::new(Notify::new());
        mutex_guard.push_back(LockGroup::new(is_writer, notify.clone()));
        notify
    }

    #[inline]
    pub async fn write(&self) -> WriteGuard<T> {
        let mut mutex_guard = self.inner.runtime_data.current.lock().await;
        let none = matches!(mutex_guard.deref(), Current::None);
        if none {
            let notify: Arc<Notify> = Arc::new(Notify::new());
            *mutex_guard = Current::Write(notify.to_owned());
            drop(mutex_guard);
            notify.notified().await;
        } else {
            drop(mutex_guard);
            let t = self.join_queue(false).await;
            t.notified().await;
        }
        WriteGuard::new(self, self.inner.clone())
    }

    #[inline]
    pub async fn read(&self) -> ReadGuard<T> {
        let mut mutex_guard = self.inner.runtime_data.current.lock().await;
        let none = matches!(mutex_guard.deref(), Current::None);
        if none {
            let notify: Arc<Notify> = Arc::new(Notify::new());
            *mutex_guard = Current::Read(notify.to_owned());
            drop(mutex_guard);
        } else {
            drop(mutex_guard);
            let t = self.join_queue(false).await;
            t.notified().await;
        }
        ReadGuard::new(self, self.inner.clone())
    }
}
