#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 02: API に最小のクロージャトレイトを付ける
//!
//! 呼び出し側が渡した判定処理と完了処理を実行する高階関数を作ります
//! starter の型引数 `F` にはまだトレイト境界がありません
//! 処理を何回呼ぶか、環境から値を取り出す必要があるかを考え、最小の境界を追加してください
//!
//! 仕様:
//! - `find_with` は入力順に `predicate` を呼び、最初に `true` となる `Mission` を借用して返す
//! - 最初の一致が見つかった後は残りを調べない
//! - `predicate` は呼び出し回数や履歴を変更できる
//! - `complete_with` は `Mission` とその所有権を `operation` へ一度だけ渡す
//! - `complete_with` の戻り型は呼び出し側が自由に決められる
//! - `Mission` やクロージャを複製しない
//!
//! ヒント:
//! - 複数回呼ばれ、環境を変更できる処理には `FnMut`
//! - 所有値を渡して一度だけ呼ぶ処理には `FnOnce`
//! - `Iterator::find` は最初の一致で走査を終了する

#[derive(Debug, PartialEq, Eq)]
struct Mission {
    id: String,
    priority: u8,
}

impl Mission {
    fn new(id: &str, priority: u8) -> Self {
        Self {
            id: id.to_owned(),
            priority,
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn priority(&self) -> u8 {
        self.priority
    }

    fn into_id(self) -> String {
        self.id
    }
}

fn find_with<F>(missions: &[Mission], predicate: F) -> Option<&Mission> {
    drop(predicate);
    todo!(
        "{} 件を順番に調べるため、F に最小のトレイト境界を追加してください",
        missions.len()
    )
}

fn complete_with<F, R>(mission: Mission, operation: F) -> R {
    drop(operation);
    todo!(
        "任務 {} を一度だけ operation へ渡す境界を追加してください",
        mission.id()
    )
}

fn main() {
    let missions = vec![Mission::new("M-720", 3), Mission::new("M-721", 9)];
    let selected = find_with(&missions, |mission: &Mission| mission.priority() >= 8);
    println!("選択結果: {:?}", selected.map(Mission::id));

    let completed_id: String = complete_with(Mission::new("M-722", 5), |mission: Mission| {
        mission.into_id()
    });
    println!("完了した任務: {completed_id}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn missions() -> Vec<Mission> {
        vec![
            Mission::new("M-701", 2),
            Mission::new("M-702", 8),
            Mission::new("M-703", 10),
            Mission::new("M-704", 1),
        ]
    }

    fn is_critical(mission: &Mission) -> bool {
        mission.priority() >= 10
    }

    #[test]
    fn 状態を変更するpredicateを最初の一致まで呼ぶ() {
        let missions = missions();
        let mut visited = Vec::new();
        let mut calls = 0;

        let found = find_with(&missions, |mission: &Mission| {
            calls += 1;
            visited.push(mission.id().to_owned());
            mission.priority() >= 8
        })
        .expect("一致する任務がある");

        assert_eq!(found.id(), "M-702");
        assert_eq!(calls, 2);
        assert_eq!(visited, ["M-701", "M-702"]);
    }

    #[test]
    fn 最後の要素が一致すると全要素を順番に調べる() {
        let missions = missions();
        let mut visited = Vec::new();

        let found = find_with(&missions, |mission: &Mission| {
            visited.push(mission.id().to_owned());
            mission.id() == "M-704"
        })
        .expect("末尾の任務が一致する");

        assert_eq!(found.id(), "M-704");
        assert_eq!(visited, ["M-701", "M-702", "M-703", "M-704"]);
    }

    #[test]
    fn 一致しなければ全要素を調べてnoneを返す() {
        let missions = missions();
        let mut visited = Vec::new();

        let found = find_with(&missions, |mission: &Mission| {
            visited.push(mission.id().to_owned());
            mission.id() == "M-999"
        });

        assert_eq!(found, None);
        assert_eq!(visited, ["M-701", "M-702", "M-703", "M-704"]);
    }

    #[test]
    fn 空の入力ではpredicateを呼ばない() {
        let missions: Vec<Mission> = Vec::new();
        let mut calls = 0;

        let found = find_with(&missions, |_mission: &Mission| {
            calls += 1;
            true
        });

        assert_eq!(found, None);
        assert_eq!(calls, 0);
    }

    #[test]
    fn 検索結果は元のmissionそのものを参照する() {
        let missions = missions();

        let found = find_with(&missions, |mission: &Mission| mission.id() == "M-703")
            .expect("M-703 が存在する");

        assert!(std::ptr::eq(found, &missions[2]));
        assert_eq!(found.id().as_ptr(), missions[2].id().as_ptr());
    }

    #[test]
    fn 関数アイテムをpredicateとして渡せる() {
        let missions = missions();

        let found = find_with(&missions, is_critical).expect("重大な任務が存在する");

        assert_eq!(found.id(), "M-703");
    }

    #[test]
    fn non_cloneのmissionとpermitを一度だけ消費する() {
        #[derive(Debug, PartialEq, Eq)]
        struct Permit {
            code: String,
        }

        #[derive(Debug, PartialEq, Eq)]
        struct Receipt {
            mission_id: String,
            priority: u8,
            permit: Permit,
        }

        let mission = Mission::new("M-705", 42);
        let mission_id_pointer = mission.id().as_ptr();
        let permit = Permit {
            code: "PERMIT-705".to_owned(),
        };
        let calls = Cell::new(0);
        let calls_ref = &calls;

        let receipt: Receipt = complete_with(mission, move |mission: Mission| {
            calls_ref.set(calls_ref.get() + 1);
            let priority = mission.priority();

            Receipt {
                mission_id: mission.into_id(),
                priority,
                permit,
            }
        });

        assert_eq!(calls.get(), 1);
        assert_eq!(receipt.mission_id, "M-705");
        assert_eq!(receipt.mission_id.as_ptr(), mission_id_pointer);
        assert_eq!(receipt.priority, 42);
        assert_eq!(receipt.permit.code, "PERMIT-705");
    }

    #[test]
    fn 呼び出し側だけが知る戻り型とutf8を扱う() {
        struct LocalSummary {
            label: String,
            priority: u8,
        }

        let summary: LocalSummary = complete_with(
            Mission::new("緊急任務🚚-七", u8::MAX),
            |mission: Mission| {
                let priority = mission.priority();
                LocalSummary {
                    label: format!("完了: {}", mission.into_id()),
                    priority,
                }
            },
        );

        assert_eq!(summary.label, "完了: 緊急任務🚚-七");
        assert_eq!(summary.priority, u8::MAX);
    }
}
