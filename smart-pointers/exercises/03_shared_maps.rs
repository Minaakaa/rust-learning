#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 03: `Rc` でキャンパスマップを共有する
//!
//! 管制システムと複数の配送ロボットが、同じ `CampusMap` を読み取ります
//! `CampusMap` 全体をロボットごとに複製する代わりに、単一スレッド向けの
//! 参照カウント型 `Rc<T>` で所有権を共有してください
//!
//! 仕様:
//! - `DispatchNetwork::new` は受け取った `CampusMap` を1回だけ `Rc::new` で包む
//! - 管制側の `DispatchNetwork` 自身も、その `Rc<CampusMap>` を1つ保持する
//! - `add_robot` はロボット ID を所有する `String` にし、入力順で末尾へ追加する
//! - 各 `RobotView` のマップには `Rc::clone` で同じ allocation の所有者を追加する
//! - `remove_robot` は一致する最初のロボットを順序を保って取り除き、所有値として返す
//! - 見つからない ID では待機中のロボットと参照カウントを変更しない
//!
//! 制約:
//! - `CampusMap` に `Clone` を実装または derive しない
//! - ロボットごとに新しい `CampusMap` やマップ内の `String` を作らない
//! - 課題部分で許可される複製は `Rc::clone` と、入力されたロボット ID の所有化だけ
//! - `unsafe` や外部 crate を使わない
//!
//! ヒント:
//! - `Rc::clone(&map)` は `CampusMap` を複製せず、同じ allocation の strong count を増やす
//! - `Rc::ptr_eq` は2つの `Rc` が同じ allocation を指すか確認できる
//! - `Vec::iter().position(...)` と `Vec::remove(...)` で最初の一致を所有値として取り出せる

use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
struct CampusMap {
    campus_name: String,
    checkpoints: Vec<String>,
}

impl CampusMap {
    fn new(campus_name: &str, checkpoints: &[&str]) -> Self {
        Self {
            campus_name: campus_name.to_owned(),
            checkpoints: checkpoints
                .iter()
                .map(|checkpoint| (*checkpoint).to_owned())
                .collect(),
        }
    }

    fn campus_name(&self) -> &str {
        &self.campus_name
    }

    fn checkpoints(&self) -> &[String] {
        &self.checkpoints
    }
}

#[derive(Debug)]
struct RobotView {
    robot_id: String,
    map: Rc<CampusMap>,
}

impl RobotView {
    fn robot_id(&self) -> &str {
        &self.robot_id
    }

    fn map(&self) -> &CampusMap {
        self.map.as_ref()
    }

    fn map_handle(&self) -> &Rc<CampusMap> {
        &self.map
    }
}

#[derive(Debug)]
struct DispatchNetwork {
    control_map: Rc<CampusMap>,
    robots: Vec<RobotView>,
}

impl DispatchNetwork {
    fn new(map: CampusMap) -> Self {
        let _ = map;
        todo!("CampusMap を1つの Rc へ移し、管制ネットワークを作ってください")
    }

    fn add_robot(&mut self, robot_id: &str) {
        let _ = robot_id;
        todo!("管制側と同じ CampusMap を共有する RobotView を追加してください")
    }

    fn remove_robot(&mut self, robot_id: &str) -> Option<RobotView> {
        let _ = robot_id;
        todo!("ID が一致する最初の RobotView を所有値として取り除いてください")
    }

    fn map(&self) -> &CampusMap {
        self.control_map.as_ref()
    }

    fn map_handle(&self) -> &Rc<CampusMap> {
        &self.control_map
    }

    fn robots(&self) -> &[RobotView] {
        &self.robots
    }
}

