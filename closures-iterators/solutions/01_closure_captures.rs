//! 問題 01 の解答例

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

fn make_dispatch_filter(required_area: String, minimum_priority: u8) -> impl Fn(&Mission) -> bool {
    move |mission| {
        mission.destination().starts_with(&required_area) && mission.priority() >= minimum_priority
    }
}

fn select_missions<P>(missions: &[Mission], predicate: P) -> Vec<&Mission>
where
    P: Fn(&Mission) -> bool,
{
    missions
        .iter()
        .filter(|mission| predicate(mission))
        .collect()
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
