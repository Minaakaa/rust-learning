#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 02: blanket implementationとnewtypeで監査表示を拡張する
//!
//! `Display`を実装するすべての型へ、監査用の文字列を作る`AuditText`を提供します
//! 個別の型を列挙せず、1つのblanket implementationでextension methodを追加してください
//!
//! 一方、標準ライブラリの`Display`を標準ライブラリの`Vec<ModuleStatus>`へ直接実装する
//! ことはorphan ruleにより禁止されています
//! そこでlocalな`StatusBoard` newtypeで`Vec`を包み、`Display`と`FromIterator`を実装します
//!
//! 仕様:
//! - `ModuleStatus`は`モジュール名: 状態`の形式で表示する
//! - 状態は`稼働中`、`要点検`、`停止中`のいずれかとする
//! - `AuditText::audit_text`は任意の`Display`値を`[監査] `で始まる文字列へ変換する
//! - blanket implementationは`str`のような`Sized`でない`Display`型にも適用する
//! - `StatusBoard`は`FromIterator<ModuleStatus>`で入力順を保って収集する
//! - 空のboardは`モジュール登録なし`と表示する
//! - 複数の状態は` | `で区切り、入力順に表示する
//! - 収集や取り出しのために`ModuleStatus`や名前を複製しない
//!
//! API設計上の注意:
//! - `AuditText`はこのcrateのlocal traitなので、foreignな型へも実装できる
//! - blanket implementation後に`AuditText for ModuleStatus`を追加すると実装が重複する
//! - blanket implementationは省略記法ではなく、将来追加できる実装にも影響する契約となる
//! - `StatusBoard`はlocal typeなので、foreign traitの`Display`を合法的に実装できる
//!
//! TODO:
//! - 用意された`T: Display + ?Sized`へのblanket implで監査用prefixを付ける
//! - `ModuleStatus`と`StatusBoard`の`Display`を実装する
//! - `FromIterator`で`ModuleStatus`を`StatusBoard`へ所有したまま収集する
//!
//! 設計確認:
//! - `?Sized`を一度外すと`<str as AuditText>::audit_text`がなぜcompileできないか確認する
//! - `Display for Vec<ModuleStatus>`ではなくnewtypeが必要な理由をorphan ruleから説明する

use std::fmt::{self, Display, Formatter};

trait AuditText {
    fn audit_text(&self) -> String;
}

