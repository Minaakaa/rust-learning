//! 問題 05 の解答例。

#[derive(Debug, PartialEq, Eq)]
struct Parcel {
    id: String,
    destination: String,
    weight_kg: u16,
}

impl Parcel {
    fn new(id: &str, destination: &str, weight_kg: u16) -> Self {
        Self {
            id: id.to_string(),
            destination: destination.to_string(),
            weight_kg,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Depot {
    waiting: Vec<Parcel>,
    delivered: Vec<Parcel>,
}

impl Depot {
    fn new(waiting: Vec<Parcel>) -> Self {
        Self {
            waiting,
            delivered: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Robot {
    id: String,
    capacity_kg: u16,
    cargo: Vec<Parcel>,
}

impl Robot {
    fn new(id: &str, capacity_kg: u16) -> Self {
        Self {
            id: id.to_string(),
            capacity_kg,
            cargo: Vec::new(),
        }
    }
}

fn cargo_weight(robot: &Robot) -> u32 {
    robot
        .cargo
        .iter()
        .map(|parcel| u32::from(parcel.weight_kg))
        .sum()
}

fn load_for_destination(depot: &mut Depot, robot: &mut Robot, destination: &str) -> usize {
    let mut remaining = Vec::with_capacity(depot.waiting.len());
    let mut current_weight = cargo_weight(robot);
    let capacity = u32::from(robot.capacity_kg);
    let mut loaded = 0;

    for parcel in depot.waiting.drain(..) {
        let weight = u32::from(parcel.weight_kg);
        let fits = weight <= capacity.saturating_sub(current_weight);

        if parcel.destination == destination && fits {
            current_weight += weight;
            robot.cargo.push(parcel);
            loaded += 1;
        } else {
            remaining.push(parcel);
        }
    }

    depot.waiting = remaining;
    loaded
}

fn deliver_all(robot: &mut Robot, depot: &mut Depot) -> usize {
    let delivered = robot.cargo.len();
    depot.delivered.append(&mut robot.cargo);
    delivered
}

fn parcel_ids(parcels: &[Parcel]) -> Vec<&str> {
    parcels.iter().map(|parcel| parcel.id.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn depot() -> Depot {
        Depot::new(vec![
            Parcel::new("P-1", "図書館", 3),
            Parcel::new("P-2", "研究棟", 4),
            Parcel::new("P-3", "図書館", 6),
            Parcel::new("P-4", "図書館", 2),
        ])
    }

    #[test]
    fn 荷物を借用して積載重量を計算する() {
        let mut robot = Robot::new("RB-30", 20);
        robot.cargo.push(Parcel::new("P-A", "食堂", 7));
        robot.cargo.push(Parcel::new("P-B", "体育館", 5));

        assert_eq!(cargo_weight(&robot), 12);
        assert_eq!(parcel_ids(&robot.cargo), vec!["P-A", "P-B"]);
    }

    #[test]
    fn 配送先と容量に合う荷物だけを順序どおり積む() {
        let mut depot = depot();
        let mut robot = Robot::new("RB-31", 7);

        let loaded = load_for_destination(&mut depot, &mut robot, "図書館");

        assert_eq!(loaded, 2);
        assert_eq!(cargo_weight(&robot), 5);
        assert_eq!(parcel_ids(&robot.cargo), vec!["P-1", "P-4"]);
        assert_eq!(parcel_ids(&depot.waiting), vec!["P-2", "P-3"]);
        assert!(depot.delivered.is_empty());
    }

    #[test]
    fn 既存の荷物を残して残り容量だけを使う() {
        let mut depot = Depot::new(vec![
            Parcel::new("P-5", "図書館", 5),
            Parcel::new("P-6", "図書館", 1),
        ]);
        let mut robot = Robot::new("RB-32", 10);
        robot.cargo.push(Parcel::new("P-0", "図書館", 5));

        let loaded = load_for_destination(&mut depot, &mut robot, "図書館");

        assert_eq!(loaded, 1);
        assert_eq!(parcel_ids(&robot.cargo), vec!["P-0", "P-5"]);
        assert_eq!(parcel_ids(&depot.waiting), vec!["P-6"]);
        assert_eq!(cargo_weight(&robot), 10);
    }

    #[test]
    fn 全荷物を配送済み記録の末尾へ移す() {
        let mut depot = Depot::new(Vec::new());
        depot.delivered.push(Parcel::new("P-D", "保健センター", 1));
        let mut robot = Robot::new("RB-33", 10);
        robot.cargo.push(Parcel::new("P-7", "図書館", 3));
        robot.cargo.push(Parcel::new("P-8", "図書館", 2));

        let delivered = deliver_all(&mut robot, &mut depot);

        assert_eq!(delivered, 2);
        assert!(robot.cargo.is_empty());
        assert_eq!(parcel_ids(&depot.delivered), vec!["P-D", "P-7", "P-8"]);
    }

    #[test]
    fn 一連の処理で荷物を重複も紛失もさせない() {
        let mut depot = depot();
        let mut robot = Robot::new("RB-34", 7);

        load_for_destination(&mut depot, &mut robot, "図書館");
        deliver_all(&mut robot, &mut depot);
        load_for_destination(&mut depot, &mut robot, "研究棟");
        deliver_all(&mut robot, &mut depot);

        assert!(robot.cargo.is_empty());
        assert_eq!(parcel_ids(&depot.waiting), vec!["P-3"]);
        assert_eq!(parcel_ids(&depot.delivered), vec!["P-1", "P-4", "P-2"]);
        assert_eq!(depot.waiting.len() + depot.delivered.len(), 4);
    }

    #[test]
    fn 空の状態でも処理できる() {
        let mut depot = Depot::new(Vec::new());
        let mut robot = Robot::new("RB-35", 10);

        assert_eq!(load_for_destination(&mut depot, &mut robot, "図書館"), 0);
        assert_eq!(deliver_all(&mut robot, &mut depot), 0);
        assert!(parcel_ids(&depot.waiting).is_empty());
        assert!(parcel_ids(&depot.delivered).is_empty());
    }
}
