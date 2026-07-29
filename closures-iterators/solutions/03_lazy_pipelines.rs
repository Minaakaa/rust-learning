//! 問題 03 の解答例

#[derive(Debug, PartialEq, Eq)]
struct Stop {
    name: String,
    open: bool,
    distance_m: Option<u32>,
}

impl Stop {
    fn new(name: &str, open: bool, distance_m: Option<u32>) -> Self {
        Self {
            name: name.to_string(),
            open,
            distance_m,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    const fn is_open(&self) -> bool {
        self.open
    }

    const fn distance_m(&self) -> Option<u32> {
        self.distance_m
    }
}

#[derive(Debug, PartialEq, Eq)]
struct CampusRoute {
    name: String,
    stops: Vec<Stop>,
}

impl CampusRoute {
    fn new(name: &str, stops: Vec<Stop>) -> Self {
        Self {
            name: name.to_string(),
            stops,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn stops(&self) -> &[Stop] {
        &self.stops
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ScheduledStop<'a> {
    sequence: usize,
    stop: &'a Stop,
    distance_m: u32,
}

impl<'a> ScheduledStop<'a> {
    const fn new(sequence: usize, stop: &'a Stop, distance_m: u32) -> Self {
        Self {
            sequence,
            stop,
            distance_m,
        }
    }

    const fn sequence(&self) -> usize {
        self.sequence
    }

    const fn stop(&self) -> &'a Stop {
        self.stop
    }

    const fn distance_m(&self) -> u32 {
        self.distance_m
    }
}

fn dispatchable_stops<'a, P>(
    routes: &'a [CampusRoute],
    max_distance_m: u32,
    mut predicate: P,
) -> impl Iterator<Item = ScheduledStop<'a>> + 'a
where
    P: FnMut(&Stop) -> bool + 'a,
{
    routes
        .iter()
        .flat_map(|route| route.stops().iter())
        .filter_map(move |stop| {
            if !stop.is_open() {
                return None;
            }

            let distance_m = stop.distance_m()?;
            if distance_m > max_distance_m || !predicate(stop) {
                return None;
            }

            Some((stop, distance_m))
        })
        .enumerate()
        .map(|(index, (stop, distance_m))| ScheduledStop::new(index + 1, stop, distance_m))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn routes() -> Vec<CampusRoute> {
        vec![
            CampusRoute::new(
                "本郷キャンパス便",
                vec![
                    Stop::new("総合図書館", true, Some(100)),
                    Stop::new("閉鎖中の実験棟", false, Some(50)),
                    Stop::new("距離未測定の倉庫", true, None),
                    Stop::new("工学部2号館", true, Some(500)),
                ],
            ),
            CampusRoute::new(
                "駒場キャンパス便",
                vec![
                    Stop::new("駒場食堂", true, Some(200)),
                    Stop::new("遠方研究施設", true, Some(800)),
                    Stop::new("正門", true, Some(0)),
                ],
            ),
        ]
    }

    #[test]
    fn 作成しただけではpredicateを呼ばない() {
        let routes = routes();
        let mut calls = 0;

        {
            let iterator = dispatchable_stops(&routes, u32::MAX, |_: &Stop| {
                calls += 1;
                true
            });
            drop(iterator);
        }

        assert_eq!(calls, 0);
    }

    #[test]
    fn next一回に必要な位置までだけ評価する() {
        let routes = routes();
        let mut calls = 0;

        {
            let mut iterator = dispatchable_stops(&routes, u32::MAX, |stop: &Stop| {
                calls += 1;
                stop.name() == "工学部2号館"
            });

            let selected = iterator.next().expect("一致する停留所がある");
            assert_eq!(selected.sequence(), 1);
            assert_eq!(selected.stop().name(), "工学部2号館");
        }

        assert_eq!(calls, 2);
    }

    #[test]
    fn takeは必要な二件を得た時点で評価を止める() {
        let routes = routes();
        let mut calls = 0;

        let names = {
            dispatchable_stops(&routes, 500, |stop: &Stop| {
                calls += 1;
                stop.name() != "工学部2号館"
            })
            .take(2)
            .map(|scheduled| scheduled.stop().name())
            .collect::<Vec<_>>()
        };

        assert_eq!(names, ["総合図書館", "駒場食堂"]);
        assert_eq!(calls, 3);
    }

    #[test]
    fn 基本条件を満たさないstopではpredicateを呼ばない() {
        let routes = vec![CampusRoute::new(
            "基本条件の確認便",
            vec![
                Stop::new("閉鎖中", false, Some(10)),
                Stop::new("距離未測定", true, None),
                Stop::new("距離超過", true, Some(101)),
            ],
        )];
        let mut calls = 0;

        let selected = {
            dispatchable_stops(&routes, 100, |_: &Stop| {
                calls += 1;
                true
            })
            .collect::<Vec<_>>()
        };

        assert!(selected.is_empty());
        assert_eq!(calls, 0);
    }

    #[test]
    fn 複数routeの入力順で採用後のsequenceを付ける() {
        let routes = routes();

        let selected = dispatchable_stops(&routes, 500, |_: &Stop| true).collect::<Vec<_>>();
        let names = selected
            .iter()
            .map(|scheduled| scheduled.stop().name())
            .collect::<Vec<_>>();
        let sequences = selected
            .iter()
            .map(ScheduledStop::sequence)
            .collect::<Vec<_>>();

        assert_eq!(routes[0].name(), "本郷キャンパス便");
        assert_eq!(routes[1].name(), "駒場キャンパス便");
        assert_eq!(names, ["総合図書館", "工学部2号館", "駒場食堂", "正門"]);
        assert_eq!(sequences, [1, 2, 3, 4]);
    }

    #[test]
    fn 距離上限を境界値として含める() {
        let routes = routes();

        let selected = dispatchable_stops(&routes, 500, |stop: &Stop| stop.name() == "工学部2号館")
            .next()
            .expect("上限ちょうどの停留所がある");

        assert_eq!(selected.distance_m(), 500);
        assert!(selected.stop().is_open());
        assert_eq!(selected.stop().distance_m(), Some(500));
    }

    #[test]
    fn 空のroute一覧と空のstop一覧を処理する() {
        let empty_routes: Vec<CampusRoute> = Vec::new();
        assert!(
            dispatchable_stops(&empty_routes, u32::MAX, |_: &Stop| true)
                .next()
                .is_none()
        );

        let routes = vec![CampusRoute::new("停留所なし", Vec::new())];
        assert!(
            dispatchable_stops(&routes, u32::MAX, |_: &Stop| true)
                .next()
                .is_none()
        );
    }

    #[test]
    fn scheduled_stopは元のstopを借用する() {
        let routes = routes();

        let borrowed = {
            let scheduled = dispatchable_stops(&routes, 100, |_: &Stop| true)
                .next()
                .expect("総合図書館が対象になる");
            scheduled.stop()
        };

        assert!(std::ptr::eq(borrowed, &routes[0].stops()[0]));
        assert_eq!(borrowed.name(), "総合図書館");
    }

    #[test]
    fn utf8の名前を変更せず扱う() {
        let routes = vec![CampusRoute::new(
            "柏キャンパス🚚便",
            vec![
                Stop::new("実験棟🔬", true, Some(42)),
                Stop::new("図書館📚", true, Some(43)),
            ],
        )];

        let selected = dispatchable_stops(&routes, 42, |stop: &Stop| stop.name().contains('🔬'))
            .collect::<Vec<_>>();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].stop().name(), "実験棟🔬");
        assert_eq!(selected[0].distance_m(), 42);
    }

    #[test]
    fn 枯渇後は繰り返しnoneを返す() {
        let routes = vec![CampusRoute::new(
            "短距離便",
            vec![
                Stop::new("第一地点", true, Some(1)),
                Stop::new("第二地点", true, Some(2)),
            ],
        )];
        let mut calls = 0;

        {
            let mut iterator = dispatchable_stops(&routes, 2, |_: &Stop| {
                calls += 1;
                true
            });

            assert_eq!(iterator.next().unwrap().sequence(), 1);
            assert_eq!(iterator.next().unwrap().sequence(), 2);
            assert_eq!(iterator.next(), None);
            assert_eq!(iterator.next(), None);
        }

        assert_eq!(calls, 2);
    }
}
