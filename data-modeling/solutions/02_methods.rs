//! 問題 02 の解答例。

#[derive(Debug, Clone, PartialEq, Eq)]
struct Mission {
    id: String,
    distance_m: u32,
    required_battery: u8,
}

impl Mission {
    fn new(id: &str, distance_m: u32, required_battery: u8) -> Self {
        Self {
            id: id.to_string(),
            distance_m,
            required_battery,
        }
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
        Self {
            id: id.to_string(),
            battery_percent: 100,
            travelled_m: 0,
            active_mission: None,
        }
    }

    fn can_accept(&self, mission: &Mission) -> bool {
        self.active_mission.is_none() && mission.required_battery <= self.battery_percent
    }

    fn assign(&mut self, mission: Mission) -> Result<(), Mission> {
        if self.can_accept(&mission) {
            self.active_mission = Some(mission);
            Ok(())
        } else {
            Err(mission)
        }
    }

    fn complete_active(&mut self) -> Option<Mission> {
        let mission = self.active_mission.take()?;
        self.battery_percent -= mission.required_battery;
        self.travelled_m = self.travelled_m.saturating_add(mission.distance_m);
        Some(mission)
    }

    fn into_report(self) -> RobotReport {
        RobotReport {
            id: self.id,
            battery_percent: self.battery_percent,
            travelled_m: self.travelled_m,
            unfinished_mission_id: self.active_mission.map(|mission| mission.id),
        }
    }
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
