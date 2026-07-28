#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 04: 独立した借用で任務を割り当てる
//!
//! 配送任務とロボットは別々の場所で管理され、異なる期間だけ有効です
//! `Assignment` が両方を複製せず借用し、それぞれ本来のライフタイムで返せるようにしてください
//!
//! 仕様:
//! - `Assignment` は `Mission` と `Robot` を借用し、所有しない
//! - 2つの参照には独立したライフタイム引数を使う
//! - オフラインのロボットには `RobotOffline` を返す
//! - オンラインでも重量が積載量を超える場合は `OverCapacity` を返す
//! - `mission` と `robot` は、`Assignment` 自体の借用期間ではなく保存した参照の期間で返す
//! - `remaining_capacity_kg` は割り当て後の残り積載量を返す
//! - `Mission` と `Robot` を `clone` しない
//!
//! ヒント:
//! - starter の所有フィールドと戻り値を、参照へ変更する必要がある
//! - `Assignment<'mission, 'robot>` のように参照元ごとに型引数を分ける
//! - `&self` の省略規則に任せると、戻り値が `Assignment` の借用へ結び付く

#[derive(Debug, Clone, PartialEq, Eq)]
struct Mission {
    id: String,
    destination: String,
    weight_kg: u16,
}

impl Mission {
    fn new(id: &str, destination: &str, weight_kg: u16) -> Self {
        Self {
            id: id.to_string(),
            destination: destination.to_string(),
            weight_kg,
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn destination(&self) -> &str {
        &self.destination
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Robot {
    name: String,
    capacity_kg: u16,
    online: bool,
}

impl Robot {
    fn new(name: &str, capacity_kg: u16, online: bool) -> Self {
        Self {
            name: name.to_string(),
            capacity_kg,
            online,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, PartialEq, Eq)]
enum AssignmentError {
    RobotOffline,
    OverCapacity { weight_kg: u16, capacity_kg: u16 },
}

#[derive(Debug, PartialEq, Eq)]
struct Assignment {
    mission: Mission,
    robot: Robot,
}

impl Assignment {
    fn new(mission: &Mission, robot: &Robot) -> Result<Self, AssignmentError> {
        let _ = (mission, robot);
        todo!("任務とロボットを複製せず割り当ててください")
    }

    fn mission(&self) -> Mission {
        todo!(
            "割り当てた任務 {} を元のライフタイムで返してください",
            self.mission.id
        )
    }

    fn robot(&self) -> Robot {
        todo!(
            "割り当てたロボット {} を元のライフタイムで返してください",
            self.robot.name
        )
    }

    fn remaining_capacity_kg(&self) -> u16 {
        todo!(
            "ロボット {} の残り積載量を計算してください",
            self.robot.name
        )
    }
}

fn main() {
    let mission = Mission::new("M-604", "工学部2号館", 18);
    let robot = Robot::new("Aoi", 30, true);
    let assignment = Assignment::new(&mission, &robot).expect("割り当て可能なはずです");

    println!(
        "{} が {} へ配送します（残り {} kg）",
        assignment.robot().name(),
        assignment.mission().destination(),
        assignment.remaining_capacity_kg()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 任務とロボットを複製せず借用する() {
        let mission = Mission::new("M-604", "工学部2号館", 18);
        let robot = Robot::new("Aoi", 30, true);

        let assignment = Assignment::new(&mission, &robot).unwrap();
        let selected_mission = assignment.mission();
        let selected_robot = assignment.robot();

        assert_eq!(selected_mission.id(), "M-604");
        assert_eq!(selected_robot.name(), "Aoi");
        assert_eq!(selected_mission.id().as_ptr(), mission.id().as_ptr());
        assert_eq!(selected_robot.name().as_ptr(), robot.name().as_ptr());
        assert_eq!(assignment.remaining_capacity_kg(), 12);
    }

    #[test]
    fn 積載量ちょうどの任務を割り当てる() {
        let mission = Mission::new("M-605", "理学部1号館", 25);
        let robot = Robot::new("Rin", 25, true);

        let assignment = Assignment::new(&mission, &robot).unwrap();

        assert_eq!(assignment.remaining_capacity_kg(), 0);
    }

    #[test]
    fn 積載量を超える任務を拒否する() {
        let mission = Mission::new("M-606", "図書館", 31);
        let robot = Robot::new("Sora", 30, true);

        assert_eq!(
            Assignment::new(&mission, &robot),
            Err(AssignmentError::OverCapacity {
                weight_kg: 31,
                capacity_kg: 30,
            })
        );
    }

    #[test]
    fn オフラインを重量判定より先に拒否する() {
        let mission = Mission::new("M-607", "情報基盤センター", 99);
        let robot = Robot::new("Haku", 10, false);

        assert_eq!(
            Assignment::new(&mission, &robot),
            Err(AssignmentError::RobotOffline)
        );
    }

    #[test]
    fn 短命なrobotとassignmentの後でもmissionを使える() {
        let mission = Mission::new("M-608", "総合研究棟", 8);

        let selected_mission = {
            let robot = Robot::new("Yui", 12, true);
            let assignment = Assignment::new(&mission, &robot).unwrap();
            assignment.mission()
        };

        assert_eq!(selected_mission.destination(), "総合研究棟");
        assert_eq!(selected_mission.id().as_ptr(), mission.id().as_ptr());
    }

    #[test]
    fn 短命なmissionとassignmentの後でもrobotを使える() {
        let robot = Robot::new("Mio", 40, true);

        let selected_robot = {
            let mission = Mission::new("M-609", "本郷郵便局", 4);
            let assignment = Assignment::new(&mission, &robot).unwrap();
            assignment.robot()
        };

        assert_eq!(selected_robot.name(), "Mio");
        assert_eq!(selected_robot.name().as_ptr(), robot.name().as_ptr());
    }

    #[test]
    fn 重量ゼロの任務では積載量がすべて残る() {
        let mission = Mission::new("M-610", "管制室", 0);
        let robot = Robot::new("Kai", u16::MAX, true);

        let assignment = Assignment::new(&mission, &robot).unwrap();

        assert_eq!(assignment.remaining_capacity_kg(), u16::MAX);
    }

    #[test]
    fn 日本語と絵文字を借用したまま保持する() {
        let mission = Mission::new("任務🚚-01", "工学部Ａ棟🔧", 7);
        let robot = Robot::new("葵🤖", 20, true);

        let assignment = Assignment::new(&mission, &robot).unwrap();

        assert_eq!(assignment.mission().destination(), "工学部Ａ棟🔧");
        assert_eq!(assignment.robot().name(), "葵🤖");
    }
}
