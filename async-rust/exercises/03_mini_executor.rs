#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 03: `Waker` で駆動する小さな executor を作る
//!
//! `Future` は自分では処理を進めません
//! 実行可能になった task を ready queue へ戻し、再び poll する executor が必要です
//! 標準ライブラリだけを使い、単一スレッドのミニ executor を完成させてください
//!
//! 仕様:
//! - task は `Pin<Box<dyn Future<Output = ()> + Send>>` を所有する
//! - `Spawner::spawn` は future を task に包み、最初の poll のため ready queue へ送る
//! - `Task` は安全な `std::task::Wake` を実装し、wake 時に自分を ready queue へ戻す
//! - task がすでに queue 内なら重複した wake を一つの実行予約へまとめる
//! - executor は `mpsc::Receiver::recv` で待ち、busy loop を行わない
//! - queue から取り出した印を付けてから future を poll し、`Pending` なら slot へ戻す
//! - `Ready(())` なら future を破棄し、完了数を1増やす
//! - 重複した wake で完了済み task が届いても、再度 poll も完了数の加算もしない
//! - すべての spawner と task がなくなり queue が閉じたら `run` を終了する
//! - executor が先に破棄された場合、`spawn` は `SpawnError::ExecutorClosed` を返す
//! - `unsafe`、`thread::sleep`、外部 crate は使わない
//!
//! TODO:
//! - `Task::enqueue` と `Wake` で未予約の task だけを ready queue へ送る
//! - `Spawner::spawn` で future を pin して最初の実行を予約する
//! - `Executor::run` で queue 状態を解除してから task を1つずつ poll する
//!
//! ヒント:
//! - `queued` は task の `Arc` が queue に入ってから executor が取り出すまで `true` にする
//! - `send` が失敗した場合は `queued` を `false` に戻す
//! - poll の前に `queued` を解除すれば、poll 中の wake を次の実行予約として保持できる

use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex, MutexGuard,
        mpsc::{self, Receiver, Sender},
    },
    task::{Context, Poll, Wake},
};

type TaskFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[allow(
    dead_code,
    reason = "executor 完成前のスターターでは task のフィールドを読み取らないため"
)]
struct Task {
    future: Mutex<Option<TaskFuture>>,
    queued: Mutex<bool>,
    ready_sender: Sender<Arc<Task>>,
}

impl Task {
    #[allow(
        dead_code,
        reason = "run 完成前のスターターでは future slot を lock しないため"
    )]
    fn lock_future(&self) -> MutexGuard<'_, Option<TaskFuture>> {
        self.future
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[allow(
        dead_code,
        reason = "enqueue 完成前のスターターでは queue 状態を lock しないため"
    )]
    fn lock_queued(&self) -> MutexGuard<'_, bool> {
        self.queued
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn enqueue(self: &Arc<Self>) -> Result<(), SpawnError> {
        todo!("未予約の task だけを ready queue へ送ってください")
    }

    fn mark_dequeued(&self) {
        todo!("poll 前に task の queue 状態を解除してください")
    }
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        let _ = self.enqueue();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let _ = self.enqueue();
    }
}

#[derive(Clone)]
#[allow(
    dead_code,
    reason = "spawn 完成前のスターターでは ready sender を使わないため"
)]
struct Spawner {
    ready_sender: Sender<Arc<Task>>,
}

#[derive(Debug, PartialEq, Eq)]
enum SpawnError {
    ExecutorClosed,
}

impl Spawner {
    fn spawn<F>(&self, future: F) -> Result<(), SpawnError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        std::mem::drop(future);
        todo!("future を pin した task を作り、最初の poll を予約してください")
    }
}

#[allow(
    dead_code,
    reason = "run 完成前のスターターでは ready receiver を使わないため"
)]
struct Executor {
    ready_receiver: Receiver<Arc<Task>>,
}

impl Executor {
    fn run(self) -> usize {
        let _ = self;
        todo!("ready queue が閉じるまで task を poll してください")
    }
}

fn mini_executor() -> (Spawner, Executor) {
    let (ready_sender, ready_receiver) = mpsc::channel();
    (Spawner { ready_sender }, Executor { ready_receiver })
}

struct YieldOnce {
    yielded: bool,
}

impl Future for YieldOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            context.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

fn yield_once() -> YieldOnce {
    YieldOnce { yielded: false }
}

