use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::{Mutex, RwLock};

pub type CachedGetter<T, E> = fn() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;

pub struct Cached<T, E>
where
    T: Clone,
{
    getter: CachedGetter<T, E>,
    last_update: Arc<Mutex<Instant>>,
    expiry: Duration,
    value: Arc<RwLock<T>>,
}

impl<T: Clone, E> Clone for Cached<T, E> {
    fn clone(&self) -> Self {
        Self {
            getter: self.getter,
            last_update: self.last_update.clone(),
            expiry: self.expiry,
            value: self.value.clone(),
        }
    }
}

impl<T: Clone, E> Cached<T, E> {
    pub async fn new(getter: CachedGetter<T, E>, expiry: Duration) -> Result<Self, E> {
        let last_update = Arc::new(Mutex::new(Instant::now()));
        let value = Arc::new(RwLock::new(getter().await?));
        Ok(Self {
            getter,
            last_update,
            expiry,
            value,
        })
    }

    pub async fn get(&self) -> Result<T, E> {
        self.get_verbose().await.map(|(value, _)| value)
    }

    pub async fn get_verbose(&self) -> Result<(T, bool), E> {
        let mut last_update = self.last_update.lock().await;
        if last_update.elapsed() > self.expiry {
            let mut value = self.value.write().await;
            *last_update = Instant::now();
            *value = (self.getter)().await?;
            Ok((value.clone(), true))
        } else {
            Ok((self.value.read().await.clone(), false))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_42() -> Pin<Box<dyn Future<Output = Result<i32, ()>> + Send>> {
        Box::pin(async { Ok(42) })
    }

    #[tokio::test]
    async fn cached_stress_test_1sec() {
        let cached: Cached<i32, ()> = Cached::new(get_42, Duration::from_micros(100))
            .await
            .unwrap();

        let start_time = Instant::now();

        let mut tasks = Vec::new();

        for _ in 0..100 {
            let cached = cached.clone();
            tasks.push(tokio::spawn(async move {
                let mut count_updates: i32 = 0;
                let mut count_no_updates: i32 = 0;
                while start_time.elapsed() < Duration::from_secs(1) {
                    let (value, updated) = cached.get_verbose().await.unwrap();
                    assert_eq!(value, 42);
                    if updated {
                        count_updates += 1;
                    } else {
                        count_no_updates += 1;
                    }
                }
                (count_updates, count_no_updates)
            }));
        }

        let mut total_count_updates: i32 = 0;
        let mut total_count_no_updates: i32 = 0;

        for task in tasks {
            let (count_updates, count_no_updates) = task.await.unwrap();
            total_count_updates += count_updates;
            total_count_no_updates += count_no_updates;
        }

        assert!(total_count_updates <= 1000000 / 100);

        println!("Updates: {}", total_count_updates);
        println!("Non-updates: {}", total_count_no_updates);
    }
}
