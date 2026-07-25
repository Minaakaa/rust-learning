#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 05: 型安全な配送ミッションを設計する
//!
//! 最終課題では、ロボット ID、ミッション ID、重量、バッテリー残量を別々の型で表し、
//! 不正なモデルをコンストラクタで拒否します。その上で、配送可能性の確認と状態更新を
//! `Robot` の API として実装してください。
//!
//! モデルの不変条件:
//! - ロボット ID、ミッション ID、配送先は `trim` 後に空でない。
//! - 最大積載量と荷物重量は 1 kg 以上。
//! - バッテリー残量と必要残量は 0..=100。
//!
//! 配送の仕様:
//! - 重量超過を先に、次にバッテリー不足を確認する。
//! - 失敗時はロボットを変更せず、ミッションの所有権を `DispatchFailure` で返す。
//! - 成功時は必要残量を消費し、完了数を飽和加算する。
//! - 成功したミッションは `DispatchReceipt` へ変換する。
//! - レシートは独立して保存するため、ロボット ID の `Clone` は意図的な複製である。

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
        todo!("ロボット ID「{value}」を trim して検証してください")
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct MissionId(String);

impl MissionId {
    fn new(value: &str) -> Result<Self, ModelError> {
        todo!("ミッション ID「{value}」を trim して検証してください")
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
        todo!("バッテリー値 {value}% を検証してください")
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
        todo!(
            "ミッション {id}: {destination}、{cargo_weight_kg} kg、必要残量 {required_battery}% を検証してください"
        )
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
        todo!("ロボット {id}: 容量 {capacity_kg} kg、残量 {battery}% を検証してください")
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
        todo!(
            "{} が {} を運べるか重量、残量の順に確認してください",
            self.id.as_str(),
            mission.id.as_str()
        )
    }

    fn dispatch(&mut self, mission: Mission) -> Result<DispatchReceipt, DispatchFailure> {
        todo!(
            "{} で {} を配送するか、所有権付きの失敗を返してください",
            self.id.as_str(),
            mission.id.as_str()
        )
    }
}

fn main() {
    let mut robot = Robot::new("RB-40", 12, 80).expect("固定値は有効");
    let mission = Mission::new("M-400", "図書館", 5, 25).expect("固定値は有効");

    println!("配送結果: {:#?}", robot.dispatch(mission));
    println!("ロボット: {robot:#?}");
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
