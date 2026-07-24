//! 問題 01 の解答例。

#[derive(Debug, PartialEq, Eq)]
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
}

#[derive(Debug, PartialEq, Eq)]
struct CargoBag {
    owner: String,
    parcels: Vec<Parcel>,
}

impl CargoBag {
    fn new(owner: &str) -> Self {
        Self {
            owner: owner.to_string(),
            parcels: Vec::new(),
        }
    }
}

fn load(mut bag: CargoBag, parcel: Parcel) -> CargoBag {
    bag.parcels.push(parcel);
    bag
}

fn unload_last(mut bag: CargoBag) -> (CargoBag, Option<Parcel>) {
    let parcel = bag.parcels.pop();
    (bag, parcel)
}

fn transfer_all(mut source: CargoBag, mut destination: CargoBag) -> (CargoBag, CargoBag) {
    destination.parcels.append(&mut source.parcels);
    (source, destination)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 荷物をバッグへムーブする() {
        let bag = CargoBag::new("RB-01");
        let parcel = Parcel::new("P-100", "図書館", 3);

        let bag = load(bag, parcel);

        assert_eq!(bag.owner, "RB-01");
        assert_eq!(bag.parcels, vec![Parcel::new("P-100", "図書館", 3)]);
    }

    #[test]
    fn バッグと降ろした荷物を両方返す() {
        let bag = load(
            load(CargoBag::new("RB-02"), Parcel::new("P-200", "研究棟", 5)),
            Parcel::new("P-201", "学生寮", 2),
        );

        let (bag, parcel) = unload_last(bag);

        assert_eq!(bag.parcels, vec![Parcel::new("P-200", "研究棟", 5)]);
        assert_eq!(parcel, Some(Parcel::new("P-201", "学生寮", 2)));
    }

    #[test]
    fn 空のバッグから降ろしてもバッグを失わない() {
        let (bag, parcel) = unload_last(CargoBag::new("RB-03"));

        assert_eq!(bag.owner, "RB-03");
        assert!(bag.parcels.is_empty());
        assert_eq!(parcel, None);
    }

    #[test]
    fn 全荷物を順序どおり別のバッグへ移す() {
        let source = load(
            load(CargoBag::new("RB-04"), Parcel::new("P-401", "食堂", 1)),
            Parcel::new("P-402", "体育館", 4),
        );
        let destination = load(CargoBag::new("RB-05"), Parcel::new("P-500", "図書館", 2));

        let (source, destination) = transfer_all(source, destination);

        assert_eq!(source.owner, "RB-04");
        assert!(source.parcels.is_empty());
        assert_eq!(
            destination.parcels,
            vec![
                Parcel::new("P-500", "図書館", 2),
                Parcel::new("P-401", "食堂", 1),
                Parcel::new("P-402", "体育館", 4),
            ]
        );
    }
}
