//! # 解答 03: TT muncherで小さなDSLを読む

#[derive(Debug, PartialEq, Eq)]
struct Mission {
    id: String,
    destination: String,
    priority: u8,
}

impl Mission {
    fn new(id: impl Into<String>, destination: impl Into<String>, priority: u8) -> Self {
        Self {
            id: id.into(),
            destination: destination.into(),
            priority,
        }
    }
}

macro_rules! mission_plan {
    (@parse $out:ident;) => {};
    (@parse $out:ident; mission $id:ident => $destination:expr, priority $priority:expr; $($rest:tt)*) => {{
        $out.push(Mission::new(stringify!($id), $destination, $priority));
        mission_plan!(@parse $out; $($rest)*);
    }};
    ($($tokens:tt)*) => {{
        #[allow(
            unused_mut,
            reason = "再帰parserが0件から複数件まで同じ出力bufferを使うため"
        )]
        let mut missions = Vec::new();
        mission_plan!(@parse missions; $($tokens)*);
        missions
    }};
}

fn total_priority(missions: &[Mission]) -> u16 {
    missions
        .iter()
        .map(|mission| u16::from(mission.priority))
        .sum()
}

fn main() {
    let missions = mission_plan! {
        mission M1301 => "図書館", priority 80;
        mission M1302 => "工学部", priority 95;
    };
    println!(
        "{}件 / 優先度合計={}",
        missions.len(),
        total_priority(&missions)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dslを入力順にmissionへ変換する() {
        let missions = mission_plan! {
            mission M1 => "図書館", priority 80;
            mission M2 => "工学部", priority 95;
        };

        assert_eq!(missions[0], Mission::new("M1", "図書館", 80));
        assert_eq!(missions[1], Mission::new("M2", "工学部", 95));
    }

    #[test]
    fn 空のdslは空のvectorになる() {
        let missions = mission_plan! {};

        assert!(missions.is_empty());
        assert_eq!(total_priority(&missions), 0);
    }

    #[test]
    fn 末尾semicolonを許可する() {
        let missions = mission_plan! {
            mission M3 => "食堂", priority 0;
        };

        assert_eq!(missions, [Mission::new("M3", "食堂", 0)]);
    }

    #[test]
    fn destinationの式とutf8を保つ() {
        let building = String::from("本郷・工学部🚚");
        let missions = mission_plan! {
            mission M4 => building.clone(), priority 255;
        };

        assert_eq!(missions[0].destination, building);
        assert_eq!(missions[0].priority, u8::MAX);
    }

    #[test]
    fn priorityの合計を集計できる() {
        let missions = mission_plan! {
            mission M5 => "A", priority 1 + 2;
            mission M6 => "B", priority 100;
        };

        assert_eq!(total_priority(&missions), 103);
    }

    #[test]
    fn identは文字列idとして保存される() {
        let missions = mission_plan! {
            mission Robot_1307 => "研究棟", priority 7;
        };

        assert_eq!(missions[0].id, "Robot_1307");
    }
}
