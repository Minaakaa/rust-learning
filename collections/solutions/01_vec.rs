//! 問題 01 の解答例。

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
        Self {
            max_slots,
            parcels: Vec::with_capacity(max_slots),
        }
    }

    fn len(&self) -> usize {
        self.parcels.len()
    }

    fn is_empty(&self) -> bool {
        self.parcels.is_empty()
    }

    fn load(&mut self, parcel: Parcel) -> Result<(), LoadError> {
        if self.parcels.iter().any(|stored| stored.id() == parcel.id()) {
            return Err(LoadError::DuplicateId(parcel));
        }

        if self.parcels.len() >= self.max_slots {
            return Err(LoadError::ShelfFull(parcel));
        }

        self.parcels.push(parcel);
        Ok(())
    }

    fn parcel_at(&self, index: usize) -> Option<&Parcel> {
        self.parcels.get(index)
    }

    fn unload_last(&mut self) -> Option<Parcel> {
        self.parcels.pop()
    }

    fn unload_at(&mut self, index: usize) -> Option<Parcel> {
        if index < self.parcels.len() {
            Some(self.parcels.remove(index))
        } else {
            None
        }
    }

    fn total_weight_kg(&self) -> u32 {
        self.parcels
            .iter()
            .map(|parcel| u32::from(parcel.weight_kg()))
            .sum()
    }
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
