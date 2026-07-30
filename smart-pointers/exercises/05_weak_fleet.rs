#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 05: `Weak` で循環しないロボット群を作る
//!
//! 管制塔は登録された配送ロボットを所有し、各ロボットは所属する管制塔へ
//! 所有権を持たない逆参照を保存します
//! `Rc`、`RefCell`、`Weak` を組み合わせ、管制塔を破棄した後も循環参照で
//! メモリを保持し続けない登録システムを完成させてください
//!
//! 仕様:
//! - `ControlTower::new` は管制塔を1つの `Rc` に入れて返す
//! - 管制塔は `RefCell<Vec<Rc<Robot>>>` で登録順にロボットを強所有する
//! - `register` は `self: &Rc<Self>` から `Rc::downgrade` で逆参照を作る
//! - 登録成功時は管制塔と呼び出し側が同じ `Robot` allocation を強所有する
//! - 同じ ID が登録済みなら `RegisterError::DuplicateId` で入力 `String` を返す
//! - 重複登録ではロボット一覧と strong count、weak countを変更しない
//! - `find` は同じ `Robot` allocation の新しい `Rc` ハンドルを返す
//! - `unregister` は一致する `Rc<Robot>` を一覧から取り除き、所有値として返す
//! - ロボットの `tower` は `Weak::upgrade` の結果を返す
//! - 管制塔の破棄後も外部の `Rc<Robot>` は利用でき、`tower` は `None` を返す
//!
//! 制約:
//! - `ControlTower`、`Robot`、内部の `String` を複製しない
//! - 複製してよいのは同じ allocation の所有者を増やす `Rc::clone` だけ
//! - ロボットから管制塔への参照を `Rc<ControlTower>` に変更しない
//! - `unsafe` や外部 crate を使わない
//!
//! ヒント:
//! - `Rc::downgrade(&tower)` は strong count を増やさず `Weak` を作る
//! - `Weak::upgrade` は対象が生存中なら `Some(Rc<_>)`、破棄後なら `None` を返す
//! - `Rc::ptr_eq` で2つのハンドルが同じ allocation を指すか確認できる
//! - `RefCell` の借用ガードは必要な処理が終わった位置でスコープから外す
//! - `Vec::remove` は要素を複製せず、所有値として取り出せる

use std::cell::{Ref, RefCell};
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct ControlTower {
    name: String,
    robots: RefCell<Vec<Rc<Robot>>>,
}

impl ControlTower {
    fn new(name: String) -> Rc<Self> {
        todo!("管制塔 {name:?} を Rc に入れ、空のロボット一覧を作ってください")
    }

    fn register(self: &Rc<Self>, id: String) -> Result<Rc<Robot>, RegisterError> {
        todo!(
            "管制塔 {:?} へ {id:?} を登録してください strong={} weak={}",
            self.name(),
            Rc::strong_count(self),
            Rc::weak_count(self)
        )
    }

    fn find(&self, id: &str) -> Option<Rc<Robot>> {
        todo!("管制塔 {:?} から ID {id:?} を検索してください", self.name())
    }

    fn unregister(&self, id: &str) -> Option<Rc<Robot>> {
        todo!(
            "管制塔 {:?} から ID {id:?} の Robot を取り除いてください",
            self.name()
        )
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn robots(&self) -> Ref<'_, [Rc<Robot>]> {
        Ref::map(self.robots.borrow(), |robots| robots.as_slice())
    }
}

#[derive(Debug)]
struct Robot {
    id: String,
    #[allow(dead_code, reason = "tower 完成前のスターターでは参照されないため")]
    tower: Weak<ControlTower>,
}

impl Robot {
    fn id(&self) -> &str {
        &self.id
    }

    fn tower(&self) -> Option<Rc<ControlTower>> {
        todo!("Robot {:?} の Weak 参照を upgrade してください", self.id())
    }
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code, reason = "register 完成前のスターターでは構築されないため")]
enum RegisterError {
    DuplicateId(String),
}

