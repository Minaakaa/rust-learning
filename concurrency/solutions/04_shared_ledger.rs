#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 04: Arc と Mutex で配送台帳を共有する
//!
//! 複数の worker thread が同じ配送台帳へ完了記録を書き込みます
//! `Arc` で台帳の所有権を共有し、`Mutex` の短いロック区間で状態を更新してください
//! 実行順序に左右されない集計と、キー順が決定的なスナップショットを作ります
//!
//! 仕様:
//! - `SharedLedger` は `Arc<Mutex<LedgerState>>` を保持する
//! - `record` は所有する `Delivery` を受け取り、robot ID を複製せず台帳へ移す
//! - 全体とロボット別の配送回数、距離を `saturating_add` で更新する
//! - `snapshot` はキー順が安定する `BTreeMap` を含む独立した値を返す
//! - `record_batches` は各バッチにつき1つの worker thread を起動する
//! - 各 worker は `Arc` の所有者を1つ持ち、すべての thread を join してから戻る
//! - poisoned mutex は `PoisonError::into_inner` で回復する
//!
//! この課題で回復対象とするのは、更新前に別処理が panic して付いた poison です
//! `record` はユーザーコードや callback をロック中に呼ばず、閉じた飽和加算だけを
//! 行います
//! 一般の `Mutex` で意味上の不変条件が壊れた可能性まで無条件に回復してはいけません
//!
//! 制約:
//! - `Rc`、`RefCell`、global state、外部 crate、`unsafe` を使わない
//! - ロックを保持したまま thread の join や待機を行わない
//! - worker の実行順序を前提にしない
//!
//! ヒント:
//! - worker ごとに `Arc::clone` し、同じ allocation の所有者だけを増やす
//! - `Mutex::lock` の poison error からは `PoisonError::into_inner` で guard を回収できる
//! - `BTreeMap::entry` を使うと初回登録と既存 robot の更新を同じ処理にできる
//! - 先に全 handle を集め、ロックを保持していない場所で1つずつ join する

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;

#[derive(Debug, PartialEq, Eq)]
struct Delivery {
    robot_id: String,
    distance_m: u64,
}

impl Delivery {
    fn new(robot_id: String, distance_m: u64) -> Self {
        Self {
            robot_id,
            distance_m,
        }
    }

    fn robot_id(&self) -> &str {
        &self.robot_id
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RobotTotals {
    deliveries: u64,
    distance_m: u64,
}

impl RobotTotals {
    const fn deliveries(&self) -> u64 {
        self.deliveries
    }

    const fn distance_m(&self) -> u64 {
        self.distance_m
    }
}

#[derive(Debug, Default)]
struct LedgerState {
    total_deliveries: u64,
    total_distance_m: u64,
    by_robot: BTreeMap<String, RobotTotals>,
}

#[derive(Clone, Debug)]
struct SharedLedger {
    state: Arc<Mutex<LedgerState>>,
}

fn join_all(handles: Vec<thread::JoinHandle<()>>) {
    let mut first_panic = None;

    for handle in handles {
        if let Err(payload) = handle.join()
            && first_panic.is_none()
        {
            first_panic = Some(payload);
        }
    }

    if let Some(payload) = first_panic {
        std::panic::resume_unwind(payload);
    }
}

impl SharedLedger {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(LedgerState::default())),
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, LedgerState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn record(&self, delivery: Delivery) {
        let Delivery {
            robot_id,
            distance_m,
        } = delivery;
        let mut state = self.lock_state();

        state.total_deliveries = state.total_deliveries.saturating_add(1);
        state.total_distance_m = state.total_distance_m.saturating_add(distance_m);

        let totals = state.by_robot.entry(robot_id).or_default();
        totals.deliveries = totals.deliveries.saturating_add(1);
        totals.distance_m = totals.distance_m.saturating_add(distance_m);
    }

    fn snapshot(&self) -> LedgerSnapshot {
        let state = self.lock_state();

        LedgerSnapshot {
            total_deliveries: state.total_deliveries,
            total_distance_m: state.total_distance_m,
            by_robot: state.by_robot.clone(),
        }
    }

    fn record_batches(&self, batches: Vec<Vec<Delivery>>) {
        let mut handles = Vec::with_capacity(batches.len());

        for batch in batches {
            let worker_ledger = Self {
                state: Arc::clone(&self.state),
            };
            handles.push(thread::spawn(move || {
                for delivery in batch {
                    worker_ledger.record(delivery);
                }
            }));
        }

        join_all(handles);
    }

    fn shares_state_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }

