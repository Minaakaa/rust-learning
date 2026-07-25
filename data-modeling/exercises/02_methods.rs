#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 02: メソッドでロボットの操作をまとめる
//!
//! 構造体に関係する操作を自由関数として散らす代わりに、`impl` ブロックへまとめます。
//! 各メソッドが値を読むだけか、更新するか、最後に所有権を消費するかを考えて、
//! `&self`、`&mut self`、`self` を使い分けてください。
//!
//! 仕様:
//! - `Mission::new` と `Robot::new` は関連関数として初期値を作る。
//! - ロボットはミッション未割り当てで、必要残量以上のバッテリーがあるときだけ受理できる。
//! - 割り当てに失敗したミッションは `Err` で所有権を返す。
//! - 完了時にバッテリーを消費し、走行距離を加算する。距離は飽和加算する。
//! - `into_report` はロボットを消費し、未完了ミッションがあればその ID を報告する。

#[derive(Debug, Clone, PartialEq, Eq)]
struct Mission {
    id: String,
    distance_m: u32,
    required_battery: u8,
}

impl Mission {
    fn new(id: &str, distance_m: u32, required_battery: u8) -> Self {
        todo!(
            "ミッション {id} を距離 {distance_m} m、必要残量 {required_battery}% で作ってください"
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Robot {
    id: String,
    battery_percent: u8,
    travelled_m: u32,
    active_mission: Option<Mission>,
}

#[derive(Debug, PartialEq, Eq)]
struct RobotReport {
    id: String,
    battery_percent: u8,
    travelled_m: u32,
    unfinished_mission_id: Option<String>,
}

impl Robot {
    fn new(id: &str) -> Self {
        todo!("満充電で未割り当てのロボット {id} を作ってください")
    }

    fn can_accept(&self, mission: &Mission) -> bool {
        todo!(
            "{} が {} を受理できるか、状態を変更せず判定してください",
            self.id,
            mission.id
        )
    }

    fn assign(&mut self, mission: Mission) -> Result<(), Mission> {
        todo!(
            "{} へ {} を割り当てるか、所有権を Err で返してください",
            self.id,
            mission.id
        )
    }

    fn complete_active(&mut self) -> Option<Mission> {
        todo!("{} の実行中ミッションを完了してください", self.id)
    }

    fn into_report(self) -> RobotReport {
        todo!("{} を消費して最終レポートへ変換してください", self.id)
    }
}

fn main() {
    let mut robot = Robot::new("RB-20");
    let mission = Mission::new("M-100", 450, 25);

    robot.assign(mission).expect("初期状態なら割り当てられる");
    robot.complete_active();
    println!("レポート: {:#?}", robot.into_report());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 関連関数で初期状態を作る() {
        assert_eq!(
            Mission::new("M-1", 300, 20),
            Mission {
                id: "M-1".to_string(),
                distance_m: 300,
                required_battery: 20,
            }
        );
        assert_eq!(
            Robot::new("RB-20"),
            Robot {
                id: "RB-20".to_string(),
                battery_percent: 100,
                travelled_m: 0,
                active_mission: None,
            }
        );
    }

    #[test]
    fn 共有借用で受理可能かを判定する() {
        let robot = Robot::new("RB-21");
        let exact = Mission::new("M-2", 100, 100);
        let too_expensive = Mission::new("M-3", 100, 101);

        assert!(robot.can_accept(&exact));
        assert!(!robot.can_accept(&too_expensive));
        assert!(robot.active_mission.is_none());
        assert_eq!(robot.battery_percent, 100);
    }

    #[test]
    fn 可変借用でミッションを割り当てる() {
        let mut robot = Robot::new("RB-22");
        let mission = Mission::new("M-4", 250, 30);

        assert_eq!(robot.assign(mission), Ok(()));
        assert_eq!(robot.active_mission, Some(Mission::new("M-4", 250, 30)));
        assert_eq!(robot.battery_percent, 100);
    }

    #[test]
    fn 割り当て失敗時にミッションを失わない() {
        let mut robot = Robot::new("RB-23");
        robot.battery_percent = 10;
        let expensive = Mission::new("M-5", 700, 40);

        assert_eq!(robot.assign(expensive), Err(Mission::new("M-5", 700, 40)));
        assert!(robot.active_mission.is_none());

        robot.battery_percent = 100;
        robot.assign(Mission::new("M-6", 100, 10)).unwrap();
        let waiting = Mission::new("M-7", 200, 10);
        assert_eq!(robot.assign(waiting), Err(Mission::new("M-7", 200, 10)));
    }

    #[test]
    fn 完了時に状態を一度だけ更新する() {
        let mut robot = Robot::new("RB-24");
        robot.assign(Mission::new("M-8", 600, 35)).unwrap();

        let completed = robot.complete_active();

        assert_eq!(completed, Some(Mission::new("M-8", 600, 35)));
        assert_eq!(robot.battery_percent, 65);
        assert_eq!(robot.travelled_m, 600);
        assert!(robot.active_mission.is_none());
        assert_eq!(robot.complete_active(), None);
        assert_eq!(robot.battery_percent, 65);
        assert_eq!(robot.travelled_m, 600);
    }

    #[test]
    fn 走行距離をオーバーフローさせない() {
        let mut robot = Robot::new("RB-25");
        robot.travelled_m = u32::MAX - 10;
        robot.assign(Mission::new("M-9", 50, 1)).unwrap();

        robot.complete_active();

        assert_eq!(robot.travelled_m, u32::MAX);
    }

    #[test]
    fn 所有権を消費してレポートへ変換する() {
        let idle_report = Robot::new("RB-IDLE").into_report();
        assert_eq!(idle_report.unfinished_mission_id, None);

        let mut robot = Robot::new("RB-26");
        robot.battery_percent = 80;
        robot.travelled_m = 1_200;
        robot.assign(Mission::new("M-10", 200, 10)).unwrap();

        let report = robot.into_report();

        assert_eq!(
            report,
            RobotReport {
                id: "RB-26".to_string(),
                battery_percent: 80,
                travelled_m: 1_200,
                unfinished_mission_id: Some("M-10".to_string()),
            }
        );
    }
}
