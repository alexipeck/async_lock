use crate::{mutex::Mutex, rwlock::RwLock};
use std::{
    collections::BTreeSet,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{join, sync::Notify, task::JoinHandle};

pub async fn tokio_mutex() {
    let data: BTreeSet<u8> = (0..255).collect();
    let locked_data = Arc::new(tokio::sync::Mutex::new(data.to_owned()));

    let notify_start: Arc<Notify> = Arc::new(Notify::new());
    let handles: Vec<JoinHandle<()>> = (0..=u8::MAX as usize)
        .step_by(256 / 4)
        .map(|i| (i as u8..=(i + 63) as u8).collect::<BTreeSet<u8>>())
        .collect::<Vec<BTreeSet<u8>>>()
        .into_iter()
        .map(|data| {
            let locked_data = locked_data.to_owned();
            let notify_start = notify_start.to_owned();
            tokio::spawn(async move {
                notify_start.notified().await;
                for i in data {
                    let mut guard = locked_data.lock().await;
                    guard.remove(&i);
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();
    tokio::time::sleep(Duration::from_millis(50)).await;
    notify_start.notify_waiters();
    for handle in handles {
        if let Err(err) = join!(handle).0 {
            eprintln!("{err}");
        }
    }
    assert_eq!(*locked_data.lock().await, BTreeSet::new());
}

pub async fn my_mutex() {
    let data: BTreeSet<u8> = (0..255).collect();
    let locked_data = Arc::new(Mutex::new(data.to_owned()));

    let notify_start: Arc<Notify> = Arc::new(Notify::new());
    let handles: Vec<JoinHandle<()>> = (0..=u8::MAX as usize)
        .step_by(256 / 4)
        .map(|i| (i as u8..=(i + 63) as u8).collect::<BTreeSet<u8>>())
        .collect::<Vec<BTreeSet<u8>>>()
        .into_iter()
        .map(|data| {
            let locked_data = locked_data.to_owned();
            let notify_start = notify_start.to_owned();
            tokio::spawn(async move {
                notify_start.notified().await;
                for i in data {
                    let mut guard = locked_data.lock().await;
                    guard.remove(&i);
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();
    tokio::time::sleep(Duration::from_millis(50)).await;
    notify_start.notify_waiters();
    for handle in handles {
        if let Err(err) = join!(handle).0 {
            eprintln!("{err}");
        }
    }
    assert_eq!(*locked_data.lock().await, BTreeSet::new());
}

pub async fn my_rwlock() {
    let data: BTreeSet<u8> = (0..255).collect();
    let locked_data = RwLock::new(data.to_owned());
    //println!("{:?}", *locked_data.read().await);
    let counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    let notify_start: Arc<Notify> = Arc::new(Notify::new());
    let mut handles: Vec<JoinHandle<()>> = (0..=u8::MAX as usize)
        .step_by(256 / 4)
        .map(|i| (i as u8..=(i + 63) as u8).collect::<BTreeSet<u8>>())
        .collect::<Vec<BTreeSet<u8>>>()
        .into_iter()
        .map(|data| {
            let locked_data = locked_data.to_owned();
            let notify_start = notify_start.to_owned();
            let counter: Arc<AtomicUsize> = counter.to_owned();
            tokio::spawn(async move {
                notify_start.notified().await;
                for i in data {
                    locked_data.write().await.remove(&i);
                    counter.fetch_add(1, Ordering::SeqCst);
                }
                //println!("Write thread finished work.");
            })
        })
        .collect::<Vec<JoinHandle<()>>>();
    for _ in 0..4 {
        handles.push({
            let locked_data = locked_data.to_owned();
            let notify_start = notify_start.to_owned();
            let counter: Arc<AtomicUsize> = counter.to_owned();
            tokio::spawn(async move {
                notify_start.notified().await;
                for _ in 0..100 {
                    let guard = locked_data.read().await;
                    //println!("Locked");
                    let _ = guard.len();
                    counter.fetch_add(1, Ordering::SeqCst);
                }
                //println!("Read thread finished work.");
            })
        });
    }
    tokio::time::sleep(Duration::from_millis(50)).await;
    notify_start.notify_waiters();
    for (i, handle) in handles.into_iter().enumerate() {
        //println!("Waiting on thread {i}");
        if let Err(err) = handle.await {
            eprintln!("{err}");
        } else {
            //println!("{i}: Thread finished work.");
        }
        //println!("{:?}", *locked_data.read().await);
    }
    //println!("Count: {}", counter.load(Ordering::SeqCst));
    //println!("{:?}", *locked_data.read().await);
    assert_eq!(*locked_data.read().await, BTreeSet::new());
}

pub async fn my_rwlock_v2() {
    let data: BTreeSet<u8> = (0..255).collect();
    let locked_data = crate::rwlock_v2::RwLock::new(data.to_owned());
    //println!("{:?}", *locked_data.read().await);
    let counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    let notify_start: Arc<Notify> = Arc::new(Notify::new());
    let mut handles: Vec<JoinHandle<()>> = (0..=u8::MAX as usize)
        .step_by(256 / 4)
        .map(|i| (i as u8..=(i + 63) as u8).collect::<BTreeSet<u8>>())
        .collect::<Vec<BTreeSet<u8>>>()
        .into_iter()
        .map(|data| {
            let locked_data = locked_data.to_owned();
            let notify_start = notify_start.to_owned();
            let counter: Arc<AtomicUsize> = counter.to_owned();
            tokio::spawn(async move {
                notify_start.notified().await;
                for i in data {
                    locked_data.write().await.remove(&i);
                    counter.fetch_add(1, Ordering::SeqCst);
                }
                //println!("Write thread finished work.");
            })
        })
        .collect::<Vec<JoinHandle<()>>>();
    for _ in 0..4 {
        handles.push({
            let locked_data = locked_data.to_owned();
            let notify_start = notify_start.to_owned();
            let counter: Arc<AtomicUsize> = counter.to_owned();
            tokio::spawn(async move {
                notify_start.notified().await;
                for _ in 0..100 {
                    let guard = locked_data.read().await;
                    //println!("Locked");
                    let _ = guard.len();
                    counter.fetch_add(1, Ordering::SeqCst);
                }
                //println!("Read thread finished work.");
            })
        });
    }
    tokio::time::sleep(Duration::from_millis(50)).await;
    notify_start.notify_waiters();
    for (i, handle) in handles.into_iter().enumerate() {
        //println!("Waiting on thread {i}");
        if let Err(err) = handle.await {
            eprintln!("{err}");
        } else {
            //println!("{i}: Thread finished work.");
        }
        //println!("{:?}", *locked_data.read().await);
    }
    //println!("Count: {}", counter.load(Ordering::SeqCst));
    //println!("{:?}", *locked_data.read().await);
    assert_eq!(*locked_data.read().await, BTreeSet::new());
}

pub async fn tokio_rwlock() {
    let data: BTreeSet<u8> = (0..255).collect();
    let locked_data = Arc::new(tokio::sync::RwLock::new(data.to_owned()));
    //println!("{:?}", *locked_data.read().await);
    let counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    let notify_start: Arc<Notify> = Arc::new(Notify::new());
    let mut handles: Vec<JoinHandle<()>> = (0..=u8::MAX as usize)
        .step_by(256 / 4)
        .map(|i| (i as u8..=(i + 63) as u8).collect::<BTreeSet<u8>>())
        .collect::<Vec<BTreeSet<u8>>>()
        .into_iter()
        .map(|data| {
            let locked_data = locked_data.to_owned();
            let notify_start = notify_start.to_owned();
            let counter: Arc<AtomicUsize> = counter.to_owned();
            tokio::spawn(async move {
                notify_start.notified().await;
                for i in data {
                    locked_data.write().await.remove(&i);
                    counter.fetch_add(1, Ordering::SeqCst);
                }
                //println!("Write thread finished work.");
            })
        })
        .collect::<Vec<JoinHandle<()>>>();
    for _ in 0..4 {
        handles.push({
            let locked_data = locked_data.to_owned();
            let notify_start = notify_start.to_owned();
            let counter: Arc<AtomicUsize> = counter.to_owned();
            tokio::spawn(async move {
                notify_start.notified().await;
                for _ in 0..100 {
                    let guard = locked_data.read().await;
                    //println!("Locked");
                    let _ = guard.len();
                    counter.fetch_add(1, Ordering::SeqCst);
                }
                //println!("Read thread finished work.");
            })
        });
    }
    tokio::time::sleep(Duration::from_millis(50)).await;
    notify_start.notify_waiters();
    for (i, handle) in handles.into_iter().enumerate() {
        //println!("Waiting on thread {i}");
        if let Err(err) = handle.await {
            eprintln!("{err}");
        } else {
            //println!("{i}: Thread finished work.");
        }
        //println!("{:?}", *locked_data.read().await);
    }
    //println!("Count: {}", counter.load(Ordering::SeqCst));
    //println!("{:?}", *locked_data.read().await);
    assert_eq!(*locked_data.read().await, BTreeSet::new());
}

#[cfg(test)]
pub mod tests {
    use std::time::Instant;

    use crate::tests::{my_mutex, my_rwlock, my_rwlock_v2, tokio_mutex, tokio_rwlock};

    #[tokio::test]
    pub async fn test_tokio_mutex() {
        tokio_mutex().await;
    }

    #[tokio::test]
    pub async fn test_my_mutex() {
        my_mutex().await
    }

    #[tokio::test]
    pub async fn test_tokio_rwlock() {
        tokio_rwlock().await
    }

    #[tokio::test]
    pub async fn test_my_rwlock() {
        my_rwlock().await
    }

    #[tokio::test]
    pub async fn test_my_rwlock_v2() {
        my_rwlock_v2().await
    }

    /* #[tokio::test]
    pub async fn test_benchmark_my_mutex() {
        let instant: Instant = Instant::now();
        for _ in 0..100 {
            my_mutex().await;
        }
        println!("100 runs took {}ms", instant.elapsed().as_millis());
    }

    #[tokio::test]
    pub async fn test_benchmark_tokio_mutex() {
        let instant: Instant = Instant::now();
        for _ in 0..100 {
            tokio_mutex().await;
        }
        println!("100 runs took {}ms", instant.elapsed().as_millis());
    } */

    #[tokio::test]
    pub async fn test_benchmark_tokio_rwlock() {
        let instant: Instant = Instant::now();
        for _ in 0..100 {
            tokio_rwlock().await;
        }
        println!("100 runs took {}ms", instant.elapsed().as_millis());
    }

    #[tokio::test]
    pub async fn test_benchmark_my_rwlock() {
        let instant: Instant = Instant::now();
        for _ in 0..100 {
            my_rwlock().await;
        }
        println!("100 runs took {}ms", instant.elapsed().as_millis());
    }

    #[tokio::test]
    pub async fn test_benchmark_my_rwlock_v2() {
        let instant: Instant = Instant::now();
        for _ in 0..100 {
            my_rwlock_v2().await;
        }
        println!("100 runs took {}ms", instant.elapsed().as_millis());
    }
}