fn main() {
    let map = CampusMap::new("本郷キャンパス", &["正門", "総合図書館", "工学部2号館"]);
    let mut network = DispatchNetwork::new(map);
    network.add_robot("配送ロボット🤖-01");
    network.add_robot("配送ロボット🤖-02");

    for view in network.robots() {
        println!(
            "{} は {} の {} 地点を共有中",
            view.robot_id(),
            view.map().campus_name(),
            view.map().checkpoints().len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hongo_map() -> CampusMap {
        CampusMap::new("本郷キャンパス", &["正門", "総合図書館", "工学部2号館"])
    }

    #[test]
    fn network構築時は管制側だけがmapを所有する() {
        let network = DispatchNetwork::new(hongo_map());

        assert_eq!(network.map().campus_name(), "本郷キャンパス");
        assert_eq!(
            network.map().checkpoints(),
            ["正門", "総合図書館", "工学部2号館"]
        );
        assert_eq!(Rc::strong_count(network.map_handle()), 1);
    }

    #[test]
    fn robotが空でも有効なnetworkとして扱う() {
        let mut network = DispatchNetwork::new(hongo_map());

        assert!(network.robots().is_empty());
        assert!(network.remove_robot("未登録").is_none());
        assert!(network.robots().is_empty());
        assert_eq!(Rc::strong_count(network.map_handle()), 1);
    }

    #[test]
    fn robotを入力順で追加する() {
        let mut network = DispatchNetwork::new(hongo_map());
        network.add_robot("robot-03");
        network.add_robot("robot-01");
        network.add_robot("robot-02");

        let ids = network
            .robots()
            .iter()
            .map(RobotView::robot_id)
            .collect::<Vec<_>>();

        assert_eq!(ids, ["robot-03", "robot-01", "robot-02"]);
        drop(ids);

        assert!(network.remove_robot("robot-99").is_none());
        assert_eq!(
            network
                .robots()
                .iter()
                .map(RobotView::robot_id)
                .collect::<Vec<_>>(),
            ["robot-03", "robot-01", "robot-02"]
        );
        assert_eq!(Rc::strong_count(network.map_handle()), 4);
    }

    #[test]
    fn 管制側とすべてのviewが同じallocationを指す() {
        let mut network = DispatchNetwork::new(hongo_map());
        network.add_robot("robot-01");
        network.add_robot("robot-02");

        assert!(
            network
                .robots()
                .iter()
                .all(|view| Rc::ptr_eq(network.map_handle(), view.map_handle()))
        );
        assert!(Rc::ptr_eq(
            network.robots()[0].map_handle(),
            network.robots()[1].map_handle()
        ));
    }

    #[test]
    fn robot一台につきstrong_countが一つ増える() {
        let mut network = DispatchNetwork::new(hongo_map());
        assert_eq!(Rc::strong_count(network.map_handle()), 1);

        network.add_robot("robot-01");
        assert_eq!(Rc::strong_count(network.map_handle()), 2);

        network.add_robot("robot-02");
        assert_eq!(Rc::strong_count(network.map_handle()), 3);

        network.add_robot("robot-03");
        assert_eq!(Rc::strong_count(network.map_handle()), 4);
    }

    #[test]
    fn 取り外したviewをdropするとstrong_countが減る() {
        let mut network = DispatchNetwork::new(hongo_map());
        network.add_robot("robot-01");
        network.add_robot("robot-02");
        network.add_robot("robot-03");

        let removed = network
            .remove_robot("robot-02")
            .expect("登録済みロボットを取り外せる");

        assert_eq!(removed.robot_id(), "robot-02");
        assert_eq!(network.robots()[0].robot_id(), "robot-01");
        assert_eq!(network.robots()[1].robot_id(), "robot-03");
        assert_eq!(Rc::strong_count(network.map_handle()), 4);

        drop(removed);
        assert_eq!(Rc::strong_count(network.map_handle()), 3);
    }

    #[test]
    fn 元map内のstringバッファを複製しない() {
        let map = hongo_map();
        let campus_name_ptr = map.campus_name().as_ptr();
        let checkpoint_ptrs = map
            .checkpoints()
            .iter()
            .map(|checkpoint| checkpoint.as_ptr())
            .collect::<Vec<_>>();

        let mut network = DispatchNetwork::new(map);
        network.add_robot("robot-01");
        let view = &network.robots()[0];

        assert_eq!(view.map().campus_name().as_ptr(), campus_name_ptr);
        assert_eq!(
            view.map()
                .checkpoints()
                .iter()
                .map(|checkpoint| checkpoint.as_ptr())
                .collect::<Vec<_>>(),
            checkpoint_ptrs
        );
    }

    #[test]
    fn 同じ内容でも別々に作ったmapは別allocationである() {
        let first = DispatchNetwork::new(hongo_map());
        let second = DispatchNetwork::new(hongo_map());

        assert_eq!(first.map(), second.map());
        assert!(!Rc::ptr_eq(first.map_handle(), second.map_handle()));
        assert_eq!(Rc::strong_count(first.map_handle()), 1);
        assert_eq!(Rc::strong_count(second.map_handle()), 1);
    }

    #[test]
    fn viewはnetworkより長くmapを所有できる() {
        let mut network = DispatchNetwork::new(hongo_map());
        network.add_robot("robot-01");
        let view = network
            .remove_robot("robot-01")
            .expect("登録済みロボットを取り外せる");

        assert_eq!(Rc::strong_count(view.map_handle()), 2);
        drop(network);

        assert_eq!(view.map().campus_name(), "本郷キャンパス");
        assert_eq!(view.map().checkpoints()[1], "総合図書館");
        assert_eq!(Rc::strong_count(view.map_handle()), 1);
    }

    #[test]
    fn utf8のcampus地点robot_idを変更せず共有する() {
        let map = CampusMap::new(
            "柏キャンパス🚚",
            &["宇宙線研究所🔭", "実験棟🧪", "図書館📚"],
        );
        let mut network = DispatchNetwork::new(map);
        network.add_robot("配送ロボット🤖-一号");

        let view = &network.robots()[0];
        assert_eq!(view.robot_id(), "配送ロボット🤖-一号");
        assert_eq!(view.map().campus_name(), "柏キャンパス🚚");
        assert_eq!(
            view.map().checkpoints(),
            ["宇宙線研究所🔭", "実験棟🧪", "図書館📚"]
        );
        assert!(Rc::ptr_eq(network.map_handle(), view.map_handle()));
    }
}
