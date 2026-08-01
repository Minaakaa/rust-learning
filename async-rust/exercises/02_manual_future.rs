#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 02: センサー通知を待つ `Future` を手動で実装する
//!
//! 1回だけ値を発行できるセンサーと、その値を非同期に待つ `SensorFuture` を作ります
//! `async fn` が利用する契約を理解するため、`Future::poll` を直接実装してください
//!
//! 仕様:
//! - `sensor_channel` は1組の `SensorPublisher` と `SensorFuture` を返す
//! - 値が未発行なら `poll` は現在の `Waker` を保存して `Poll::Pending` を返す
//! - 同じ task の `Waker` で再度 poll された場合は不要な置き換えをしない
//! - 別の task の `Waker` で再度 poll された場合は最新のものへ置き換える
//! - `publish` は `SensorReading` を複製せず共有状態へ移し、保存済みの task を1回起こす
//! - publisher が未発行のまま drop された場合は待機側を起こし、`SensorError::Closed` を返す
//! - 発行が poll より先でも値を失わない
//! - wake は mutex の lock を解放してから実行する
//! - `SensorFuture` は `PhantomPinned` により `Unpin` ではないため、`Box::pin` などで固定する
//! - `unsafe`、実時間の待機、外部 crate は使わない
//!
//! TODO:
//! - `SensorPublisher::publish` で値を保存し、必要なら待機 task を起こす
//! - `SensorPublisher::drop` で未発行の channel を閉じる
//! - `Future for SensorFuture` の `poll` で `Ready` と `Pending` を返し分ける

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

#[allow(
    dead_code,
    reason = "Future 完成前のスターターでは一部の状態を構築しないため"
)]
enum SensorState {
    Waiting { waker: Option<Waker> },
    Published(Option<SensorReading>),
    Closed,
    Consumed,
}

#[allow(
    dead_code,
    reason = "Future 完成前のスターターでは共有状態を読み取らないため"
)]
struct Shared {
    state: Mutex<SensorState>,
}

impl Shared {
    #[allow(
        dead_code,
        reason = "Future 完成前のスターターでは lock helper を呼ばないため"
    )]
    fn lock_state(&self) -> MutexGuard<'_, SensorState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[allow(
    dead_code,
    reason = "publish 完成前のスターターでは publisher の状態を読まないため"
)]
struct SensorPublisher {
    shared: Arc<Shared>,
    published: bool,
}

impl SensorPublisher {
    #[allow(
        unused_mut,
        reason = "完成後は発行済みフラグを更新するため self の可変性が必要"
    )]
    fn publish(mut self, reading: SensorReading) {
        let _ = reading;
        todo!("値を共有状態へ移し、保存済みの Waker を lock の外で起こしてください")
    }
}

impl Drop for SensorPublisher {
    fn drop(&mut self) {
        // TODO: 未発行なら Closed へ遷移し、保存済みの Waker を lock の外で起こしてください
    }
}

#[allow(
    dead_code,
    reason = "poll 完成前のスターターでは共有状態を読み取らないため"
)]
struct SensorFuture {
    shared: Arc<Shared>,
    _pin: PhantomPinned,
}

impl Future for SensorFuture {
    type Output = Result<SensorReading, SensorError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let _ = (self, context);
        todo!("共有状態を確認し、値・切断・待機を Poll で表してください")
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
