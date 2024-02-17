use async_lock::tests::{my_mutex, tokio_mutex};
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    let instant: Instant = Instant::now();
    for _ in 0..100 {
        my_mutex().await;
    }
    println!("Mine: {}", instant.elapsed().as_millis());

    let instant: Instant = Instant::now();
    for _ in 0..100 {
        tokio_mutex().await;
    }
    println!("Tokio: {}", instant.elapsed().as_millis());
}
