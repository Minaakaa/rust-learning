//! 問題 02 の解答例。

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
    match request {
        Request::EmergencyStop => Action::Stop("緊急停止"),
        _ if battery_percent > 100 => Action::Stop("バッテリー値が不正です"),
        _ if battery_percent <= 19 => Action::Charge(battery_percent),
        Request::Laboratory { floor: 0 } => Action::Stop("0階は指定できません"),
        Request::Laboratory {
            floor: floor @ 4..=u8::MAX,
        } => Action::UseElevator(floor),
        Request::Laboratory { floor: 1..=3 } => Action::DriveTo {
            destination: "研究棟",
            speed_m_per_min: 40,
        },
        Request::Library if is_raining => Action::DriveTo {
            destination: "図書館",
            speed_m_per_min: 30,
        },
        Request::Library => Action::DriveTo {
            destination: "図書館",
            speed_m_per_min: 60,
        },
        Request::ReturnToBase => Action::DriveTo {
            destination: "充電基地",
            speed_m_per_min: 50,
        },
    }
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
