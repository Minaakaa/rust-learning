#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 03: 借用するミッションカタログを作る
//!
//! 現在の `MissionCatalog` は入力された全ミッションを複製し、検索結果も所有値として
//! 返します
//! 元の一覧を借用し、同じ `Mission` への参照を返すゼロコピーのカタログへ変更してください
//!
//! 仕様:
//! - `MissionCatalog<'data>` は `&'data [Mission]` を保持する
//! - `new` は入力スライスを複製せずに借用する
//! - `find` は一致する最初の `Mission` を `Option<&'data Mission>` で返す
//! - `highest_priority` は最大の `priority` を持つミッションを返す
//! - 最大値が同じ場合は入力で先に現れたミッションを返す
//! - `urgent` は `priority >= minimum` の参照を入力順で返す
//! - `into_missions` はカタログを消費して元の `&'data [Mission]` を返す
//! - `find` の戻り値を検索用 `id` や一時的な `&self` の借用へ結び付けない
//! - `Mission` を複製しない
//!
//! テストは文字列とスライスのポインタも確認します
//! 値が等しいだけでなく、元のミッションそのものを借用してください
//!
//! ヒント:
//! - 構造体、`impl`、`new` の引数へ同じ `'data` を付ける
//! - 保存データを返すメソッドでは `&self` と戻り値の `'data` を区別する
//! - ライフタイム注釈は値を長生きさせず、参照同士の関係を表す

#[derive(Debug, Clone, PartialEq, Eq)]
struct Mission {
    id: String,
    destination: String,
    priority: u8,
}

