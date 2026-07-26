#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 01: Vec で荷物棚を管理する
//!
//! 倉庫の荷物棚では、荷物を受け入れた順番のまま保存し、位置を指定して取り出します。
//! 可変長の連続したデータを扱う `Vec<T>` を使って、`CargoShelf` を完成させてください。
//!
//! 仕様:
//! - `CargoShelf::new` は `Vec::with_capacity` で最大スロット数を事前確保する。
//! - 同じ ID の荷物は登録しない。棚が満杯でも、重複エラーを先に返す。
//! - `load` が失敗したときは棚を変更せず、受け取った荷物を `LoadError` で返す。
//! - `parcel_at` は範囲外の添字でもパニックせず `None` を返す。
//! - `unload_last` は末尾、`unload_at` は指定位置から荷物の所有権を取り出す。
//! - `unload_at` で範囲外を指定した場合は、棚を変更しない。
//!
//! ヒント:
//! - `push`、`pop`、`get`、`remove` を使い分けます。
//! - `Vec::remove` は範囲外でパニックするため、先に添字を確認します。
//! - エラーへ荷物を入れて返せば、呼び出し側はその所有権を失いません。

#[derive(Debug, Clone, PartialEq, Eq)]
struct Parcel {
    id: String,
    destination: String,
    weight_kg: u16,
}

