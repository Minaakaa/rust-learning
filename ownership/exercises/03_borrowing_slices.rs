#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 03: 共有借用とスライスで読む
//!
//! 所有権を必要としない読み取り関数は、値そのものではなく `&T` を受け取ります。
//! `&[T]` は配列や `Vec<T>` の連続した一部分を借用するスライス、`&str` は UTF-8
//! 文字列を借用するスライスです。
//!
//! 制約:
//! - `Parcel` と `Manifest` に `Clone` を追加しない。
//! - 引数を所有する `Vec<Parcel>` や `String` に変更しない。
//! - 戻り値のスライスや参照のために、新しいコレクションを作らない。
//! - 範囲外の件数が渡されても `first_batch` をパニックさせない。
//!
//! 戻り値の参照がどの入力を借用するかは、この問題ではライフタイム省略規則によって
//! コンパイラが推論します。明示的なライフタイムは後の章で扱います。

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
struct Manifest {
    robot_id: String,
    parcels: Vec<Parcel>,
}

impl Manifest {
    fn new(robot_id: &str, parcels: Vec<Parcel>) -> Self {
        Self {
            robot_id: robot_id.to_string(),
            parcels,
        }
    }
}

/// マニフェストを借用し、全荷物の重量を返す。
fn total_weight(manifest: &Manifest) -> u32 {
    todo!(
        "{} の荷物を借用して重量を合計してください",
        manifest.robot_id
    )
}

/// スライスから最も重い荷物への参照を返す。
fn heaviest(parcels: &[Parcel]) -> Option<&Parcel> {
    todo!(
        "{} 個の荷物から最重量の要素を借用してください",
        parcels.len()
    )
}

/// 先頭から最大 `max_count` 件までを、新しいスライスとして借用する。
fn first_batch(parcels: &[Parcel], max_count: usize) -> &[Parcel] {
    todo!(
        "{} 個のうち先頭 {max_count} 個までをスライスしてください",
        parcels.len()
    )
}

/// `"エリア:建物"` ならエリア部分、区切りがなければ文字列全体を借用して返す。
///
/// 返す前にエリア部分の前後の空白を除く。
fn area_name(label: &str) -> &str {
    todo!("「{label}」からエリア名の文字列スライスを返してください")
}

fn main() {
    let manifest = Manifest::new(
        "RB-10",
        vec![
            Parcel::new("P-10", "図書館", 2),
            Parcel::new("P-11", "研究棟", 7),
        ],
    );

    println!("総重量: {} kg", total_weight(&manifest));
    println!("最重量: {:?}", heaviest(&manifest.parcels));
    println!("先頭便: {:?}", first_batch(&manifest.parcels, 1));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> Manifest {
        Manifest::new(
            "RB-10",
            vec![
                Parcel::new("P-10", "図書館", 2),
                Parcel::new("P-11", "研究棟", 7),
                Parcel::new("P-12", "学生寮", 4),
            ],
        )
    }

    #[test]
    fn 所有権を奪わず重量を合計する() {
        let manifest = manifest();

        assert_eq!(total_weight(&manifest), 13);
        assert_eq!(manifest.robot_id, "RB-10");
        assert_eq!(manifest.parcels.len(), 3);
    }

    #[test]
    fn 最も重い荷物への参照を返す() {
        let manifest = manifest();

        let parcel = heaviest(&manifest.parcels).expect("荷物が存在する");

        assert_eq!(parcel.id, "P-11");
        assert_eq!(parcel.weight_kg, 7);
        assert_eq!(manifest.parcels.len(), 3);
    }

    #[test]
    fn 空のスライスには最重量の荷物がない() {
        assert_eq!(heaviest(&[]), None);
    }

    #[test]
    fn 件数を安全な範囲に収めてスライスする() {
        let manifest = manifest();

        assert_eq!(first_batch(&manifest.parcels, 2), &manifest.parcels[0..2]);
        assert_eq!(first_batch(&manifest.parcels, 99), &manifest.parcels[..]);
        assert!(first_batch(&manifest.parcels, 0).is_empty());
    }

    #[test]
    fn 複数の共有借用を同時に使える() {
        let manifest = manifest();
        let batch = first_batch(&manifest.parcels, 2);
        let heavy = heaviest(&manifest.parcels).expect("荷物が存在する");

        assert_eq!(batch[0].id, "P-10");
        assert_eq!(heavy.id, "P-11");
        assert_eq!(total_weight(&manifest), 13);
    }

    #[test]
    fn utf8文字列の一部をstrとして借用する() {
        let label = String::from("  北キャンパス : 図書館");

        assert_eq!(area_name(&label), "北キャンパス");
        assert_eq!(area_name("研究棟"), "研究棟");
        assert_eq!(label, "  北キャンパス : 図書館");
    }
}