fn main() {
    let (spawner, executor) = mini_executor();
    let events = Arc::new(Mutex::new(Vec::new()));
    let task_events = Arc::clone(&events);

    spawner
        .spawn(async move {
            task_events
                .lock()
                .expect("イベントを記録できる")
                .push("点検開始");
            yield_once().await;
            task_events
                .lock()
                .expect("イベントを記録できる")
                .push("点検完了");
        })
        .expect("task を登録できる");
    drop(spawner);

    let completed = executor.run();
    println!("完了 task 数: {completed}、イベント: {events:?}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        task::Waker,
        thread,
    };

    fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
        value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn next_task(executor: &Executor) -> Arc<Task> {
        executor
            .ready_receiver
            .try_recv()
            .expect("ready queue から task を取り出せる")
    }

    fn assert_ready_queue_empty(executor: &Executor) {
        assert!(matches!(
            executor.ready_receiver.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }

    fn poll_once(task: &Arc<Task>) -> Option<Poll<()>> {
        task.mark_dequeued();
        let mut future_slot = task.lock_future();
        let mut future = future_slot.take()?;
        let waker = Waker::from(Arc::clone(task));
        let mut context = Context::from_waker(&waker);

        match future.as_mut().poll(&mut context) {
            Poll::Ready(()) => Some(Poll::Ready(())),
            Poll::Pending => {
                *future_slot = Some(future);
                Some(Poll::Pending)
            }
        }
    }

    #[test]
    fn taskがなければqueue切断後すぐ終了する() {
        let (spawner, executor) = mini_executor();
        drop(spawner);

        assert_eq!(executor.run(), 0);
    }

    #[test]
    fn readyなtaskを登録順に一度ずつ実行する() {
        let (spawner, executor) = mini_executor();
        let events = Arc::new(Mutex::new(Vec::new()));

        for id in ["R-1", "R-2", "R-3"] {
            let task_events = Arc::clone(&events);
            spawner
                .spawn(async move { lock(&task_events).push(id) })
                .expect("task を登録できる");
        }
        drop(spawner);

        assert_eq!(executor.run(), 3);
        assert_eq!(*lock(&events), ["R-1", "R-2", "R-3"]);
    }

    #[test]
    fn pendingになったtaskをwake順に再pollする() {
        let (spawner, executor) = mini_executor();
        let events = Arc::new(Mutex::new(Vec::new()));

        for id in ["A", "B"] {
            let task_events = Arc::clone(&events);
            spawner
                .spawn(async move {
                    lock(&task_events).push(format!("{id}-開始"));
                    yield_once().await;
                    lock(&task_events).push(format!("{id}-完了"));
                })
                .expect("task を登録できる");
        }
        drop(spawner);

        assert_eq!(executor.run(), 2);
        assert_eq!(*lock(&events), ["A-開始", "B-開始", "A-完了", "B-完了"]);
    }

    #[test]
    fn futureへ所有値をmoveして結果へそのまま移せる() {
        let (spawner, executor) = mini_executor();
        let (result_sender, result_receiver) = mpsc::channel();
        let robot_id = String::from("配送ロボット-owned");
        let pointer = robot_id.as_ptr();

        spawner
            .spawn(async move {
                result_sender.send(robot_id).expect("結果を送信できる");
            })
            .expect("task を登録できる");
        drop(spawner);

        assert_eq!(executor.run(), 1);
        let returned = result_receiver.recv().expect("結果を受信できる");
        assert_eq!(returned, "配送ロボット-owned");
        assert_eq!(returned.as_ptr(), pointer);
    }

    struct GateState {
        open: bool,
        waker: Option<Waker>,
        first_poll: Option<Sender<()>>,
    }

    struct GateFuture {
        state: Arc<Mutex<GateState>>,
    }

    struct GateHandle {
        state: Arc<Mutex<GateState>>,
    }

    fn gate() -> (GateHandle, GateFuture, Receiver<()>) {
        let (first_poll_sender, first_poll_receiver) = mpsc::channel();
        let state = Arc::new(Mutex::new(GateState {
            open: false,
            waker: None,
            first_poll: Some(first_poll_sender),
        }));
        (
            GateHandle {
                state: Arc::clone(&state),
            },
            GateFuture { state },
            first_poll_receiver,
        )
    }

    impl Future for GateFuture {
        type Output = ();

        fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
            let mut state = lock(&self.state);
            if state.open {
                Poll::Ready(())
            } else {
                state.waker = Some(context.waker().clone());
                if let Some(sender) = state.first_poll.take() {
                    sender.send(()).expect("最初の poll を通知できる");
                }
                Poll::Pending
            }
        }
    }

    impl GateHandle {
        fn open(self) {
            let waker = {
                let mut state = lock(&self.state);
                state.open = true;
                state.waker.take()
            };
            if let Some(waker) = waker {
                waker.wake();
            }
        }
    }

    #[test]
    fn pending中はbusy_loopせず外部wakeで再開する() {
        let (spawner, executor) = mini_executor();
        let (handle, future, first_poll) = gate();
        let polls = Arc::new(AtomicUsize::new(0));
        let task_polls = Arc::clone(&polls);
        let (done_sender, done_receiver) = mpsc::channel();

        spawner
            .spawn(async move {
                task_polls.fetch_add(1, Ordering::SeqCst);
                future.await;
                task_polls.fetch_add(1, Ordering::SeqCst);
                done_sender.send(()).expect("完了を通知できる");
            })
            .expect("task を登録できる");
        drop(spawner);

        let executor_thread = thread::spawn(move || executor.run());
        first_poll.recv().expect("最初の poll を確認できる");
        assert_eq!(polls.load(Ordering::SeqCst), 1);

        handle.open();

        done_receiver.recv().expect("task の完了を確認できる");
        assert_eq!(executor_thread.join().expect("executor は panic しない"), 1);
        assert_eq!(polls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn 最初の実行待ち中の重複wakeはqueueを増やさない() {
        let (spawner, executor) = mini_executor();
        spawner
            .spawn(std::future::pending::<()>())
            .expect("task を登録できる");

        let task = next_task(&executor);
        let waker = Waker::from(Arc::clone(&task));
        waker.wake_by_ref();
        waker.wake_by_ref();

        assert_ready_queue_empty(&executor);
    }

    struct WakeTwicePending;

    impl Future for WakeTwicePending {
        type Output = ();

        fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
            context.waker().wake_by_ref();
            context.waker().wake_by_ref();
            Poll::Pending
        }
    }

    #[test]
    fn pendingを返すpoll中の重複wakeは一つの実行予約になる() {
        let (spawner, executor) = mini_executor();
        spawner.spawn(WakeTwicePending).expect("task を登録できる");

        let task = next_task(&executor);
        assert_eq!(poll_once(&task), Some(Poll::Pending));

        let queued_task = next_task(&executor);
        assert!(Arc::ptr_eq(&task, &queued_task));
        assert_ready_queue_empty(&executor);
    }

    struct WakeTwice {
        polls: Arc<AtomicUsize>,
    }

    impl Future for WakeTwice {
        type Output = ();

        fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
            self.polls.fetch_add(1, Ordering::SeqCst);
            context.waker().wake_by_ref();
            context.waker().wake_by_ref();
            Poll::Ready(())
        }
    }

    #[test]
    fn readyを返すpoll中の重複wakeも一つの実行予約になる() {
        let (spawner, executor) = mini_executor();
        let polls = Arc::new(AtomicUsize::new(0));

        spawner
            .spawn(WakeTwice {
                polls: Arc::clone(&polls),
            })
            .expect("task を登録できる");

        let task = next_task(&executor);
        assert_eq!(poll_once(&task), Some(Poll::Ready(())));

        let queued_task = next_task(&executor);
        assert!(Arc::ptr_eq(&task, &queued_task));
        assert_ready_queue_empty(&executor);
        assert_eq!(poll_once(&queued_task), None);
        assert_eq!(polls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn executorは完了時の重複wakeを再pollも二重集計もしない() {
        let (spawner, executor) = mini_executor();
        let polls = Arc::new(AtomicUsize::new(0));

        spawner
            .spawn(WakeTwice {
                polls: Arc::clone(&polls),
            })
            .expect("task を登録できる");
        drop(spawner);

        assert_eq!(executor.run(), 1);
        assert_eq!(polls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn executor破棄後のspawnはerrorを返す() {
        let (spawner, executor) = mini_executor();
        drop(executor);

        assert_eq!(spawner.spawn(async {}), Err(SpawnError::ExecutorClosed));
    }
}
