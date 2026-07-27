#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 03: トレイト境界で配送候補を選ぶ
//!
//! 配送依頼と安全警報は別々の型ですが、どちらにも識別子と優先度があります
//! 共通する振る舞いを `Candidate` トレイトで表し、具体的な型を決めずに候補を
//! 整形したり比較したりできる関数を完成させてください
//!
//! 仕様:
//! - `DeliveryRequest` と `SafetyAlert` に `Candidate` を実装する
//! - `format_candidate` の引数には引数位置の `impl Trait` を使う
//! - `choose_higher` は型引数 `T` と `where T: Candidate` を使う
//! - `choose_higher` の2引数は同じ具体型とし、優先度が高い値を返す
//! - 優先度が同じなら最初の値を返す
//! - 候補を所有権ごと受け取り、`Clone` せず選んだ値を返す

trait Candidate {
    fn id(&self) -> &str;
    fn priority(&self) -> u8;
}

#[derive(Debug, PartialEq, Eq)]
struct DeliveryRequest {
    request_id: String,
    destination: String,
    priority: u8,
}

impl DeliveryRequest {
    fn new(request_id: &str, destination: &str, priority: u8) -> Self {
        Self {
            request_id: request_id.to_string(),
            destination: destination.to_string(),
            priority,
        }
    }
}

impl Candidate for DeliveryRequest {
    fn id(&self) -> &str {
        todo!(
            "配送依頼 {} の ID を借用して返してください",
            self.request_id
        )
    }

    fn priority(&self) -> u8 {
        todo!("配送依頼 {} の優先度を返してください", self.request_id)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SafetyAlert {
    alert_code: String,
    area: String,
    severity: u8,
}

impl SafetyAlert {
    fn new(alert_code: &str, area: &str, severity: u8) -> Self {
        Self {
            alert_code: alert_code.to_string(),
            area: area.to_string(),
            severity,
        }
    }
}

impl Candidate for SafetyAlert {
    fn id(&self) -> &str {
        todo!(
            "安全警報 {} の ID を借用して返してください",
            self.alert_code
        )
    }

    fn priority(&self) -> u8 {
        todo!("安全警報 {} の優先度を返してください", self.alert_code)
    }
}

fn format_candidate(candidate: &impl Candidate) -> String {
    todo!(
        "候補 {} を優先度 {} とともに整形してください",
        candidate.id(),
        candidate.priority()
    )
}

fn choose_higher<T>(first: T, second: T) -> T
where
    T: Candidate,
{
    todo!(
        "候補 {} と {} を比較し、所有権を持つ一方を返してください",
        first.id(),
        second.id()
    )
}

fn main() {
    let request = DeliveryRequest::new("REQ-501", "図書館", 80);
    let alert = SafetyAlert::new("SAFE-12", "工学部A棟", 95);

    println!("配送候補: {}", format_candidate(&request));
    println!("安全候補: {}", format_candidate(&alert));

    let selected = choose_higher(request, DeliveryRequest::new("REQ-502", "体育館", 90));
    println!("選択結果: {}", format_candidate(&selected));
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NonCloneCandidate {
        id: String,
        priority: u8,
        payload: String,
    }

    impl Candidate for NonCloneCandidate {
        fn id(&self) -> &str {
            &self.id
        }

        fn priority(&self) -> u8 {
            self.priority
        }
    }

    #[test]
    fn 配送依頼をcandidateとして扱う() {
        let request = DeliveryRequest::new("REQ-510", "図書館", 80);

        assert_eq!(request.id(), "REQ-510");
        assert_eq!(request.priority(), 80);
        assert_eq!(request.destination, "図書館");
    }

    #[test]
    fn 安全警報をcandidateとして扱う() {
        let alert = SafetyAlert::new("SAFE-20", "実験棟", 200);

        assert_eq!(alert.id(), "SAFE-20");
        assert_eq!(alert.priority(), 200);
        assert_eq!(alert.area, "実験棟");
    }

    #[test]
    fn impl_traitで異なる型を同じ形式に整形する() {
        let request = DeliveryRequest::new("REQ-511", "学生寮", 42);
        let alert = SafetyAlert::new("SAFE-21", "北門", 210);

        assert_eq!(format_candidate(&request), "REQ-511（優先度 42）");
        assert_eq!(format_candidate(&alert), "SAFE-21（優先度 210）");
    }

    #[test]
    fn 最初の候補の優先度が高ければ最初を返す() {
        let selected = choose_higher(
            DeliveryRequest::new("REQ-HIGH", "図書館", 120),
            DeliveryRequest::new("REQ-LOW", "体育館", 119),
        );

        assert_eq!(selected.id(), "REQ-HIGH");
        assert_eq!(selected.destination, "図書館");
    }

    #[test]
    fn 二番目の候補の優先度が高ければ二番目を返す() {
        let selected = choose_higher(
            SafetyAlert::new("SAFE-LOW", "西門", 30),
            SafetyAlert::new("SAFE-HIGH", "東門", 31),
        );

        assert_eq!(selected.id(), "SAFE-HIGH");
        assert_eq!(selected.area, "東門");
    }

    #[test]
    fn 同じ優先度なら最初の候補を返す() {
        let selected = choose_higher(
            DeliveryRequest::new("REQ-FIRST", "本郷門", 100),
            DeliveryRequest::new("REQ-SECOND", "赤門", 100),
        );

        assert_eq!(selected.id(), "REQ-FIRST");
        assert_eq!(selected.destination, "本郷門");
    }

    #[test]
    fn 優先度の最小値と最大値を比較する() {
        let selected = choose_higher(
            SafetyAlert::new("SAFE-MIN", "南門", 0),
            SafetyAlert::new("SAFE-MAX", "北門", u8::MAX),
        );

        assert_eq!(selected.id(), "SAFE-MAX");
        assert_eq!(selected.priority(), 255);
    }

    #[test]
    fn cloneなしで所有値を選んで返す() {
        let selected = choose_higher(
            NonCloneCandidate {
                id: "NC-LOW".to_string(),
                priority: 70,
                payload: "選ばれない値".to_string(),
            },
            NonCloneCandidate {
                id: "NC-HIGH".to_string(),
                priority: 71,
                payload: "引き渡された値".to_string(),
            },
        );

        assert_eq!(selected.id(), "NC-HIGH");
        assert_eq!(selected.payload, "引き渡された値");
    }
}
