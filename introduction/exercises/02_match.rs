#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 02: match で次の行動を決める
//!
//! 配送依頼とバッテリー残量から、ロボットの次の行動を 1 つ決めます。
//! `plan_action` を 1 つの `match` 式を中心に実装してください。
//!
//! 判定は上から優先します。
//! 1. 緊急停止は、ほかの条件より常に優先する。
//! 2. 100 を超えるバッテリー値は不正として停止する。
//! 3. 緊急停止以外で残量が 0..=19 なら充電する。
//! 4. 研究棟の 0 階は不正、1..=3 階は速度 40、4 階以上ではエレベーターを使う。
//! 5. 図書館便は通常速度 60、雨の日は速度 30 とする。
//! 6. 充電基地へ戻るときは速度 50 とする。
//!
//! パターンによる分解、範囲パターン、`if` ガード、`_` を使うと簡潔に書けます。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Request {
    Library,
    Laboratory { floor: u8 },
    ReturnToBase,
    EmergencyStop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
    DriveTo {
        destination: &'static str,
        speed_m_per_min: u16,
    },
    UseElevator(u8),
    Charge(u8),
    Stop(&'static str),
}

fn plan_action(request: Request, battery_percent: u8, is_raining: bool) -> Action {
    todo!(
        "Request を網羅的に match してください: {request:?}, battery={battery_percent}, rain={is_raining}"
    )
}

fn main() {
    let action = plan_action(Request::Library, 72, true);
    println!("ロボットの次の行動: {action:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 緊急停止を最優先する() {
        assert_eq!(
            plan_action(Request::EmergencyStop, 5, false),
            Action::Stop("緊急停止")
        );
        assert_eq!(
            plan_action(Request::EmergencyStop, 101, false),
            Action::Stop("緊急停止")
        );
    }

    #[test]
    fn 不正なバッテリー値なら停止する() {
        assert_eq!(
            plan_action(Request::ReturnToBase, 101, false),
            Action::Stop("バッテリー値が不正です")
        );
        assert_eq!(
            plan_action(Request::Library, 101, true),
            Action::Stop("バッテリー値が不正です")
        );
    }

    #[test]
    fn 残量が少なければ現在値を持って充電する() {
        assert_eq!(plan_action(Request::Library, 19, false), Action::Charge(19));
        assert_eq!(
            plan_action(Request::Laboratory { floor: 4 }, 10, false),
            Action::Charge(10)
        );
    }

    #[test]
    fn バッテリーの境界値では通常走行できる() {
        assert_eq!(
            plan_action(Request::Library, 20, false),
            Action::DriveTo {
                destination: "図書館",
                speed_m_per_min: 60,
            }
        );
        assert_eq!(
            plan_action(Request::ReturnToBase, 100, false),
            Action::DriveTo {
                destination: "充電基地",
                speed_m_per_min: 50,
            }
        );
    }

    #[test]
    fn 図書館への速度を天候で変える() {
        assert_eq!(
            plan_action(Request::Library, 80, true),
            Action::DriveTo {
                destination: "図書館",
                speed_m_per_min: 30,
            }
        );
        assert_eq!(
            plan_action(Request::Library, 80, false),
            Action::DriveTo {
                destination: "図書館",
                speed_m_per_min: 60,
            }
        );
    }

    #[test]
    fn 研究棟の階数を分解して行動を選ぶ() {
        assert_eq!(
            plan_action(Request::Laboratory { floor: 0 }, 80, false),
            Action::Stop("0階は指定できません")
        );
        assert_eq!(
            plan_action(Request::Laboratory { floor: 3 }, 80, false),
            Action::DriveTo {
                destination: "研究棟",
                speed_m_per_min: 40,
            }
        );
        assert_eq!(
            plan_action(Request::Laboratory { floor: 4 }, 80, false),
            Action::UseElevator(4)
        );
    }

    #[test]
    fn 基地へ戻る() {
        assert_eq!(
            plan_action(Request::ReturnToBase, 50, false),
            Action::DriveTo {
                destination: "充電基地",
                speed_m_per_min: 50,
            }
        );
    }
}
