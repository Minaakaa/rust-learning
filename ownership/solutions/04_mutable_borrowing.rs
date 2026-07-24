//! 問題 04 の解答例。

#[derive(Debug, PartialEq, Eq)]
struct Robot {
    id: String,
    battery_percent: u8,
    cargo: Vec<String>,
    events: Vec<String>,
}

impl Robot {
    fn new(id: &str, battery_percent: u8, cargo: Vec<String>) -> Self {
        Self {
            id: id.to_string(),
            battery_percent,
            cargo,
            events: Vec::new(),
        }
    }
}

fn recharge(robot: &mut Robot, amount: u8) -> u8 {
    let before = robot.battery_percent;
    robot.battery_percent = robot.battery_percent.saturating_add(amount).min(100);
    robot.battery_percent - before
}

fn record_event(robot: &mut Robot, event: &str) {
    robot.events.push(event.to_string());
}

fn prepare_for_shift(robot: &mut Robot) -> u8 {
    let added = recharge(robot, 100);
    record_event(robot, &format!("シフト準備: {added}% 充電"));
    added
}

fn transfer_front(source: &mut Robot, destination: &mut Robot) -> bool {
    if source.cargo.is_empty() {
        return false;
    }

    let parcel = source.cargo.remove(0);
    let event = format!("荷物 {parcel} を引き渡し");
    destination.cargo.push(parcel);
    record_event(source, &event);
    record_event(destination, &event);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 最大100パーセントまで充電する() {
        let mut robot = Robot::new("RB-20", 92, Vec::new());

        let added = recharge(&mut robot, 15);

        assert_eq!(added, 8);
        assert_eq!(robot.battery_percent, 100);
        assert!(robot.events.is_empty());
    }

    #[test]
    fn 可変借用でイベントを追記する() {
        let mut robot = Robot::new("RB-21", 80, Vec::new());

        record_event(&mut robot, "点検完了");
        record_event(&mut robot, "出発");

        assert_eq!(robot.id, "RB-21");
        assert_eq!(robot.events, vec!["点検完了", "出発"]);
    }

    #[test]
    fn 同じ可変参照を順番に再借用する() {
        let mut robot = Robot::new("RB-22", 65, Vec::new());

        let added = prepare_for_shift(&mut robot);

        assert_eq!(added, 35);
        assert_eq!(robot.battery_percent, 100);
        assert_eq!(robot.events, vec!["シフト準備: 35% 充電"]);
    }

    #[test]
    fn stringを複製せず別のロボットへ移す() {
        let mut source = Robot::new("RB-23", 70, vec!["P-230".to_string(), "P-231".to_string()]);
        let mut destination = Robot::new("RB-24", 80, vec!["P-240".to_string()]);

        let transferred = transfer_front(&mut source, &mut destination);

        assert!(transferred);
        assert_eq!(source.cargo, vec!["P-231"]);
        assert_eq!(destination.cargo, vec!["P-240", "P-230"]);
        assert_eq!(source.events, vec!["荷物 P-230 を引き渡し"]);
        assert_eq!(destination.events, vec!["荷物 P-230 を引き渡し"]);
    }

    #[test]
    fn 荷物がなければ何も変更しない() {
        let mut source = Robot::new("RB-25", 70, Vec::new());
        let mut destination = Robot::new("RB-26", 80, vec!["P-260".to_string()]);

        let transferred = transfer_front(&mut source, &mut destination);

        assert!(!transferred);
        assert!(source.cargo.is_empty());
        assert!(source.events.is_empty());
        assert_eq!(destination.cargo, vec!["P-260"]);
        assert!(destination.events.is_empty());
    }
}
