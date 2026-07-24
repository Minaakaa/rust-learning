//! 問題 03 の解答例。

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

fn total_weight(manifest: &Manifest) -> u32 {
    manifest
        .parcels
        .iter()
        .map(|parcel| u32::from(parcel.weight_kg))
        .sum()
}

fn heaviest(parcels: &[Parcel]) -> Option<&Parcel> {
    parcels.iter().max_by_key(|parcel| parcel.weight_kg)
}

fn first_batch(parcels: &[Parcel], max_count: usize) -> &[Parcel] {
    &parcels[..max_count.min(parcels.len())]
}

fn area_name(label: &str) -> &str {
    label.split_once(':').map_or(label, |(area, _)| area).trim()
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
