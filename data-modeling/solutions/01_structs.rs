//! 問題 01 の解答例。

#[derive(Debug, Clone, PartialEq, Eq)]
struct CampusLocation {
    building: String,
    room: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeliveryRequest {
    id: String,
    pickup: CampusLocation,
    destination: CampusLocation,
    notes: Vec<String>,
}

fn location(building: &str, room: &str) -> CampusLocation {
    CampusLocation {
        building: building.to_string(),
        room: room.to_string(),
    }
}

fn create_request(
    id: &str,
    pickup: CampusLocation,
    destination: CampusLocation,
) -> DeliveryRequest {
    DeliveryRequest {
        id: id.to_string(),
        pickup,
        destination,
        notes: Vec::new(),
    }
}

fn redirect(request: DeliveryRequest, destination: CampusLocation) -> DeliveryRequest {
    DeliveryRequest {
        destination,
        ..request
    }
}

fn add_note(mut request: DeliveryRequest, note: &str) -> DeliveryRequest {
    request.notes.push(note.to_string());
    request
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 場所を名前付きフィールドで表す() {
        assert_eq!(
            location("工学部A棟", "A-101"),
            CampusLocation {
                building: "工学部A棟".to_string(),
                room: "A-101".to_string(),
            }
        );
    }

    #[test]
    fn 場所を入れ子にした配送依頼を作る() {
        let request = create_request(
            "REQ-001",
            location("工学部A棟", "A-101"),
            location("図書館", "受付"),
        );

        assert_eq!(request.id, "REQ-001");
        assert_eq!(request.pickup, location("工学部A棟", "A-101"));
        assert_eq!(request.destination, location("図書館", "受付"));
        assert!(request.notes.is_empty());
    }

    #[test]
    fn 配送先だけを変更してほかのフィールドを保つ() {
        let request = add_note(
            create_request(
                "REQ-002",
                location("学生寮", "管理室"),
                location("食堂", "厨房"),
            ),
            "冷蔵品",
        );

        let redirected = redirect(request, location("保健センター", "受付"));

        assert_eq!(redirected.id, "REQ-002");
        assert_eq!(redirected.pickup, location("学生寮", "管理室"));
        assert_eq!(redirected.destination, location("保健センター", "受付"));
        assert_eq!(redirected.notes, vec!["冷蔵品"]);
    }

    #[test]
    fn メモを追加順に保持する() {
        let request = create_request(
            "REQ-003",
            location("研究棟", "R-204"),
            location("体育館", "器具室"),
        );
        let request = add_note(add_note(request, "壊れ物"), "16時まで");

        assert_eq!(request.notes, vec!["壊れ物", "16時まで"]);
        assert_eq!(request.destination.building, "体育館");
    }
}