impl Mission {
    fn new(id: &str, destination: &str, priority: u8) -> Self {
        Self {
            id: id.to_owned(),
            destination: destination.to_owned(),
            priority,
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
}

#[derive(Debug, PartialEq, Eq)]
struct MissionCatalog {
    missions: Vec<Mission>,
}

impl MissionCatalog {
    fn new(missions: &[Mission]) -> Self {
        Self {
            missions: missions.to_vec(),
        }
    }

    fn find(&self, id: &str) -> Option<Mission> {
        todo!(
            "ID {id} を {} 件から探し、元の Mission への参照を返してください",
            self.missions.len()
        )
    }

    fn highest_priority(&self) -> Option<Mission> {
        todo!(
            "{} 件から最大 priority の Mission を探してください",
            self.missions.len()
        )
    }

    fn urgent(&self, minimum: u8) -> Vec<Mission> {
        todo!(
            "{} 件から priority が {minimum} 以上の Mission を集めてください",
            self.missions.len()
        )
    }

    fn into_missions(self) -> Vec<Mission> {
        todo!(
            "カタログを消費し、借用中の {} 件を返してください",
            self.missions.len()
        )
    }
}

fn main() {
    let missions = vec![
        Mission::new("M-630", "図書館", 4),
        Mission::new("M-631", "先端科学研究棟", 9),
        Mission::new("M-632", "学生食堂", 6),
    ];
    let catalog = MissionCatalog::new(&missions);

    println!("検索結果: {:?}", catalog.find("M-631"));
    println!("最優先: {:?}", catalog.highest_priority());
    println!("priority 6 以上: {:?}", catalog.urgent(6));
    println!("全ミッション: {:?}", catalog.into_missions());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn missions() -> Vec<Mission> {
        vec![
            Mission::new("M-601", "図書館", 3),
            Mission::new("M-602", "先端科学研究棟", 9),
            Mission::new("M-603", "学生食堂", 6),
            Mission::new("M-604", "駒場博物館", 9),
        ]
    }

    #[test]
    fn 短命な検索語とカタログを破棄した後も検索結果を使える() {
        let source = missions();

        let found = {
            let catalog = MissionCatalog::new(&source);
            let query = String::from("M-602");
            catalog.find(&query).expect("M-602 が存在する")
        };

        assert_eq!(found.id(), "M-602");
        assert_eq!(found.destination(), "先端科学研究棟");
        assert_eq!(found.priority(), 9);
        assert_eq!(found.id().as_ptr(), source[1].id().as_ptr());
    }

    #[test]
    fn 検索結果は元のmissionそのものを参照する() {
        let source = missions();
        let catalog = MissionCatalog::new(&source);

        let found = catalog.find("M-603").expect("M-603 が存在する");

        assert_eq!(found.id().as_ptr(), source[2].id().as_ptr());
        assert_eq!(
            found.destination().as_ptr(),
            source[2].destination().as_ptr()
        );
    }

    #[test]
    fn 存在しないidではnoneを返す() {
        let source = missions();
        let catalog = MissionCatalog::new(&source);

        assert_eq!(catalog.find("M-999"), None);
    }

    #[test]
    fn 最大のpriorityを選ぶ() {
        let source = vec![
            Mission::new("M-610", "本郷", 1),
            Mission::new("M-611", "弥生", u8::MAX),
            Mission::new("M-612", "浅野", 100),
        ];
        let highest = {
            let catalog = MissionCatalog::new(&source);
            catalog.highest_priority().expect("ミッションが存在する")
        };

        assert_eq!(highest.id(), "M-611");
        assert_eq!(highest.id().as_ptr(), source[1].id().as_ptr());
    }

    #[test]
    fn 最大priorityが同じなら先頭側を選ぶ() {
        let source = missions();
        let catalog = MissionCatalog::new(&source);

        let highest = catalog.highest_priority().expect("ミッションが存在する");

        assert_eq!(highest.id(), "M-602");
        assert_eq!(highest.id().as_ptr(), source[1].id().as_ptr());
    }

    #[test]
    fn urgentは境界値を含めて入力順で返す() {
        let source = missions();
        let urgent = {
            let catalog = MissionCatalog::new(&source);
            catalog.urgent(6)
        };
        let ids: Vec<_> = urgent.iter().map(|mission| mission.id()).collect();

        assert_eq!(ids, ["M-602", "M-603", "M-604"]);
        assert_eq!(urgent[0].id().as_ptr(), source[1].id().as_ptr());
        assert_eq!(urgent[1].id().as_ptr(), source[2].id().as_ptr());
        assert_eq!(urgent[2].id().as_ptr(), source[3].id().as_ptr());
    }

    #[test]
    fn urgentはu8の両端を扱う() {
        let source = vec![
            Mission::new("M-620", "低優先", 0),
            Mission::new("M-621", "高優先", u8::MAX),
        ];
        let catalog = MissionCatalog::new(&source);

        assert_eq!(catalog.urgent(0).len(), 2);
        let maximum = catalog.urgent(u8::MAX);
        assert_eq!(maximum.len(), 1);
        assert_eq!(maximum[0].id(), "M-621");
        assert_eq!(maximum[0].id().as_ptr(), source[1].id().as_ptr());
    }

    #[test]
    fn 空のカタログを安全に処理する() {
        let source = Vec::new();
        let catalog = MissionCatalog::new(&source);

        assert_eq!(catalog.find("M-000"), None);
        assert_eq!(catalog.highest_priority(), None);
        assert!(catalog.urgent(0).is_empty());
        assert!(catalog.into_missions().is_empty());
    }

    #[test]
    fn カタログを消費した後も元のスライスを使える() {
        let source = missions();

        let returned = {
            let catalog = MissionCatalog::new(&source);
            catalog.into_missions()
        };

        assert_eq!(returned.len(), source.len());
        assert_eq!(returned.as_ptr(), source.as_ptr());
        assert_eq!(returned[3].destination(), "駒場博物館");
    }

    #[test]
    fn utf8のidと配送先をそのまま借用する() {
        let source = vec![
            Mission::new("緊急-🚚", "柏キャンパス・実験棟🔬", 42),
            Mission::new("通常-1", "本郷キャンパス", 1),
        ];
        let catalog = MissionCatalog::new(&source);

        let found = catalog
            .find("緊急-🚚")
            .expect("日本語と絵文字の ID が存在する");

        assert_eq!(found.destination(), "柏キャンパス・実験棟🔬");
        assert_eq!(found.id().as_ptr(), source[0].id().as_ptr());
        assert_eq!(
            found.destination().as_ptr(),
            source[0].destination().as_ptr()
        );
    }
}
