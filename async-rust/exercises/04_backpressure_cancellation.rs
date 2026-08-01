#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 04: backpressure とキャンセルを安全に扱う
//!
//! 遠隔測定の流入が処理速度を上回ったとき、bounded channel の空きを待って
//! producer 側へ backpressure を伝えます
//! timeout や task のキャンセルが起きても、未送信データと処理中件数を失わない
//! API を完成させてください
//!
//! 仕様:
//! - `submit_with_timeout` は `Sender::reserve` で送信枠を先に予約する
//! - 制限時間内に予約できた場合だけ `Permit::send` で値を送る
//! - timeout では未送信の `Telemetry` を `SubmitError::TimedOut` で返す
//! - receiver が閉じていた場合も `SubmitError::Closed` で元の値を返す
//! - `InFlight::enter` は処理中件数を1増やす RAII guard を返す
//! - guard は通常終了でも task のキャンセルでも drop 時に件数を1減らす
//! - 処理中件数の mutex guard は `.await` をまたいで保持しない
//! - `unsafe`、実時間の待機、unbounded channel は使わない
//!
//! ヒント:
//! - `timeout(wait, sender.reserve())` は timeout と channel close を区別できる
//! - 値は予約が成功するまで `send` future の外側に保持する
//! - `JoinHandle::abort` は task を次の yield point で停止し、future 内の値を drop する
//! - `InFlightGuard` 自体は mutex guard ではなく、共有カウンタの `Arc` だけを所有する

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
#[allow(
    unused_imports,
    reason = "submit_with_timeout 完成前のスターターでは timeout を使用しないため"
)]
use tokio::time::timeout;

#[derive(Debug, PartialEq, Eq)]
struct Telemetry {
    sequence: u64,
    payload: String,
}

impl Telemetry {
    fn new(sequence: u64, payload: String) -> Self {
        Self { sequence, payload }
    }

    const fn sequence(&self) -> u64 {
        self.sequence
    }

    fn payload(&self) -> &str {
        &self.payload
    }
}

#[derive(Debug, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "submit_with_timeout 完成前のスターターでは error を構築しないため"
)]
enum SubmitError {
    TimedOut(Telemetry),
    Closed(Telemetry),
}

async fn submit_with_timeout(
    sender: &mpsc::Sender<Telemetry>,
    telemetry: Telemetry,
    wait: Duration,
) -> Result<(), SubmitError> {
    let _ = (sender, wait);
    todo!(
        "sequence {} の送信枠を予約し、失敗時は所有値を返してください",
        telemetry.sequence()
    )
}

#[derive(Clone, Debug, Default)]
struct InFlight {
    active: Arc<Mutex<usize>>,
}

impl InFlight {
    fn lock_active(&self) -> MutexGuard<'_, usize> {
        self.active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn enter(&self) -> InFlightGuard {
        todo!("処理中件数を増やし、同じ共有カウンタを持つ guard を返してください")
    }

    fn active(&self) -> usize {
        *self.lock_active()
    }
}

#[derive(Debug)]
struct InFlightGuard {
    #[allow(
        dead_code,
        reason = "Drop 完成前のスターターでは処理中件数を参照しないため"
    )]
    active: Arc<Mutex<usize>>,
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        todo!("通常終了とキャンセルの両方で処理中件数を減らしてください")
    }
}

