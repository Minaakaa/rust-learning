#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 02: `Deref` と `Drop` でドック利用を安全に管理する
//!
//! 点検中のロボットと、そのロボットが使用しているドックを借用する
//! `DockGuard<'a>` を完成させます
//! ガードを持っている間はロボットを自然に操作でき、ガードがどの経路で破棄されても
//! ドックが必ず解放されるようにしてください
//!
//! 仕様:
//! - `DockGuard<'a>` は同じ期間だけ `Dock` と `Robot` を可変借用する
//! - `Deref<Target = Robot>` を実装し、ガードから `Robot` の読み取りメソッドを呼べるようにする
//! - `DerefMut` を実装し、ガードから `Robot` の更新メソッドを呼べるようにする
//! - 共有参照と可変参照の deref coercion で、`&Robot` と `&mut Robot` を取る関数へ渡せるようにする
//! - `Drop` ではドックの `occupied` を `false` に戻す
//! - 解放時に `release_count` を1増やし、`u32::MAX` ではオーバーフローさせず飽和させる
//! - 通常のスコープ終了、`drop(guard)`、早期 return のすべてで同じ解放処理を行う
//! - `acquire` はドックを使用中にし、ガードからその状態を確認できるようにする
//!
//! `Dock::acquire` は完成済みです
//! この問題ではガードのポインタらしい振る舞いと RAII に集中してください
//!
//! ヒント:
//! - `Deref::deref` は `&Self::Target`、`DerefMut::deref_mut` は `&mut Self::Target` を返す
//! - フィールドの `&mut Robot` を移動せず、必要な期間だけ再借用する
//! - `Drop::drop` は明示的に呼ばず、早く解放したい場合は `drop(guard)` を使う
//! - カウンターには `saturating_add` を使える
//! - panic 中にも `Drop` は実行されるため、`Drop::drop` 自体から panic させない

use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, Eq)]
struct Robot {
    id: String,
    battery_percent: u8,
    completed_missions: u32,
}

impl Robot {
    fn new(id: &str, battery_percent: u8) -> Self {
        Self {
            id: id.to_owned(),
            battery_percent: battery_percent.min(100),
            completed_missions: 0,
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn battery_percent(&self) -> u8 {
        self.battery_percent
    }

    fn completed_missions(&self) -> u32 {
        self.completed_missions
    }

    fn recharge(&mut self, amount: u8) {
        self.battery_percent = self.battery_percent.saturating_add(amount).min(100);
    }

    fn complete_mission(&mut self) {
        self.completed_missions = self.completed_missions.saturating_add(1);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Dock {
    name: String,
    occupied: bool,
    release_count: u32,
}

impl Dock {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            occupied: false,
            release_count: 0,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_occupied(&self) -> bool {
        self.occupied
    }

    fn release_count(&self) -> u32 {
        self.release_count
    }

    fn acquire<'a>(&'a mut self, robot: &'a mut Robot) -> DockGuard<'a> {
        self.occupied = true;
        DockGuard { dock: self, robot }
    }
}

#[derive(Debug)]
struct DockGuard<'a> {
    dock: &'a mut Dock,
    #[allow(
        dead_code,
        reason = "完成前のスターターでは Deref 実装から参照されないため"
    )]
    robot: &'a mut Robot,
}

impl DockGuard<'_> {
    fn dock_name(&self) -> &str {
        self.dock.name()
    }

    fn dock_is_occupied(&self) -> bool {
        self.dock.is_occupied()
    }
}

impl Deref for DockGuard<'_> {
    type Target = Robot;

    fn deref(&self) -> &Self::Target {
        todo!("ガードが借用している Robot への共有参照を返してください")
    }
}

impl DerefMut for DockGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        todo!("ガードが借用している Robot への可変参照を返してください")
    }
}

impl Drop for DockGuard<'_> {
    fn drop(&mut self) {
        // TODO: ドックを未使用へ戻し、release_count を飽和加算してください
        // panic の巻き戻し中に二重 panic を起こさないよう、ここでは todo!() を使いません
    }
}

