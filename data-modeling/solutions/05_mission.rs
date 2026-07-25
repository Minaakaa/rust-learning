//! 問題 05 の解答例。

#[derive(Debug, Clone, PartialEq, Eq)]
enum ModelError {
    EmptyRobotId,
    EmptyMissionId,
    EmptyDestination,
    ZeroCapacity,
    ZeroCargoWeight,
    BatteryOutOfRange(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RobotId(String);

impl RobotId {
    fn new(value: &str) -> Result<Self, ModelError> {
        let value = value.trim();
        if value.is_empty() {
            Err(ModelError::EmptyRobotId)
        } else {
            Ok(Self(value.to_string()))
        }
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct MissionId(String);

impl MissionId {
    fn new(value: &str) -> Result<Self, ModelError> {
        let value = value.trim();
        if value.is_empty() {
            Err(ModelError::EmptyMissionId)
        } else {
            Ok(Self(value.to_string()))
        }
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Kilograms(u16);

impl Kilograms {
    const fn value(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BatteryPercent(u8);

impl BatteryPercent {
    fn new(value: u8) -> Result<Self, ModelError> {
        if value <= 100 {
            Ok(Self(value))
        } else {
            Err(ModelError::BatteryOutOfRange(value))
        }
    }

    const fn value(self) -> u8 {
        self.0
    }

    fn consume(&mut self, required: Self) {
        self.0 = self.0.saturating_sub(required.0);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Mission {
    id: MissionId,
    destination: String,
    cargo_weight: Kilograms,
    required_battery: BatteryPercent,
}

impl Mission {
    fn new(
        id: &str,
        destination: &str,
        cargo_weight_kg: u16,
        required_battery: u8,
    ) -> Result<Self, ModelError> {
        let id = MissionId::new(id)?;
        let destination = destination.trim();
        if destination.is_empty() {
            return Err(ModelError::EmptyDestination);
        }
        if cargo_weight_kg == 0 {
            return Err(ModelError::ZeroCargoWeight);
        }

        Ok(Self {
            id,
            destination: destination.to_string(),
            cargo_weight: Kilograms(cargo_weight_kg),
            required_battery: BatteryPercent::new(required_battery)?,
        })
    }

    fn id(&self) -> &MissionId {
        &self.id
    }

    fn destination(&self) -> &str {
        &self.destination
    }

    fn cargo_weight(&self) -> Kilograms {
        self.cargo_weight
    }

    fn required_battery(&self) -> BatteryPercent {
        self.required_battery
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DispatchReason {
    CargoTooHeavy {
        capacity: Kilograms,
        actual: Kilograms,
    },
    InsufficientBattery {
        available: BatteryPercent,
        required: BatteryPercent,
    },
}

#[derive(Debug, PartialEq, Eq)]
struct DispatchFailure {
    mission: Mission,
    reason: DispatchReason,
}

#[derive(Debug, PartialEq, Eq)]
struct DispatchReceipt {
    robot_id: RobotId,
    mission_id: MissionId,
    destination: String,
    cargo_weight: Kilograms,
    battery_remaining: BatteryPercent,
    sequence_number: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct Robot {
    id: RobotId,
    capacity: Kilograms,
    battery: BatteryPercent,
    completed_missions: u32,
}

impl Robot {
    fn new(id: &str, capacity_kg: u16, battery: u8) -> Result<Self, ModelError> {
        let id = RobotId::new(id)?;
        if capacity_kg == 0 {
            return Err(ModelError::ZeroCapacity);
        }

        Ok(Self {
            id,
            capacity: Kilograms(capacity_kg),
            battery: BatteryPercent::new(battery)?,
            completed_missions: 0,
        })
    }

    fn id(&self) -> &RobotId {
        &self.id
    }

    fn capacity(&self) -> Kilograms {
        self.capacity
    }

    fn battery(&self) -> BatteryPercent {
        self.battery
    }

    fn completed_missions(&self) -> u32 {
        self.completed_missions
    }

    fn check_mission(&self, mission: &Mission) -> Result<(), DispatchReason> {
        if mission.cargo_weight > self.capacity {
            return Err(DispatchReason::CargoTooHeavy {
                capacity: self.capacity,
                actual: mission.cargo_weight,
            });
        }
        if mission.required_battery > self.battery {
            return Err(DispatchReason::InsufficientBattery {
                available: self.battery,
                required: mission.required_battery,
            });
        }

        Ok(())
    }

    fn dispatch(&mut self, mission: Mission) -> Result<DispatchReceipt, DispatchFailure> {
        if let Err(reason) = self.check_mission(&mission) {
            return Err(DispatchFailure { mission, reason });
        }

        self.battery.consume(mission.required_battery);
        self.completed_missions = self.completed_missions.saturating_add(1);

        Ok(DispatchReceipt {
            robot_id: self.id.clone(),
            mission_id: mission.id,
            destination: mission.destination,
            cargo_weight: mission.cargo_weight,
            battery_remaining: self.battery,
            sequence_number: self.completed_missions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newtypeのコンストラクタが単独でも不変条件を守る() {
        assert_eq!(RobotId::new("  RB-00  "), Ok(RobotId("RB-00".to_string())));
        assert_eq!(RobotId::new("  "), Err(ModelError::EmptyRobotId));
        assert_eq!(
            MissionId::new("  M-000  "),
            Ok(MissionId("M-000".to_string()))
        );
        assert_eq!(MissionId::new(""), Err(ModelError::EmptyMissionId));
        assert_eq!(BatteryPercent::new(100), Ok(BatteryPercent(100)));
        assert_eq!(
            BatteryPercent::new(101),
            Err(ModelError::BatteryOutOfRange(101))
        );

        let mut battery = BatteryPercent::new(50).unwrap();
        battery.consume(BatteryPercent::new(20).unwrap());
        assert_eq!(battery.value(), 30);
    }

    #[test]
    fn 入力を正規化して有効なモデルを作る() {
        let robot = Robot::new("  RB-40  ", 12, 80).unwrap();
        let mission = Mission::new("  M-400  ", "  図書館  ", 5, 25).unwrap();

        assert_eq!(robot.id().as_str(), "RB-40");
        assert_eq!(robot.capacity().value(), 12);
        assert_eq!(robot.battery().value(), 80);
        assert_eq!(robot.completed_missions(), 0);
        assert_eq!(mission.id().as_str(), "M-400");
        assert_eq!(mission.destination(), "図書館");
        assert_eq!(mission.cargo_weight().value(), 5);
        assert_eq!(mission.required_battery().value(), 25);
    }

    #[test]
    fn 不正なモデルを入口で拒否する() {
        assert_eq!(Robot::new("   ", 10, 80), Err(ModelError::EmptyRobotId));
        assert_eq!(Robot::new("RB-41", 0, 80), Err(ModelError::ZeroCapacity));
        assert_eq!(
            Robot::new("RB-41", 10, 101),
            Err(ModelError::BatteryOutOfRange(101))
        );

        assert_eq!(
            Mission::new("  ", "図書館", 2, 10),
            Err(ModelError::EmptyMissionId)
        );
        assert_eq!(
            Mission::new("M-401", "  ", 2, 10),
            Err(ModelError::EmptyDestination)
        );
        assert_eq!(
            Mission::new("M-401", "図書館", 0, 10),
            Err(ModelError::ZeroCargoWeight)
        );
        assert_eq!(
            Mission::new("M-401", "図書館", 2, 101),
            Err(ModelError::BatteryOutOfRange(101))
        );
    }

    #[test]
    fn 配送可能性の確認では状態を変えない() {
        let robot = Robot::new("RB-42", 10, 60).unwrap();
        let mission = Mission::new("M-402", "研究棟", 10, 60).unwrap();

        assert_eq!(robot.check_mission(&mission), Ok(()));
        assert_eq!(robot.battery().value(), 60);
        assert_eq!(robot.completed_missions(), 0);
    }

    #[test]
    fn 重量超過をバッテリー不足より先に報告する() {
        let robot = Robot::new("RB-43", 5, 10).unwrap();
        let mission = Mission::new("M-403", "学生寮", 6, 20).unwrap();

        assert_eq!(
            robot.check_mission(&mission),
            Err(DispatchReason::CargoTooHeavy {
                capacity: Kilograms(5),
                actual: Kilograms(6),
            })
        );
    }

    #[test]
    fn バッテリー不足を型付きの値で報告する() {
        let robot = Robot::new("RB-44", 10, 29).unwrap();
        let mission = Mission::new("M-404", "食堂", 4, 30).unwrap();

        assert_eq!(
            robot.check_mission(&mission),
            Err(DispatchReason::InsufficientBattery {
                available: BatteryPercent(29),
                required: BatteryPercent(30),
            })
        );
    }

    #[test]
    fn 配送成功時に状態とレシートを更新する() {
        let mut robot = Robot::new("RB-45", 10, 80).unwrap();
        let mission = Mission::new("M-405", "保健センター", 4, 25).unwrap();

        let receipt = robot.dispatch(mission).unwrap();

        assert_eq!(
            receipt,
            DispatchReceipt {
                robot_id: RobotId("RB-45".to_string()),
                mission_id: MissionId("M-405".to_string()),
                destination: "保健センター".to_string(),
                cargo_weight: Kilograms(4),
                battery_remaining: BatteryPercent(55),
                sequence_number: 1,
            }
        );
        assert_eq!(robot.battery().value(), 55);
        assert_eq!(robot.completed_missions(), 1);
    }

    #[test]
    fn 配送失敗時にロボットもミッションも失わない() {
        let mut robot = Robot::new("RB-46", 3, 80).unwrap();
        let mission = Mission::new("M-406", "体育館", 4, 20).unwrap();

        let failure = robot.dispatch(mission).unwrap_err();

        assert_eq!(failure.mission.id().as_str(), "M-406");
        assert_eq!(failure.mission.destination(), "体育館");
        assert_eq!(
            failure.reason,
            DispatchReason::CargoTooHeavy {
                capacity: Kilograms(3),
                actual: Kilograms(4),
            }
        );
        assert_eq!(robot.battery().value(), 80);
        assert_eq!(robot.completed_missions(), 0);
    }

    #[test]
    fn 連続配送と完了数の飽和を処理する() {
        let mut robot = Robot::new("RB-47", 10, 100).unwrap();

        let first = robot
            .dispatch(Mission::new("M-407", "図書館", 2, 10).unwrap())
            .unwrap();
        let second = robot
            .dispatch(Mission::new("M-408", "研究棟", 3, 20).unwrap())
            .unwrap();

        assert_eq!(first.sequence_number, 1);
        assert_eq!(second.sequence_number, 2);
        assert_eq!(robot.battery().value(), 70);

        robot.completed_missions = u32::MAX;
        let final_receipt = robot
            .dispatch(Mission::new("M-409", "食堂", 1, 0).unwrap())
            .unwrap();
        assert_eq!(final_receipt.sequence_number, u32::MAX);
        assert_eq!(robot.completed_missions(), u32::MAX);
    }
}