async fn cancellable_work(
    in_flight: InFlight,
    started: oneshot::Sender<()>,
    release: oneshot::Receiver<()>,
) {
    let _guard = in_flight.enter();
    let _ = started.send(());
    let _ = release.await;
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (sender, mut receiver) = mpsc::channel(1);
    let telemetry = Telemetry::new(1, String::from("温度=24"));

    submit_with_timeout(&sender, telemetry, Duration::from_secs(1))
        .await
        .expect("送信枠を予約できる");

    let accepted = receiver.recv().await.expect("遠隔測定を受信できる");
    println!("{}: {}", accepted.sequence(), accepted.payload());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn telemetry(sequence: u64, payload: &str) -> Telemetry {
        Telemetry::new(sequence, payload.to_owned())
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn 満杯のchannelでは受信側が空きを作るまで待つ() {
        let (sender, mut receiver) = mpsc::channel(1);
        sender
            .send(telemetry(1, "先行"))
            .await
            .expect("最初の値を送信できる");

        let worker_sender = sender.clone();
        let handle = tokio::spawn(async move {
            submit_with_timeout(
                &worker_sender,
                telemetry(2, "待機中"),
                Duration::from_secs(5),
            )
            .await
        });

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(4)).await;
        assert!(!handle.is_finished());

        assert_eq!(receiver.recv().await.unwrap().sequence(), 1);
        handle
            .await
            .expect("送信 task は panic しない")
            .expect("空きができれば送信できる");
        assert_eq!(receiver.recv().await.unwrap().sequence(), 2);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn timeoutでは未送信値のallocationを返す() {
        let (sender, _receiver) = mpsc::channel(1);
        sender
            .send(telemetry(1, "占有"))
            .await
            .expect("channel を満杯にできる");
        let pending = telemetry(2, "返却される遠隔測定");
        let payload_pointer = pending.payload.as_ptr();
        let worker_sender = sender.clone();
        let handle = tokio::spawn(async move {
            submit_with_timeout(&worker_sender, pending, Duration::from_secs(3)).await
        });

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(3)).await;

        let error = handle
            .await
            .expect("送信 task は panic しない")
            .expect_err("制限時間で失敗する");
        let SubmitError::TimedOut(returned) = error else {
            panic!("想定外のエラー: {error:?}");
        };
        assert_eq!(returned.sequence(), 2);
        assert_eq!(returned.payload(), "返却される遠隔測定");
        assert_eq!(returned.payload.as_ptr(), payload_pointer);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn receiverが閉じた場合も未送信値を返す() {
        let (sender, receiver) = mpsc::channel(1);
        drop(receiver);
        let pending = telemetry(7, "閉鎖後も保持");
        let payload_pointer = pending.payload.as_ptr();

        let error = submit_with_timeout(&sender, pending, Duration::from_secs(30))
            .await
            .expect_err("閉じた channel には送信できない");
        let SubmitError::Closed(returned) = error else {
            panic!("想定外のエラー: {error:?}");
        };
        assert_eq!(returned.sequence(), 7);
        assert_eq!(returned.payload.as_ptr(), payload_pointer);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn guardの生存中だけ処理中件数を増やす() {
        let in_flight = InFlight::default();
        assert_eq!(in_flight.active(), 0);

        {
            let _guard = in_flight.enter();
            assert_eq!(in_flight.active(), 1);
        }

        assert_eq!(in_flight.active(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn taskをabortすると待機中のguardをdropする() {
        let in_flight = InFlight::default();
        let worker_in_flight = in_flight.clone();
        let (started_sender, started_receiver) = oneshot::channel();
        let (release_sender, release_receiver) = oneshot::channel();
        let handle = tokio::spawn(cancellable_work(
            worker_in_flight,
            started_sender,
            release_receiver,
        ));

        started_receiver.await.expect("guard の作成を確認できる");
        assert_eq!(in_flight.active(), 1);

        handle.abort();
        let error = handle.await.expect_err("task はキャンセルされる");
        assert!(error.is_cancelled());
        assert_eq!(in_flight.active(), 0);
        drop(release_sender);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn taskが通常終了した場合もguardをdropする() {
        let in_flight = InFlight::default();
        let worker_in_flight = in_flight.clone();
        let (started_sender, started_receiver) = oneshot::channel();
        let (release_sender, release_receiver) = oneshot::channel();
        let handle = tokio::spawn(cancellable_work(
            worker_in_flight,
            started_sender,
            release_receiver,
        ));

        started_receiver.await.expect("guard の作成を確認できる");
        assert_eq!(in_flight.active(), 1);
        release_sender.send(()).expect("task を終了できる");
        handle.await.expect("task は正常終了する");

        assert_eq!(in_flight.active(), 0);
    }
}
