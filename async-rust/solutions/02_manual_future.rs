#![cfg_attr(not(test), allow(dead_code))]

//! # 解答 02: センサー通知を待つ `Future` を手動で実装する

use std::{
    future::Future,
    marker::PhantomPinned,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard},
    task::{Context, Poll, Waker},
};

#[derive(Debug, PartialEq, Eq)]
struct SensorReading {
    robot_id: String,
    millivolts: u32,
}

impl SensorReading {
    fn new(robot_id: &str, millivolts: u32) -> Self {
        Self {
            robot_id: robot_id.to_owned(),
            millivolts,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SensorError {
    Closed,
}

enum SensorState {
    Waiting { waker: Option<Waker> },
    Published(Option<SensorReading>),
    Closed,
    Consumed,
}

struct Shared {
    state: Mutex<SensorState>,
}

impl Shared {
    fn lock_state(&self) -> MutexGuard<'_, SensorState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

struct SensorPublisher {
    shared: Arc<Shared>,
    published: bool,
}

impl SensorPublisher {
    fn publish(mut self, reading: SensorReading) {
        let waker = {
            let mut state = self.shared.lock_state();
            let SensorState::Waiting { waker } = &mut *state else {
                panic!("1回限りのセンサーへ重複した状態遷移が発生しました")
            };
            let waker = waker.take();
            *state = SensorState::Published(Some(reading));
            waker
        };

        self.published = true;
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

impl Drop for SensorPublisher {
    fn drop(&mut self) {
        if self.published {
            return;
        }

        let waker = {
            let mut state = self.shared.lock_state();
            match &mut *state {
                SensorState::Waiting { waker } => {
                    let waker = waker.take();
                    *state = SensorState::Closed;
                    waker
                }
                SensorState::Published(_) | SensorState::Closed | SensorState::Consumed => None,
            }
        };

        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

struct SensorFuture {
    shared: Arc<Shared>,
    _pin: PhantomPinned,
}

impl Future for SensorFuture {
    type Output = Result<SensorReading, SensorError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let shared = &self.as_ref().get_ref().shared;
        let mut state = shared.lock_state();

        match &mut *state {
            SensorState::Waiting { waker } => {
                if waker
                    .as_ref()
                    .is_none_or(|saved| !saved.will_wake(context.waker()))
                {
                    *waker = Some(context.waker().clone());
                }
                Poll::Pending
            }
            SensorState::Published(reading) => {
                let reading = reading
                    .take()
                    .expect("Published 状態には未消費の値が必要です");
                *state = SensorState::Consumed;
                Poll::Ready(Ok(reading))
            }
            SensorState::Closed => {
                *state = SensorState::Consumed;
                Poll::Ready(Err(SensorError::Closed))
            }
            SensorState::Consumed => panic!("完了した SensorFuture を再度 poll しました"),
        }
    }
}

fn sensor_channel() -> (SensorPublisher, SensorFuture) {
    let shared = Arc::new(Shared {
        state: Mutex::new(SensorState::Waiting { waker: None }),
    });

    (
        SensorPublisher {
            shared: Arc::clone(&shared),
            published: false,
        },
        SensorFuture {
            shared,
            _pin: PhantomPinned,
        },
    )
}

fn main() {
    let (publisher, future) = sensor_channel();
    publisher.publish(SensorReading::new("配送ロボット-1002", 3_300));

    let _future = Box::pin(future);
    println!("センサー値を待つ Future を作成しました");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Wake, Waker};

    #[derive(Default)]
    struct WakeCounter {
        count: AtomicUsize,
    }

    impl Wake for WakeCounter {
        fn wake(self: Arc<Self>) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    impl WakeCounter {
        fn count(&self) -> usize {
            self.count.load(Ordering::SeqCst)
        }
    }

    fn poll_once(
        future: Pin<&mut SensorFuture>,
        counter: &Arc<WakeCounter>,
    ) -> Poll<Result<SensorReading, SensorError>> {
        let waker = Waker::from(Arc::clone(counter));
        let mut context = Context::from_waker(&waker);
        Future::poll(future, &mut context)
    }

    #[test]
    fn pollより先に発行された値をreadyで受け取る() {
        let (publisher, future) = sensor_channel();
        let reading = SensorReading::new("R-before", 1_234);
        let robot_id_pointer = reading.robot_id.as_ptr();

        publisher.publish(reading);

        let counter = Arc::new(WakeCounter::default());
        let mut future = Box::pin(future);
        let result = poll_once(future.as_mut(), &counter);
        let Poll::Ready(Ok(reading)) = result else {
            panic!("発行済みの値は Ready(Ok(_)) になる必要があります")
        };

        assert_eq!(reading, SensorReading::new("R-before", 1_234));
        assert_eq!(reading.robot_id.as_ptr(), robot_id_pointer);
        assert_eq!(counter.count(), 0);
    }

    #[test]
    fn 未発行ならpendingになり発行時に一度だけwakeする() {
        let (publisher, future) = sensor_channel();
        let counter = Arc::new(WakeCounter::default());
        let mut future = Box::pin(future);

        assert!(poll_once(future.as_mut(), &counter).is_pending());
        assert_eq!(counter.count(), 0);

        publisher.publish(SensorReading::new("R-wake", 2_468));

        assert_eq!(counter.count(), 1);
        assert_eq!(
            poll_once(future.as_mut(), &counter),
            Poll::Ready(Ok(SensorReading::new("R-wake", 2_468)))
        );
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn 同じwakerで複数回pollしても発行時のwakeは一度だけ() {
        let (publisher, future) = sensor_channel();
        let counter = Arc::new(WakeCounter::default());
        let mut future = Box::pin(future);

        assert!(poll_once(future.as_mut(), &counter).is_pending());
        assert!(poll_once(future.as_mut(), &counter).is_pending());
        assert!(poll_once(future.as_mut(), &counter).is_pending());

        publisher.publish(SensorReading::new("R-repeat", 77));

        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn 別taskからpollされたら最新のwakerだけを起こす() {
        let (publisher, future) = sensor_channel();
        let first = Arc::new(WakeCounter::default());
        let second = Arc::new(WakeCounter::default());
        let mut future = Box::pin(future);

        assert!(poll_once(future.as_mut(), &first).is_pending());
        assert!(poll_once(future.as_mut(), &second).is_pending());

        publisher.publish(SensorReading::new("R-replaced", 88));

        assert_eq!(first.count(), 0);
        assert_eq!(second.count(), 1);
    }

    #[test]
    fn 待機中にpublisherをdropするとwakeしてclosedを返す() {
        let (publisher, future) = sensor_channel();
        let counter = Arc::new(WakeCounter::default());
        let mut future = Box::pin(future);

        assert!(poll_once(future.as_mut(), &counter).is_pending());
        drop(publisher);

        assert_eq!(counter.count(), 1);
        assert_eq!(
            poll_once(future.as_mut(), &counter),
            Poll::Ready(Err(SensorError::Closed))
        );
    }

    #[test]
    fn poll前にpublisherをdropしてもclosedをreadyで返す() {
        let (publisher, future) = sensor_channel();
        let counter = Arc::new(WakeCounter::default());
        let mut future = Box::pin(future);

        drop(publisher);

        assert_eq!(
            poll_once(future.as_mut(), &counter),
            Poll::Ready(Err(SensorError::Closed))
        );
        assert_eq!(counter.count(), 0);
    }

    #[test]
    fn utf8のrobot_idを変更しない() {
        let (publisher, future) = sensor_channel();
        let counter = Arc::new(WakeCounter::default());
        let mut future = Box::pin(future);

        assert!(poll_once(future.as_mut(), &counter).is_pending());
        publisher.publish(SensorReading::new("配送ロボット🤖-十号", 9_001));

        assert_eq!(
            poll_once(future.as_mut(), &counter),
            Poll::Ready(Ok(SensorReading::new("配送ロボット🤖-十号", 9_001)))
        );
    }
}