impl<T> AuditText for T
where
    T: Display + ?Sized,
{
    fn audit_text(&self) -> String {
        todo!("Displayの出力へ監査用prefixを付けてください")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModuleState {
    Ready,
    NeedsInspection,
    Offline,
}

impl ModuleState {
    const fn label(self) -> &'static str {
        match self {
            Self::Ready => "稼働中",
            Self::NeedsInspection => "要点検",
            Self::Offline => "停止中",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ModuleStatus {
    name: String,
    state: ModuleState,
}

impl ModuleStatus {
    fn new(name: impl Into<String>, state: ModuleState) -> Self {
        Self {
            name: name.into(),
            state,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    const fn state(&self) -> ModuleState {
        self.state
    }
}

impl Display for ModuleStatus {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let _ = (self.name(), self.state().label());
        let _ = formatter;
        todo!("モジュール名と状態を表示してください")
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct StatusBoard(Vec<ModuleStatus>);

impl StatusBoard {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn iter(&self) -> std::slice::Iter<'_, ModuleStatus> {
        self.0.iter()
    }

    fn into_inner(self) -> Vec<ModuleStatus> {
        self.0
    }
}

impl FromIterator<ModuleStatus> for StatusBoard {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = ModuleStatus>,
    {
        let _ = iter;
        todo!("iteratorの全要素を入力順に所有してください")
    }
}

impl Display for StatusBoard {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let _ = formatter;
        todo!("空または複数の状態を仕様どおり表示してください")
    }
}

fn main() {
    let board: StatusBoard = [
        ModuleStatus::new("航法", ModuleState::Ready),
        ModuleStatus::new("荷物アーム", ModuleState::NeedsInspection),
    ]
    .into_iter()
    .collect();

    println!("{}", board.audit_text());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn statuses() -> Vec<ModuleStatus> {
        vec![
            ModuleStatus::new("航法", ModuleState::Ready),
            ModuleStatus::new("荷物アーム", ModuleState::NeedsInspection),
            ModuleStatus::new("通信", ModuleState::Offline),
        ]
    }

    #[test]
    fn module_statusをdisplayで整形する() {
        let ready = ModuleStatus::new("航法", ModuleState::Ready);
        let warning = ModuleStatus::new("荷物アーム", ModuleState::NeedsInspection);
        let offline = ModuleStatus::new("通信", ModuleState::Offline);

        assert_eq!(ready.to_string(), "航法: 稼働中");
        assert_eq!(warning.to_string(), "荷物アーム: 要点検");
        assert_eq!(offline.to_string(), "通信: 停止中");
        assert_eq!(ready.name(), "航法");
        assert_eq!(ready.state(), ModuleState::Ready);
    }

    #[test]
    fn display型へaudit_textがblanket実装される() {
        let status = ModuleStatus::new("安全scanner", ModuleState::NeedsInspection);

        assert_eq!(status.audit_text(), "[監査] 安全scanner: 要点検");
        assert_eq!(42_u16.audit_text(), "[監査] 42");
    }

    #[test]
    fn sizedでないstrにもextension_methodを使える() {
        let note: &str = "手動点検を実施";

        assert_eq!(
            <str as AuditText>::audit_text(note),
            "[監査] 手動点検を実施"
        );
    }

    #[test]
    fn from_iteratorでlocal_newtypeへ収集する() {
        let board: StatusBoard = statuses().into_iter().collect();

        assert_eq!(board.len(), 3);
        assert!(!board.is_empty());
        assert_eq!(
            board.iter().map(ModuleStatus::name).collect::<Vec<_>>(),
            ["航法", "荷物アーム", "通信"]
        );
    }

    #[test]
    fn status_boardを入力順のままdisplayする() {
        let board: StatusBoard = statuses().into_iter().collect();

        assert_eq!(
            board.to_string(),
            "航法: 稼働中 | 荷物アーム: 要点検 | 通信: 停止中"
        );
    }

    #[test]
    fn 空のstatus_boardを明示的な文言でdisplayする() {
        let board = StatusBoard::default();

        assert!(board.is_empty());
        assert_eq!(board.len(), 0);
        assert_eq!(board.to_string(), "モジュール登録なし");
    }

    #[test]
    fn status_boardにも同じblanket実装が適用される() {
        let board: StatusBoard = statuses().into_iter().take(2).collect();

        assert_eq!(
            board.audit_text(),
            "[監査] 航法: 稼働中 | 荷物アーム: 要点検"
        );
    }

    #[test]
    fn newtypeへの収集と取り出しでnameを複製しない() {
        let name = String::from("所有中の制御module");
        let name_pointer = name.as_ptr();
        let status = ModuleStatus::new(name, ModuleState::Ready);

        let board: StatusBoard = std::iter::once(status).collect();
        let mut returned = board.into_inner();
        let returned = returned.pop().expect("1件の状態がある");

        assert_eq!(returned.name(), "所有中の制御module");
        assert_eq!(returned.name.as_ptr(), name_pointer);
    }

    #[test]
    fn utf8のmodule名を順序と内容を変えず表示する() {
        let board: StatusBoard = [
            ModuleStatus::new("本郷・航法🧭", ModuleState::Ready),
            ModuleStatus::new("駒場・搬送アーム🦾", ModuleState::Offline),
        ]
        .into_iter()
        .collect();

        assert_eq!(
            board.audit_text(),
            "[監査] 本郷・航法🧭: 稼働中 | 駒場・搬送アーム🦾: 停止中"
        );
    }
}
