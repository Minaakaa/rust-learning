#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 01: ムーブで荷物を引き渡す
//!
//! `Parcel` と `CargoBag` は `String` や `Vec` を所有するため、`Copy` ではありません。
//! 関数へ値として渡すと所有権がムーブします。この問題では、受け取った値を別の所有者へ
//! 移すか、戻り値として呼び出し元へ返してください。
//!
//! 制約:
//! - `Parcel` と `CargoBag` に `Clone` や `Copy` を追加しない。
//! - 文字列から同じ荷物を作り直さない。
//! - `load` と `unload_last` はバッグ自体の所有権も受け取り、戻り値で返す。
//! - `transfer_all` は荷物の順序を保つ。

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

/// バッグと荷物を受け取り、荷物を積んだバッグの所有権を返す。
fn load(mut bag: CargoBag, parcel: Parcel) -> CargoBag {
    // 未実装でも `mut` が必要な形を保つための行。実装時に削除してよい。
    let _ = &mut bag;
    todo!(
        "{} のバッグへ荷物 {} をムーブしてください",
        bag.owner,
        parcel.id
    )
}

/// 最後の荷物を降ろし、バッグと荷物の両方の所有権を返す。
fn unload_last(mut bag: CargoBag) -> (CargoBag, Option<Parcel>) {
    // 未実装でも `mut` が必要な形を保つための行。実装時に削除してよい。
    let _ = &mut bag;
    todo!("{} のバッグから最後の荷物を取り出してください", bag.owner)
}

/// `source` の全荷物を `destination` の末尾へ移し、両方のバッグを返す。
fn transfer_all(mut source: CargoBag, mut destination: CargoBag) -> (CargoBag, CargoBag) {
    // 未実装でも `mut` が必要な形を保つための行。実装時に削除してよい。
    let _ = (&mut source, &mut destination);
    todo!(
        "{} 個を {} から {} へムーブしてください",
        source.parcels.len(),
        source.owner,
        destination.owner
    )
}

fn main() {
    let bag = CargoBag::new("RB-01");
    let parcel = Parcel::new("P-100", "図書館", 3);
    let bag = load(bag, parcel);
    let (bag, parcel) = unload_last(bag);

    println!("残りの荷物: {}", bag.parcels.len());
    println!("降ろした荷物: {parcel:?}");
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