fn main() {
    let mut dock = Dock::new("本郷・整備ドックA");
    let mut robot = Robot::new("配送ロボット-802", 55);

    {
        let mut guard = dock.acquire(&mut robot);
        guard.recharge(20);
        guard.complete_mission();
        println!(
            "{} で {} を点検中: 電池 {}%",
            guard.dock_name(),
            guard.id(),
            guard.battery_percent()
        );
    }

    println!(
        "解放済み={}、解放回数={}",
        !dock.is_occupied(),
        dock.release_count()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquireでドックを使用中にしガードから確認できる() {
        let mut dock = Dock::new("D-acquire");
        let mut robot = Robot::new("R-801", 37);

        let guard = dock.acquire(&mut robot);

        assert_eq!(guard.dock_name(), "D-acquire");
        assert!(guard.dock_is_occupied());
        assert_eq!(guard.battery_percent(), 37);
    }

    #[test]
    fn derefでrobotの読み取りメソッドを直接呼べる() {
        let mut dock = Dock::new("D-read");
        let mut robot = Robot::new("R-802", 64);

        let guard = dock.acquire(&mut robot);

        assert_eq!(guard.id(), "R-802");
        assert_eq!(guard.battery_percent(), 64);
        assert_eq!(guard.completed_missions(), 0);
    }

    #[test]
    fn 共有参照へのderef_coercionが働く() {
        fn robot_summary(robot: &Robot) -> String {
            format!("{}:{}%", robot.id(), robot.battery_percent())
        }

        let mut dock = Dock::new("D-coerce");
        let mut robot = Robot::new("R-803", 48);
        let guard = dock.acquire(&mut robot);

        assert_eq!(robot_summary(&guard), "R-803:48%");
    }

    #[test]
    fn deref_mutでrobotを更新し可変参照として渡せる() {
        fn finish_service(robot: &mut Robot) {
            robot.recharge(25);
            robot.complete_mission();
        }

        let mut dock = Dock::new("D-write");
        let mut robot = Robot::new("R-804", 50);
        let mut guard = dock.acquire(&mut robot);

        guard.recharge(10);
        finish_service(&mut guard);

        assert_eq!(guard.battery_percent(), 85);
        assert_eq!(guard.completed_missions(), 1);
    }

    #[test]
    fn 通常のスコープ終了でドックを解放する() {
        let mut dock = Dock::new("D-scope");
        let mut robot = Robot::new("R-805", 70);

        {
            let guard = dock.acquire(&mut robot);
            assert_eq!(guard.dock_name(), "D-scope");
        }

        assert!(!dock.is_occupied());
        assert_eq!(dock.release_count(), 1);
    }

    #[test]
    fn 明示的なdropで直ちに一度だけ解放する() {
        let mut dock = Dock::new("D-explicit");
        let mut robot = Robot::new("R-806", 80);
        let guard = dock.acquire(&mut robot);

        drop(guard);

        assert!(!dock.is_occupied());
        assert_eq!(dock.release_count(), 1);
    }

    #[test]
    fn 早期returnでもドックを解放する() {
        fn interrupted_service(dock: &mut Dock, robot: &mut Robot) -> Result<(), &'static str> {
            let mut guard = dock.acquire(robot);
            guard.recharge(5);
            Err("点検を中断")
        }

        let mut dock = Dock::new("D-return");
        let mut robot = Robot::new("R-807", 20);

        assert_eq!(
            interrupted_service(&mut dock, &mut robot),
            Err("点検を中断")
        );
        assert!(!dock.is_occupied());
        assert_eq!(dock.release_count(), 1);
        assert_eq!(robot.battery_percent(), 25);
    }

    #[test]
    fn 解放回数はu32の最大値で飽和する() {
        let mut dock = Dock::new("D-saturating");
        dock.release_count = u32::MAX;
        let mut robot = Robot::new("R-808", 90);

        {
            let _guard = dock.acquire(&mut robot);
        }

        assert!(!dock.is_occupied());
        assert_eq!(dock.release_count(), u32::MAX);
    }

    #[test]
    fn utf8の名前を保ったまま操作して解放する() {
        let mut dock = Dock::new("本郷・整備ドック🚧-七");
        let mut robot = Robot::new("配送ロボット🤖-七", 95);

        {
            let mut guard = dock.acquire(&mut robot);
            assert_eq!(guard.dock_name(), "本郷・整備ドック🚧-七");
            assert_eq!(guard.id(), "配送ロボット🤖-七");
            guard.complete_mission();
        }

        assert_eq!(dock.name(), "本郷・整備ドック🚧-七");
        assert_eq!(dock.release_count(), 1);
        assert_eq!(robot.id(), "配送ロボット🤖-七");
        assert_eq!(robot.completed_missions(), 1);
    }
}