impl Parcel {
    fn new(id: &str, destination: &str, weight_kg: u16) -> Self {
        Self {
            id: id.to_string(),
            destination: destination.to_string(),
            weight_kg,
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn destination(&self) -> &str {
        &self.destination
    }

    fn weight_kg(&self) -> u16 {
        self.weight_kg
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LoadError {
    DuplicateId(Parcel),
    ShelfFull(Parcel),
}

#[derive(Debug, PartialEq, Eq)]
struct CargoShelf {
    max_slots: usize,
    parcels: Vec<Parcel>,
}

impl CargoShelf {
    fn new(max_slots: usize) -> Self {
        todo!("最大 {max_slots} 個分を事前確保した空の棚を作ってください")
    }

    fn len(&self) -> usize {
        todo!("棚にある荷物の個数を返してください")
    }

    fn is_empty(&self) -> bool {
        todo!("棚が空か Vec のメソッドで確認してください")
    }

    fn load(&mut self, parcel: Parcel) -> Result<(), LoadError> {
        todo!(
            "荷物 {} の重複と空きスロットを順番に確認してください",
            parcel.id()
        )
    }

    fn parcel_at(&self, index: usize) -> Option<&Parcel> {
        todo!("添字 {index} の荷物を安全に借用してください")
    }

    fn unload_last(&mut self) -> Option<Parcel> {
        todo!("棚の末尾から荷物を取り出してください")
    }

    fn unload_at(&mut self, index: usize) -> Option<Parcel> {
        todo!("添字 {index} が範囲内なら荷物を取り出してください")
    }

    fn total_weight_kg(&self) -> u32 {
        todo!("{} 個の荷物の重量を合計してください", self.parcels.len())
    }
}

fn main() {
    let mut shelf = CargoShelf::new(3);
    shelf
        .load(Parcel::new("P-401", "図書館", 2))
        .expect("空の棚には積み込める");
    shelf
        .load(Parcel::new("P-402", "研究棟", 5))
        .expect("空きスロットには積み込める");

    println!("棚: {shelf:#?}");
    println!("合計重量: {} kg", shelf.total_weight_kg());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parcel(id: &str, destination: &str, weight_kg: u16) -> Parcel {
        Parcel::new(id, destination, weight_kg)
    }

    #[test]
    fn 最大数を事前確保した空の棚を作る() {
        let shelf = CargoShelf::new(3);

        assert_eq!(shelf.max_slots, 3);
        assert_eq!(shelf.len(), 0);
        assert!(shelf.is_empty());
        assert!(shelf.parcels.capacity() >= 3);
    }

    #[test]
    fn 登録順を保ち添字で安全に借用する() {
        let mut shelf = CargoShelf::new(3);
        shelf.load(parcel("P-401", "図書館", 2)).unwrap();
        shelf.load(parcel("P-402", "研究棟", 5)).unwrap();
        shelf.load(parcel("P-403", "学生寮", 3)).unwrap();

        assert_eq!(shelf.len(), 3);
        assert!(!shelf.is_empty());
        assert_eq!(shelf.parcel_at(0).map(Parcel::id), Some("P-401"));
        assert_eq!(shelf.parcel_at(1).map(Parcel::id), Some("P-402"));
        assert_eq!(shelf.parcel_at(2).map(Parcel::id), Some("P-403"));
        assert_eq!(shelf.parcel_at(3), None);
        assert_eq!(shelf.len(), 3);
    }

    #[test]
    fn スロット数ゼロでは荷物の所有権をエラーで返す() {
        let mut shelf = CargoShelf::new(0);
        let rejected = parcel("P-410", "食堂", 4);

        assert_eq!(
            shelf.load(rejected),
            Err(LoadError::ShelfFull(parcel("P-410", "食堂", 4)))
        );
        assert!(shelf.is_empty());
    }

    #[test]
    fn 満杯でも容量エラーより重複idを優先する() {
        let mut shelf = CargoShelf::new(1);
        shelf.load(parcel("P-420", "図書館", 2)).unwrap();

        assert_eq!(
            shelf.load(parcel("P-420", "体育館", 9)),
            Err(LoadError::DuplicateId(parcel("P-420", "体育館", 9)))
        );
        assert_eq!(shelf.parcels, vec![parcel("P-420", "図書館", 2)]);
    }

    #[test]
    fn 最大数までは登録し超えた荷物をそのまま返す() {
        let mut shelf = CargoShelf::new(2);
        shelf.load(parcel("P-430", "図書館", 1)).unwrap();
        shelf.load(parcel("P-431", "研究棟", 2)).unwrap();

        assert_eq!(
            shelf.load(parcel("P-432", "保健センター", 3)),
            Err(LoadError::ShelfFull(parcel("P-432", "保健センター", 3)))
        );
        assert_eq!(shelf.len(), 2);
        assert_eq!(shelf.parcel_at(1).map(Parcel::id), Some("P-431"));
    }

    #[test]
    fn 末尾からlifo順に取り出し空ならnoneを返す() {
        let mut shelf = CargoShelf::new(2);
        assert_eq!(shelf.unload_last(), None);

        shelf.load(parcel("P-440", "図書館", 1)).unwrap();
        shelf.load(parcel("P-441", "研究棟", 2)).unwrap();

        assert_eq!(shelf.unload_last(), Some(parcel("P-441", "研究棟", 2)));
        assert_eq!(shelf.unload_last(), Some(parcel("P-440", "図書館", 1)));
        assert_eq!(shelf.unload_last(), None);
        assert!(shelf.is_empty());
    }

    #[test]
    fn 先頭と途中を取り出すと残りの順序を保つ() {
        let mut shelf = CargoShelf::new(4);
        for id in ["P-450", "P-451", "P-452", "P-453"] {
            shelf.load(parcel(id, "中央倉庫", 1)).unwrap();
        }

        assert_eq!(shelf.unload_at(0), Some(parcel("P-450", "中央倉庫", 1)));
        assert_eq!(shelf.unload_at(1), Some(parcel("P-452", "中央倉庫", 1)));
        assert_eq!(
            shelf.parcels.iter().map(Parcel::id).collect::<Vec<_>>(),
            vec!["P-451", "P-453"]
        );
    }

    #[test]
    fn 範囲外から取り出しても棚を変更しない() {
        let mut shelf = CargoShelf::new(2);
        shelf.load(parcel("P-460", "図書館", 2)).unwrap();

        assert_eq!(shelf.unload_at(1), None);
        assert_eq!(shelf.unload_at(usize::MAX), None);
        assert_eq!(shelf.parcels, vec![parcel("P-460", "図書館", 2)]);
    }

    #[test]
    fn 空と複数荷物の合計重量を求める() {
        let mut shelf = CargoShelf::new(3);
        assert_eq!(shelf.total_weight_kg(), 0);

        shelf.load(parcel("P-470", "図書館", 2)).unwrap();
        shelf.load(parcel("P-471", "研究棟", 5)).unwrap();
        shelf.load(parcel("P-472", "食堂", 3)).unwrap();

        assert_eq!(shelf.total_weight_kg(), 10);
    }

    #[test]
    fn 日本語と絵文字を含む文字列を保持する() {
        let mut shelf = CargoShelf::new(1);
        shelf
            .load(parcel("荷物📦-01", "工学部Ａ棟・受付", 2))
            .unwrap();

        let stored = shelf.parcel_at(0).expect("荷物が存在する");
        assert_eq!(stored.id(), "荷物📦-01");
        assert_eq!(stored.destination(), "工学部Ａ棟・受付");
        assert_eq!(stored.weight_kg(), 2);
    }
}
