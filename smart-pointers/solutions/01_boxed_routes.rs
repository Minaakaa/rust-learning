//! 問題 01 の解答例

#[derive(Debug, PartialEq, Eq)]
struct Stop {
    name: String,
    distance_from_previous_m: u32,
}

impl Stop {
    fn new(name: &str, distance_from_previous_m: u32) -> Self {
        Self {
            name: name.to_owned(),
            distance_from_previous_m,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    const fn distance_from_previous_m(&self) -> u32 {
        self.distance_from_previous_m
    }
}

#[derive(Debug, PartialEq, Eq)]
struct RouteNode {
    stop: Stop,
    next: Option<Box<RouteNode>>,
}

impl RouteNode {
    fn stop(&self) -> &Stop {
        &self.stop
    }

    fn next(&self) -> Option<&RouteNode> {
        self.next.as_deref()
    }

    fn len(&self) -> usize {
        1 + self.next().map_or(0, RouteNode::len)
    }

    fn checked_distance_m(&self) -> Option<u32> {
        let remaining = self.next().map_or(Some(0), RouteNode::checked_distance_m)?;
        self.stop.distance_from_previous_m().checked_add(remaining)
    }

    fn find(&self, name: &str) -> Option<&Stop> {
        if self.stop.name() == name {
            return Some(&self.stop);
        }

        self.next().and_then(|next| next.find(name))
    }
}

fn build_route(stops: Vec<Stop>) -> Option<Box<RouteNode>> {
    stops
        .into_iter()
        .rev()
        .fold(None, |next, stop| Some(Box::new(RouteNode { stop, next })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route() -> Box<RouteNode> {
        build_route(vec![
            Stop::new("本郷正門", 0),
            Stop::new("総合図書館", 180),
            Stop::new("工学部2号館", 260),
            Stop::new("学生食堂", 90),
        ])
        .expect("経路がある")
    }

    fn head_name(route: &RouteNode) -> &str {
        route.stop().name()
    }

    #[test]
    fn 空の入力では経路を作らない() {
        assert_eq!(build_route(Vec::new()), None);
    }

    #[test]
    fn 一地点の経路をboxへ格納する() {
        let route = build_route(vec![Stop::new("柏図書館", 42)]).expect("一地点がある");

        assert_eq!(route.stop().name(), "柏図書館");
        assert_eq!(route.stop().distance_from_previous_m(), 42);
        assert_eq!(route.next(), None);
        assert_eq!(route.len(), 1);
    }

    #[test]
    fn 複数地点を入力順につなぐ() {
        let route = route();
        let second = route.next().expect("2地点目がある");
        let third = second.next().expect("3地点目がある");
        let fourth = third.next().expect("4地点目がある");

        assert_eq!(route.stop().name(), "本郷正門");
        assert_eq!(second.stop().name(), "総合図書館");
        assert_eq!(third.stop().name(), "工学部2号館");
        assert_eq!(fourth.stop().name(), "学生食堂");
        assert_eq!(fourth.next(), None);
        assert_eq!(route.len(), 4);
    }

    #[test]
    fn 全地点の距離を検査付きで合計する() {
        let route = route();

        assert_eq!(route.checked_distance_m(), Some(530));
    }

    #[test]
    fn 距離合計があふれたらnoneを返す() {
        let route = build_route(vec![
            Stop::new("最大距離", u32::MAX),
            Stop::new("追加距離", 1),
        ])
        .expect("経路がある");

        assert_eq!(route.checked_distance_m(), None);
    }

    #[test]
    fn 最初に一致する地点を借用して返す() {
        let route = build_route(vec![
            Stop::new("倉庫", 0),
            Stop::new("充電所", 20),
            Stop::new("倉庫", 30),
        ])
        .expect("経路がある");
        let first = route.find("倉庫").expect("最初の倉庫がある");

        assert!(std::ptr::eq(first, route.stop()));
        assert_eq!(route.find("充電所").unwrap().distance_from_previous_m(), 20);
        assert_eq!(route.find("存在しない地点"), None);
    }

    #[test]
    fn stopとstringを複製せず経路へ移す() {
        let stops = vec![Stop::new("駒場正門", 0), Stop::new("数理科学研究科", 75)];
        let name_pointer = stops[1].name().as_ptr();

        let route = build_route(stops).expect("経路がある");
        let moved = route.find("数理科学研究科").expect("移動した地点がある");

        assert_eq!(moved.name().as_ptr(), name_pointer);
    }

    #[test]
    fn boxからroute_nodeへ自動的にderef_coercionする() {
        let route = route();

        assert_eq!(head_name(&route), "本郷正門");
    }

    #[test]
    fn 日本語と絵文字を変更せず探索する() {
        let route = build_route(vec![
            Stop::new("管制室🤖", 0),
            Stop::new("実験棟🔬", 125),
            Stop::new("配送所🚚", 80),
        ])
        .expect("経路がある");

        assert_eq!(route.find("実験棟🔬").unwrap().name(), "実験棟🔬");
        assert_eq!(route.checked_distance_m(), Some(205));
    }

    #[test]
    fn 長い経路でも全地点を数える() {
        let stops = (0..128)
            .map(|index| Stop::new(&format!("地点-{index}"), 1))
            .collect();
        let route = build_route(stops).expect("長い経路がある");

        assert_eq!(route.len(), 128);
        assert_eq!(route.find("地点-127").unwrap().name(), "地点-127");
        assert_eq!(route.checked_distance_m(), Some(128));
    }
}