fn main() {
    let tower = ControlTower::new(String::from("本郷キャンパス管制塔"));
    let robot = tower
        .register(String::from("配送ロボット🤖-805"))
        .expect("未登録のロボットを追加できる");

    println!(
        "{} に {} を登録: {} 台",
        tower.name(),
        robot.id(),
        tower.robots().len()
    );
    println!("管制塔へ接続可能={}", robot.tower().is_some());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tower() -> Rc<ControlTower> {
        ControlTower::new(String::from("本郷キャンパス管制塔"))
    }

    #[test]
    fn 新しいtowerは名前を所有し空の登録状態を持つ() {
        let name = String::from("本郷キャンパス管制塔");
        let name_pointer = name.as_ptr();
        let tower = ControlTower::new(name);

        assert_eq!(tower.name(), "本郷キャンパス管制塔");
        assert_eq!(tower.name().as_ptr(), name_pointer);
        assert!(tower.robots().is_empty());
        assert_eq!(Rc::strong_count(&tower), 1);
        assert_eq!(Rc::weak_count(&tower), 0);
        assert!(tower.find("未登録").is_none());
        assert!(tower.unregister("未登録").is_none());
    }

    #[test]
    fn robotを登録順で保持し同じallocationを返す() {
        let tower = tower();
        let third_id = String::from("R-03");
        let third_id_pointer = third_id.as_ptr();
        let third = tower.register(third_id).expect("登録できる");
        let first = tower.register(String::from("R-01")).expect("登録できる");
        let second = tower.register(String::from("R-02")).expect("登録できる");

        {
            let robots = tower.robots();
            let ids = robots.iter().map(|robot| robot.id()).collect::<Vec<_>>();

            assert_eq!(ids, ["R-03", "R-01", "R-02"]);
            assert!(Rc::ptr_eq(&robots[0], &third));
            assert!(Rc::ptr_eq(&robots[1], &first));
            assert!(Rc::ptr_eq(&robots[2], &second));
        }

        assert_eq!(third.id().as_ptr(), third_id_pointer);
        assert_eq!(Rc::weak_count(&tower), 3);
        assert_eq!(Rc::strong_count(&third), 2);
        assert_eq!(Rc::strong_count(&first), 2);
        assert_eq!(Rc::strong_count(&second), 2);
    }

    #[test]
    fn 重複idは入力stringを返し登録状態を変更しない() {
        let tower = tower();
        let registered = tower
            .register(String::from("R-duplicate"))
            .expect("最初は登録できる");
        let strong_before = Rc::strong_count(&registered);
        let weak_before = Rc::weak_count(&tower);
        let duplicate = String::from("R-duplicate");
        let duplicate_pointer = duplicate.as_ptr();

        let error = tower
            .register(duplicate)
            .expect_err("同じ ID は登録できない");

        let RegisterError::DuplicateId(returned_id) = error;
        assert_eq!(returned_id, "R-duplicate");
        assert_eq!(returned_id.as_ptr(), duplicate_pointer);
        assert_eq!(tower.robots().len(), 1);
        assert!(Rc::ptr_eq(&tower.robots()[0], &registered));
        assert_eq!(Rc::strong_count(&registered), strong_before);
        assert_eq!(Rc::weak_count(&tower), weak_before);
    }

    #[test]
    fn findは同じrobotのstrong_handleだけを追加する() {
        let tower = tower();
        let registered = tower.register(String::from("R-find")).expect("登録できる");
        assert_eq!(Rc::strong_count(&registered), 2);

        let found = tower.find("R-find").expect("登録済み Robot が見つかる");

        assert!(Rc::ptr_eq(&found, &registered));
        assert_eq!(Rc::strong_count(&registered), 3);
        drop(found);
        assert_eq!(Rc::strong_count(&registered), 2);
        assert!(tower.find("R-missing").is_none());
        assert_eq!(Rc::strong_count(&registered), 2);
    }

    #[test]
    fn unregisterは同じrobotを返し残りの登録順を保つ() {
        let tower = tower();
        let first = tower.register(String::from("R-01")).expect("登録できる");
        let second = tower.register(String::from("R-02")).expect("登録できる");
        let third = tower.register(String::from("R-03")).expect("登録できる");

        let removed = tower.unregister("R-02").expect("登録を解除できる");

        assert!(Rc::ptr_eq(&removed, &second));
        assert!(tower.find("R-02").is_none());
        {
            let robots = tower.robots();
            let ids = robots.iter().map(|robot| robot.id()).collect::<Vec<_>>();
            assert_eq!(ids, ["R-01", "R-03"]);
            assert!(Rc::ptr_eq(&robots[0], &first));
            assert!(Rc::ptr_eq(&robots[1], &third));
        }

        drop(second);
        assert_eq!(Rc::strong_count(&removed), 1);
        assert!(tower.unregister("R-02").is_none());
        assert_eq!(tower.robots().len(), 2);
    }

    #[test]
    fn 外部handleをdropしてもtowerがrobotを所有し続ける() {
        let tower = tower();
        let robot = tower.register(String::from("R-owned")).expect("登録できる");
        let robot_probe = Rc::downgrade(&robot);
        assert_eq!(Rc::strong_count(&robot), 2);

        drop(robot);

        assert_eq!(robot_probe.strong_count(), 1);
        let found = tower.find("R-owned").expect("管制塔が所有している");
        let upgraded = robot_probe.upgrade().expect("Robot は生存している");
        assert!(Rc::ptr_eq(&found, &upgraded));
        assert_eq!(Rc::strong_count(&found), 3);
    }

    #[test]
    fn robotのweak参照は同じtowerへupgradeする() {
        let tower = tower();
        let robot = tower.register(String::from("R-link")).expect("登録できる");

        assert_eq!(Rc::strong_count(&tower), 1);
        assert_eq!(Rc::weak_count(&tower), 1);

        let linked_tower = robot.tower().expect("管制塔が生存している");
        assert!(Rc::ptr_eq(&linked_tower, &tower));
        assert_eq!(Rc::strong_count(&tower), 2);
        assert_eq!(Rc::weak_count(&tower), 1);

        drop(linked_tower);
        assert_eq!(Rc::strong_count(&tower), 1);
    }

    #[test]
    fn tower破棄後も外部robotは生存し循環を残さない() {
        let tower = tower();
        let tower_probe = Rc::downgrade(&tower);
        let robot = tower
            .register(String::from("R-survivor"))
            .expect("登録できる");

        assert_eq!(Rc::strong_count(&tower), 1);
        assert_eq!(Rc::weak_count(&tower), 2);
        assert_eq!(Rc::strong_count(&robot), 2);

        drop(tower);

        assert!(tower_probe.upgrade().is_none());
        assert!(robot.tower().is_none());
        assert_eq!(robot.id(), "R-survivor");
        assert_eq!(Rc::strong_count(&robot), 1);
    }

    #[test]
    fn unregister後のrobotはtowerと独立して生存できる() {
        let tower = tower();
        let tower_probe = Rc::downgrade(&tower);
        let registered = tower
            .register(String::from("R-detached"))
            .expect("登録できる");

        let detached = tower.unregister("R-detached").expect("解除できる");
        assert!(Rc::ptr_eq(&detached, &registered));
        assert!(tower.robots().is_empty());

        drop(registered);
        assert_eq!(Rc::strong_count(&detached), 1);
        let linked_tower = detached.tower().expect("管制塔はまだ生存している");
        assert!(Rc::ptr_eq(&linked_tower, &tower));
        drop(linked_tower);

        drop(tower);
        assert!(tower_probe.upgrade().is_none());
        assert!(detached.tower().is_none());
        assert_eq!(detached.id(), "R-detached");
    }

    #[test]
    fn utf8のtower名とrobot_idを変更せず扱う() {
        let tower = ControlTower::new(String::from("柏キャンパス管制塔🗼"));
        let first = tower
            .register(String::from("配送ロボット🤖-一号"))
            .expect("登録できる");
        let second = tower
            .register(String::from("配送ロボット🚚-二号"))
            .expect("登録できる");

        assert_eq!(tower.name(), "柏キャンパス管制塔🗼");
        {
            let robots = tower.robots();
            let ids = robots.iter().map(|robot| robot.id()).collect::<Vec<_>>();
            assert_eq!(ids, ["配送ロボット🤖-一号", "配送ロボット🚚-二号"]);
        }

        let found = tower
            .find("配送ロボット🚚-二号")
            .expect("UTF-8 ID を検索できる");
        let removed = tower
            .unregister("配送ロボット🤖-一号")
            .expect("UTF-8 ID を解除できる");

        assert!(Rc::ptr_eq(&found, &second));
        assert!(Rc::ptr_eq(&removed, &first));
        assert_eq!(tower.robots().len(), 1);
        assert_eq!(tower.robots()[0].id(), "配送ロボット🚚-二号");
    }
}
