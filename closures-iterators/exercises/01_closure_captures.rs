#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 01: 環境をキャプチャするクロージャ
//!
//! 配送条件をクロージャとして組み立て、ミッションの選択処理へ渡します
//! 関数とは異なり、クロージャは定義した環境の値を借用したり所有したりできます
//!
//! 仕様:
//! - `make_dispatch_filter` は配送先の接頭辞と最小優先度を所有するクロージャを返す
//! - 戻り値を `impl Fn(&Mission) -> bool` に変更する
//! - `move` で `required_area` をクロージャへ移し、関数終了後も使えるようにする
//! - 配送先が接頭辞で始まり、かつ優先度が境界値以上なら `true` を返す
//! - `select_missions` に `Fn(&Mission) -> bool` の境界を追加する
//! - 条件に合う元の `Mission` への参照を入力順で返す
//! - `Mission` や文字列を複製しない
//!
//! ヒント:
//! - `move` はキャプチャ方法を変えるが、それだけで `FnOnce` になるわけではない
//! - 所有した文字列をクロージャ本体で読むだけなら同じクロージャを何度でも呼べる
//! - `filter` の後に `collect` すると参照の `Vec` を作れる

#[derive(Debug, PartialEq, Eq)]
struct Mission {
    id: String,
    destination: String,
    priority: u8,
    distance_m: u32,
}

impl Mission {
    fn new(id: &str, destination: &str, priority: u8, distance_m: u32) -> Self {
        Self {
            id: id.to_string(),
            destination: destination.to_string(),
            priority,
            distance_m,
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn destination(&self) -> &str {
        &self.destination
    }

    fn priority(&self) -> u8 {
        self.priority
    }

    fn distance_m(&self) -> u32 {
        self.distance_m
    }
}

fn make_dispatch_filter(required_area: String, minimum_priority: u8) -> fn(&Mission) -> bool {
    todo!(
        "地域 {required_area:?} かつ優先度 {minimum_priority} 以上を選ぶクロージャを返してください"
    )
}

fn select_missions<P>(missions: &[Mission], predicate: P) -> Vec<&Mission> {
    let _ = &predicate;
    todo!(
        "{} 件へクロージャを適用し、元の Mission を借用してください",
        missions.len()
    )
}

fn main() {
    let missions = vec![
        Mission::new("M-701", "本郷・工学部2号館", 8, 320),
        Mission::new("M-702", "駒場・研究棟", 9, 850),
    ];
    let rule = make_dispatch_filter(String::from("本郷"), 7);

    for mission in select_missions(&missions, rule) {
        println!("選択: {} → {}", mission.id(), mission.destination());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn missions() -> Vec<Mission> {
        vec![
            Mission::new("M-701", "本郷・工学部2号館", 7, 320),
            Mission::new("M-702", "本郷・総合図書館", 4, 150),
            Mission::new("M-703", "駒場・研究棟", 9, 900),
            Mission::new("M-704", "本郷・学生食堂", 8, 80),
        ]
    }

    fn call_twice<F>(predicate: &F, mission: &Mission) -> (bool, bool)
    where
        F: Fn(&Mission) -> bool,
    {
        (predicate(mission), predicate(mission))
    }

    fn is_high_priority(mission: &Mission) -> bool {
        mission.priority() >= 8
    }

    #[test]
    fn 地域と優先度の両方で判定する() {
        let missions = missions();
        let rule = make_dispatch_filter(String::from("本郷"), 7);

        assert!(rule(&missions[0]));
        assert!(!rule(&missions[1]));
        assert!(!rule(&missions[2]));
        assert!(rule(&missions[3]));
    }

    #[test]
    fn 所有した地域を関数終了後も使いfnとして繰り返し呼べる() {
        let rule = {
            let area = String::from("駒場");
            make_dispatch_filter(area, 9)
        };
        let mission = Mission::new("M-705", "駒場・数理科学研究科", 9, 410);

        assert_eq!(call_twice(&rule, &mission), (true, true));
    }

    #[test]
    fn 選択結果は入力順を保ち元のmissionを借用する() {
        let missions = missions();

        let selected = select_missions(&missions, |mission: &Mission| mission.priority() >= 7);

        let ids: Vec<_> = selected.iter().map(|mission| mission.id()).collect();
        assert_eq!(ids, ["M-701", "M-703", "M-704"]);
        assert_eq!(selected[0].id().as_ptr(), missions[0].id().as_ptr());
        assert_eq!(selected[1].id().as_ptr(), missions[2].id().as_ptr());
        assert_eq!(selected[2].id().as_ptr(), missions[3].id().as_ptr());
    }

    #[test]
    fn 外側の値を共有借用するクロージャも渡せる() {
        let missions = missions();
        let maximum_distance_m = 320;

        let selected = select_missions(&missions, |mission: &Mission| {
            mission.distance_m() <= maximum_distance_m
        });

        let ids: Vec<_> = selected.iter().map(|mission| mission.id()).collect();
        assert_eq!(ids, ["M-701", "M-702", "M-704"]);
        assert_eq!(maximum_distance_m, 320);
    }

    #[test]
    fn キャプチャしない関数も条件として渡せる() {
        let missions = missions();

        let selected = select_missions(&missions, is_high_priority);
        let ids: Vec<_> = selected.iter().map(|mission| mission.id()).collect();

        assert_eq!(ids, ["M-703", "M-704"]);
    }

    #[test]
    fn 空入力と一致なしを空のvecで返す() {
        let rule = make_dispatch_filter(String::from("柏"), 1);

        assert!(select_missions(&[], &rule).is_empty());
        assert!(select_missions(&missions(), rule).is_empty());
    }

    #[test]
    fn 日本語と絵文字の接頭辞をそのまま扱う() {
        let missions = vec![
            Mission::new("配送🤖-01", "柏キャンパス🔬・実験棟", 6, 120),
            Mission::new("配送🤖-02", "柏キャンパス・食堂", 6, 90),
        ];
        let rule = make_dispatch_filter(String::from("柏キャンパス🔬"), 6);

        let selected = select_missions(&missions, rule);

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id(), "配送🤖-01");
    }

    #[test]
    fn 優先度の最小値と最大値を境界として扱う() {
        let missions = vec![
            Mission::new("M-706", "本郷・倉庫", 0, 10),
            Mission::new("M-707", "本郷・管制室", u8::MAX, 20),
        ];

        let all = select_missions(&missions, make_dispatch_filter(String::new(), u8::MIN));
        let maximum = select_missions(
            &missions,
            make_dispatch_filter(String::from("本郷"), u8::MAX),
        );

        assert_eq!(all.len(), 2);
        assert_eq!(maximum.len(), 1);
        assert_eq!(maximum[0].id(), "M-707");
    }
}
