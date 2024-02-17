use crate::mutex::Mutex;
use std::{collections::BTreeSet, sync::Arc, time::Duration};
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
    let locked_data = Arc::new(/* tokio::sync:: */ Mutex::new(data.to_owned()));

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

#[cfg(test)]
pub mod tests {
    use crate::tests::{my_mutex, tokio_mutex};

    #[tokio::test]
    pub async fn test_tokio_mutex() {
        tokio_mutex().await;
    }

    #[tokio::test]
    pub async fn test_my_mutex() {
        my_mutex().await
    }
}
