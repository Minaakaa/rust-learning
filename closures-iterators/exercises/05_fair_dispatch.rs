#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 05: 2つの待機列を公平に反復する
//!
//! 緊急任務と通常任務を別々の待機列で管理し、どちらか一方だけを待たせ続けない
//! `FairDispatch` を作ります
//! 独自の `Iterator` と `IntoIterator` を実装し、`for` や既存の iterator adaptor から
//! `&DispatchQueues` を直接利用できるようにしてください
//!
//! 仕様:
//! - 両方に任務があれば緊急、通常の順で交互に返す
//! - 各待機列の内部順序を保つ
//! - 一方が空になった後は、もう一方の残りを順番に返す
//! - 両方が空なら `None` を返し、その後も常に `None` を返す
//! - 任務を複製せず、元の `Mission` への参照を返す
//! - `&DispatchQueues` の反復では待機列を消費も変更もしない
//!
//! TODO:
//! - `Iterator for FairDispatch<'a>` の `next` を実装する
//! - `IntoIterator for &'a DispatchQueues` で2つの slice iterator を作る
//! - `FairDispatch` 自体への `IntoIterator` は実装しない

#[derive(Debug, PartialEq, Eq)]
struct Mission {
    id: String,
}

impl Mission {
    fn new(id: &str) -> Self {
        Self { id: id.to_owned() }
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DispatchQueues {
    urgent: Vec<Mission>,
    routine: Vec<Mission>,
}

impl DispatchQueues {
    fn new(urgent: Vec<Mission>, routine: Vec<Mission>) -> Self {
        Self { urgent, routine }
    }

    fn urgent(&self) -> &[Mission] {
        &self.urgent
    }

    fn routine(&self) -> &[Mission] {
        &self.routine
    }
}

struct FairDispatch<'a> {
    urgent: std::slice::Iter<'a, Mission>,
    routine: std::slice::Iter<'a, Mission>,
    urgent_turn: bool,
}

impl<'a> Iterator for FairDispatch<'a> {
    type Item = &'a Mission;

    fn next(&mut self) -> Option<Self::Item> {
        todo!(
            "緊急 {} 件、通常 {} 件、緊急の番={}",
            self.urgent.len(),
            self.routine.len(),
            self.urgent_turn
        )
    }
}

impl<'a> IntoIterator for &'a DispatchQueues {
    type Item = &'a Mission;
    type IntoIter = FairDispatch<'a>;

    fn into_iter(self) -> Self::IntoIter {
        todo!(
            "緊急 {} 件と通常 {} 件を借用する FairDispatch を作ってください",
            self.urgent.len(),
            self.routine.len()
        )
    }
}

fn main() {
    let queues = DispatchQueues::new(
        vec![Mission::new("緊急-701"), Mission::new("緊急-702")],
        vec![Mission::new("通常-701"), Mission::new("通常-702")],
    );
    let order: Vec<_> = (&queues).into_iter().map(Mission::id).collect();

    println!("公平な配送順: {order:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queues(urgent: &[&str], routine: &[&str]) -> DispatchQueues {
        DispatchQueues::new(
            urgent.iter().copied().map(Mission::new).collect(),
            routine.iter().copied().map(Mission::new).collect(),
        )
    }

    fn collect_ids(queues: &DispatchQueues) -> Vec<&str> {
        queues.into_iter().map(Mission::id).collect()
    }

    #[test]
    fn 同数ならurgentから交互に返す() {
        let queues = queues(&["U-1", "U-2", "U-3"], &["R-1", "R-2", "R-3"]);

        assert_eq!(
            collect_ids(&queues),
            ["U-1", "R-1", "U-2", "R-2", "U-3", "R-3"]
        );
    }

    #[test]
    fn urgentが長ければ残りも内部順序で返す() {
        let queues = queues(&["U-1", "U-2", "U-3", "U-4"], &["R-1"]);

        assert_eq!(collect_ids(&queues), ["U-1", "R-1", "U-2", "U-3", "U-4"]);
    }

    #[test]
    fn routineが長ければ残りも内部順序で返す() {
        let queues = queues(&["U-1"], &["R-1", "R-2", "R-3", "R-4"]);

        assert_eq!(collect_ids(&queues), ["U-1", "R-1", "R-2", "R-3", "R-4"]);
    }

    #[test]
    fn 片方が空ならもう片方だけを返す() {
        let routine_only = queues(&[], &["R-1", "R-2"]);
        let urgent_only = queues(&["U-1", "U-2"], &[]);

        assert_eq!(collect_ids(&routine_only), ["R-1", "R-2"]);
        assert_eq!(collect_ids(&urgent_only), ["U-1", "U-2"]);
    }

    #[test]
    fn 両方が空なら繰り返しnoneを返す() {
        let queues = queues(&[], &[]);
        let mut dispatch = (&queues).into_iter();

        assert_eq!(dispatch.next(), None);
        assert_eq!(dispatch.next(), None);
        assert_eq!(dispatch.next(), None);
    }

    #[test]
    fn nextで一件ずつ進み終了後は再開しない() {
        let queues = queues(&["U-1", "U-2"], &["R-1"]);
        let mut dispatch = (&queues).into_iter();

        assert_eq!(dispatch.next().map(Mission::id), Some("U-1"));
        assert_eq!(dispatch.next().map(Mission::id), Some("R-1"));
        assert_eq!(dispatch.next().map(Mission::id), Some("U-2"));
        assert_eq!(dispatch.next(), None);
        assert_eq!(dispatch.next(), None);
    }

    #[test]
    fn missionを複製せず借用し反復後もqueuesを使える() {
        let queues = queues(&["U-1"], &["R-1"]);

        {
            let mut dispatch = (&queues).into_iter();
            let urgent = dispatch.next().expect("緊急任務がある");
            let routine = dispatch.next().expect("通常任務がある");

            assert!(std::ptr::eq(urgent, &queues.urgent()[0]));
            assert!(std::ptr::eq(routine, &queues.routine()[0]));
        }

        assert_eq!(queues.urgent()[0].id(), "U-1");
        assert_eq!(queues.routine()[0].id(), "R-1");
    }

    #[test]
    fn queuesへの参照をforで反復できる() {
        let queues = queues(&["U-1", "U-2"], &["R-1", "R-2"]);
        let mut order = Vec::new();

        for mission in &queues {
            order.push(mission.id());
        }

        assert_eq!(order, ["U-1", "R-1", "U-2", "R-2"]);
    }

    #[test]
    fn 既存のiterator_adaptorを組み合わせられる() {
        let queues = queues(&["U-1", "U-2", "U-3"], &["R-1", "R-2", "R-3"]);

        let first_three: Vec<_> = (&queues).into_iter().take(3).map(Mission::id).collect();

        assert_eq!(first_three, ["U-1", "R-1", "U-2"]);
    }

    #[test]
    fn utf8のidを順序と内容を変えず返す() {
        let queues = queues(&["緊急🚨-一", "緊急🚚-二"], &["通常🤖-一", "通常📦-二"]);

        assert_eq!(
            collect_ids(&queues),
            ["緊急🚨-一", "通常🤖-一", "緊急🚚-二", "通常📦-二"]
        );
    }
}
