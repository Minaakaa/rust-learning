//! 問題 04 の解答例

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
struct Assignment<'mission, 'robot> {
    mission: &'mission Mission,
    robot: &'robot Robot,
}

impl<'mission, 'robot> Assignment<'mission, 'robot> {
    fn new(mission: &'mission Mission, robot: &'robot Robot) -> Result<Self, AssignmentError> {
        if !robot.online {
            return Err(AssignmentError::RobotOffline);
        }

        if mission.weight_kg > robot.capacity_kg {
            return Err(AssignmentError::OverCapacity {
                weight_kg: mission.weight_kg,
                capacity_kg: robot.capacity_kg,
            });
        }

        Ok(Self { mission, robot })
    }

    fn mission(&self) -> &'mission Mission {
        self.mission
    }

    fn robot(&self) -> &'robot Robot {
        self.robot
    }

    fn remaining_capacity_kg(&self) -> u16 {
        self.robot.capacity_kg - self.mission.weight_kg
    }
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
