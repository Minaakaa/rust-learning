#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 01: 構造体で配送依頼を組み立てる
//!
//! 配送依頼を `(String, String, String, String)` のようなタプルで表すと、各値の意味を
//! 順番から推測しなければなりません。名前付きフィールドと入れ子の構造体を使い、
//! 読んだだけで意味が分かるデータモデルを作ってください。
//!
//! 仕様:
//! - 文字列は構造体が所有する `String` に変換する。
//! - 新しい依頼の `notes` は空にする。
//! - `redirect` は配送先だけを置き換え、ID、集荷場所、メモを保つ。
//! - `add_note` は既存のメモの末尾へ追加する。
//! - `redirect` では構造体更新記法 `..request` を試してみる。

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
    todo!("建物 {building}、部屋 {room} の CampusLocation を作ってください")
}

fn create_request(
    id: &str,
    pickup: CampusLocation,
    destination: CampusLocation,
) -> DeliveryRequest {
    todo!(
        "依頼 {id} を {} から {} へ送る構造体を作ってください",
        pickup.building,
        destination.building
    )
}

fn redirect(request: DeliveryRequest, destination: CampusLocation) -> DeliveryRequest {
    todo!(
        "依頼 {} の配送先を {} へ置き換えてください",
        request.id,
        destination.building
    )
}

fn add_note(mut request: DeliveryRequest, note: &str) -> DeliveryRequest {
    // 未実装でも `mut` が必要な形を保つための行。実装時に削除してよい。
    let _ = &mut request;
    todo!(
        "依頼 {} のメモ末尾へ「{note}」を追加してください",
        request.id
    )
}

fn main() {
    let pickup = location("工学部A棟", "A-101");
    let destination = location("図書館", "受付");
    let request = create_request("REQ-001", pickup, destination);
    let request = add_note(request, "取扱注意");

    println!("配送依頼: {request:#?}");
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
