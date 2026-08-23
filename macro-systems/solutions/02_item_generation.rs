//! # 解答 02: 反復で項目を生成する

macro_rules! define_module_state {
    ($( $name:ident => $label:literal ),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum ModuleState {
            $( $name, )+
        }

        impl ModuleState {
            fn label(self) -> &'static str {
                match self {
                    $( Self::$name => $label, )+
                }
            }

            fn is_terminal(self) -> bool {
                matches!(self, Self::Offline)
            }
        }
    };
}

define_module_state! {
    Ready => "稼働中",
    Inspecting => "点検中",
    Offline => "停止中",
}

fn main() {
    let states = [
        ModuleState::Ready,
        ModuleState::Inspecting,
        ModuleState::Offline,
    ];
    for state in states {
        println!("{} / 終端={}", state.label(), state.is_terminal());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 生成されたvariantのlabelを返す() {
        assert_eq!(ModuleState::Ready.label(), "稼働中");
        assert_eq!(ModuleState::Inspecting.label(), "点検中");
        assert_eq!(ModuleState::Offline.label(), "停止中");
    }

    #[test]
    fn 停止状態だけterminalになる() {
        assert!(!ModuleState::Ready.is_terminal());
        assert!(!ModuleState::Inspecting.is_terminal());
        assert!(ModuleState::Offline.is_terminal());
    }

    #[test]
    fn 生成されたenumはcopyと比較が使える() {
        let original = ModuleState::Ready;
        let copied = original;

        assert_eq!(original, copied);
        assert_ne!(original, ModuleState::Offline);
    }

    #[test]
    fn 生成順を配列に保持できる() {
        let labels: Vec<_> = [ModuleState::Offline, ModuleState::Ready]
            .into_iter()
            .map(ModuleState::label)
            .collect();

        assert_eq!(labels, ["停止中", "稼働中"]);
    }
}
