//! 問題 02 の解答例

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
        self.robot
    }
}

impl DerefMut for DockGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.robot
    }
}

impl Drop for DockGuard<'_> {
    fn drop(&mut self) {
        self.dock.occupied = false;
        self.dock.release_count = self.dock.release_count.saturating_add(1);
    }
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
