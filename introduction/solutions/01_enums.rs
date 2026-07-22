//! 問題 01 の解答例。

#[derive(Debug, Clone, PartialEq, Eq)]
enum Cargo {
    Documents { title: String, pages: u16 },
    Components(String, u16, bool),
    Empty,
}

impl Cargo {
    fn units(&self) -> u16 {
        match self {
            Self::Documents { .. } => 1,
            Self::Components(_, count, _) => *count,
            Self::Empty => 0,
        }
    }

    fn label(&self) -> String {
        match self {
            Self::Documents { title, pages } => format!("書類「{title}」({pages}ページ)"),
            Self::Components(name, count, _) => format!("部品「{name}」x{count}"),
            Self::Empty => "空荷".to_string(),
        }
    }

    fn needs_care(&self) -> bool {
        match self {
            Self::Components(_, _, fragile) => *fragile,
            Self::Documents { .. } | Self::Empty => false,
        }
    }
}

fn sample_cargo() -> Vec<Cargo> {
    vec![
        Cargo::Documents {
            title: "Rust入門ノート".to_string(),
            pages: 48,
        },
        Cargo::Components("距離センサー".to_string(), 3, true),
        Cargo::Empty,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 指定された荷物を構築できる() {
        assert_eq!(
            sample_cargo(),
            vec![
                Cargo::Documents {
                    title: "Rust入門ノート".to_string(),
                    pages: 48,
                },
                Cargo::Components("距離センサー".to_string(), 3, true),
                Cargo::Empty,
            ]
        );
    }

    #[test]
    fn バリアントごとの個数を返す() {
        let cargo = sample_cargo();
        assert_eq!(cargo[0].units(), 1);
        assert_eq!(cargo[1].units(), 3);
        assert_eq!(cargo[2].units(), 0);
    }

    #[test]
    fn 中のデータを使ってラベルを作る() {
        let cargo = sample_cargo();
        assert_eq!(cargo[0].label(), "書類「Rust入門ノート」(48ページ)");
        assert_eq!(cargo[1].label(), "部品「距離センサー」x3");
        assert_eq!(cargo[2].label(), "空荷");
    }

    #[test]
    fn 精密部品のフラグだけを取扱注意に反映する() {
        assert!(Cargo::Components("カメラ".to_string(), 1, true).needs_care());
        assert!(!Cargo::Components("ねじ".to_string(), 20, false).needs_care());
        assert!(
            !Cargo::Documents {
                title: "提出レポート".to_string(),
                pages: 12,
            }
            .needs_care()
        );
        assert!(!Cargo::Empty.needs_care());
    }
}