    fn strong_count(&self) -> usize {
        Arc::strong_count(&self.state)
    }

    fn is_poisoned(&self) -> bool {
        self.state.is_poisoned()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LedgerSnapshot {
    total_deliveries: u64,
    total_distance_m: u64,
    by_robot: BTreeMap<String, RobotTotals>,
}

impl LedgerSnapshot {
    const fn total_deliveries(&self) -> u64 {
        self.total_deliveries
    }

    const fn total_distance_m(&self) -> u64 {
        self.total_distance_m
    }

    fn robot(&self, robot_id: &str) -> Option<&RobotTotals> {
        self.by_robot.get(robot_id)
    }

    fn robots(&self) -> &BTreeMap<String, RobotTotals> {
        &self.by_robot
    }
}

fn main() {
    let ledger = SharedLedger::new();
    ledger.record_batches(vec![
        vec![
            Delivery::new(String::from("配送ロボット-904A"), 320),
            Delivery::new(String::from("配送ロボット-904A"), 180),
        ],
        vec![Delivery::new(String::from("配送ロボット-904B"), 275)],
    ]);

    let snapshot = ledger.snapshot();
    println!(
        "配送={}件 総距離={}m ロボット={}台",
        snapshot.total_deliveries(),
        snapshot.total_distance_m(),
        snapshot.robots().len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delivery(robot_id: &str, distance_m: u64) -> Delivery {
        Delivery::new(robot_id.to_owned(), distance_m)
    }

    #[test]
    fn shared_ledgerはsend_syncで空の状態から始まる() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<SharedLedger>();

        let ledger = SharedLedger::new();
        let snapshot = ledger.snapshot();

        assert_eq!(snapshot.total_deliveries(), 0);
        assert_eq!(snapshot.total_distance_m(), 0);
        assert!(snapshot.robots().is_empty());
        assert_eq!(ledger.strong_count(), 1);
        assert!(!ledger.is_poisoned());
    }

    #[test]
    fn 一件の配送を全体とrobot別へ記録する() {
        let ledger = SharedLedger::new();

        ledger.record(delivery("R-01", 125));
        let snapshot = ledger.snapshot();

        assert_eq!(snapshot.total_deliveries(), 1);
        assert_eq!(snapshot.total_distance_m(), 125);
        assert_eq!(snapshot.robot("R-01").unwrap().deliveries(), 1);
        assert_eq!(snapshot.robot("R-01").unwrap().distance_m(), 125);
    }

    #[test]
    fn 複数robotの記録を独立して集計する() {
        let ledger = SharedLedger::new();

        ledger.record(delivery("R-01", 120));
        ledger.record(delivery("R-02", 40));
        ledger.record(delivery("R-01", 80));
        let snapshot = ledger.snapshot();

        assert_eq!(snapshot.total_deliveries(), 3);
        assert_eq!(snapshot.total_distance_m(), 240);
        assert_eq!(snapshot.robot("R-01").unwrap().deliveries(), 2);
        assert_eq!(snapshot.robot("R-01").unwrap().distance_m(), 200);
        assert_eq!(snapshot.robot("R-02").unwrap().deliveries(), 1);
        assert_eq!(snapshot.robot("R-02").unwrap().distance_m(), 40);
        assert!(snapshot.robot("R-missing").is_none());
    }

    #[test]
    fn robot_idのstringを複製せず台帳へ移す() {
        let delivery = delivery("移動確認ロボット", 9);
        let id_pointer = delivery.robot_id().as_ptr();
        let ledger = SharedLedger::new();

        ledger.record(delivery);
        let state = ledger.lock_state();
        let stored_id = state.by_robot.keys().next().expect("robot が記録される");

        assert_eq!(stored_id, "移動確認ロボット");
        assert_eq!(stored_id.as_ptr(), id_pointer);
    }

    #[test]
    fn 回数と距離をオーバーフローさせず飽和する() {
        let ledger = SharedLedger::new();
        {
            let mut state = ledger.lock_state();
            state.total_deliveries = u64::MAX;
            state.total_distance_m = u64::MAX - 2;
            state.by_robot.insert(
                String::from("R-max"),
                RobotTotals {
                    deliveries: u64::MAX,
                    distance_m: u64::MAX - 1,
                },
            );
        }

        ledger.record(delivery("R-max", 10));
        let snapshot = ledger.snapshot();
        let robot = snapshot.robot("R-max").unwrap();

        assert_eq!(snapshot.total_deliveries(), u64::MAX);
        assert_eq!(snapshot.total_distance_m(), u64::MAX);
        assert_eq!(robot.deliveries(), u64::MAX);
        assert_eq!(robot.distance_m(), u64::MAX);
    }

    #[test]
    fn snapshotのrobotをキー順で反復できる() {
        let ledger = SharedLedger::new();
        ledger.record(delivery("R-10", 10));
        ledger.record(delivery("R-02", 2));
        ledger.record(delivery("R-01", 1));

        let snapshot = ledger.snapshot();
        let ids = snapshot
            .robots()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();

        assert_eq!(ids, ["R-01", "R-02", "R-10"]);
    }

    #[test]
    fn cloneしたledgerは同じstateを共有する() {
        let ledger = SharedLedger::new();
        let clone = ledger.clone();

        assert!(ledger.shares_state_with(&clone));
        assert_eq!(ledger.strong_count(), 2);

        clone.record(delivery("R-shared", 33));
        assert_eq!(ledger.snapshot().total_distance_m(), 33);

        drop(clone);
        assert_eq!(ledger.strong_count(), 1);
    }

    #[test]
    fn 複数batchを実行順に依存せず集計する() {
        let ledger = SharedLedger::new();
        ledger.record_batches(vec![
            vec![delivery("R-A", 10), delivery("R-A", 20)],
            Vec::new(),
            vec![delivery("R-B", 7), delivery("R-A", 5)],
            vec![delivery("R-C", 100)],
        ]);

        let snapshot = ledger.snapshot();

        assert_eq!(snapshot.total_deliveries(), 5);
        assert_eq!(snapshot.total_distance_m(), 142);
        assert_eq!(snapshot.robot("R-A").unwrap().deliveries(), 3);
        assert_eq!(snapshot.robot("R-A").unwrap().distance_m(), 35);
        assert_eq!(snapshot.robot("R-B").unwrap().distance_m(), 7);
        assert_eq!(snapshot.robot("R-C").unwrap().distance_m(), 100);
    }

    #[test]
    fn record_batchesはすべてのworkerをjoinしてから戻る() {
        let ledger = SharedLedger::new();
        let batches = (0..16)
            .map(|batch_index| {
                (0..32)
                    .map(|position| delivery(&format!("R-{batch_index:02}"), position + 1))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        ledger.record_batches(batches);
        let snapshot = ledger.snapshot();

        assert_eq!(ledger.strong_count(), 1);
        assert_eq!(snapshot.total_deliveries(), 512);
        assert_eq!(snapshot.total_distance_m(), 8_448);
        assert_eq!(snapshot.robots().len(), 16);
    }

    #[test]
    fn poisoned_mutexから回復して日本語idを記録できる() {
        let ledger = SharedLedger::new();
        ledger.record(delivery("既存ロボット", 4));
        let worker_ledger = ledger.clone();

        let panic_result = thread::spawn(move || {
            let _guard = worker_ledger
                .state
                .lock()
                .expect("poison 前の Mutex をロックできる");
            panic!("poison 回復を確認するための意図的な panic");
        })
        .join();

        assert!(panic_result.is_err());
        assert!(ledger.is_poisoned());

        ledger.record(delivery("復旧ロボット🤖", 8));
        let snapshot = ledger.snapshot();

        assert_eq!(snapshot.total_deliveries(), 2);
        assert_eq!(snapshot.total_distance_m(), 12);
        assert_eq!(snapshot.robot("既存ロボット").unwrap().distance_m(), 4);
        assert_eq!(snapshot.robot("復旧ロボット🤖").unwrap().distance_m(), 8);
    }
}
