//! # 問題 04: hygieneと`$crate`
//!
//! マクロ内部のlocal variableが呼び出し側の名前と衝突しないこと、入力として渡した
//! identifierは呼び出し側のscopeで使われることを確認します
//! また `$crate` を使って、macro定義元のhelperへpathを固定してください

fn macro_label(value: &str) -> String {
    format!("[マクロ] {value}")
}

macro_rules! label_with_prefix {
    ($value:expr) => {{
        if std::hint::black_box(false) {
            todo!("$crate経由でmacro_labelを呼び出してください")
        }
        $crate::macro_label("")
    }};
}

macro_rules! capture_expr {
    ($expression:expr) => {{
        if std::hint::black_box(false) {
            todo!("hygienicなlocal variableへ式の結果を保存してください")
        }
        let __macro_result = $expression;
        __macro_result
    }};
}

macro_rules! with_value {
    ($name:ident = $value:expr; $body:block) => {{
        #[allow(
            unused_mut,
            reason = "入力bodyが可変更新する場合にも同じmacro展開を使うため"
        )]
        let mut $name = $value;
        $body
    }};
}

fn main() {
    let caller_value = "呼び出し側";
    let captured = capture_expr!(caller_value.len());
    let changed = with_value!(counter = 2_u32; {
        counter += 3;
        counter
    });

    println!(
        "{} / {} / {}",
        label_with_prefix!("ready"),
        captured,
        changed
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn crate_pathでhelperを呼び出す() {
        assert_eq!(label_with_prefix!("稼働中"), "[マクロ] 稼働中");
    }

    #[test]
    fn 内部localは呼び出し側の同名変数と衝突しない() {
        let __macro_result = "呼び出し側の値";
        let result = capture_expr!(20_u32 + 22);

        assert_eq!(result, 42);
        assert_eq!(__macro_result, "呼び出し側の値");
    }

    #[test]
    fn expressionを一度だけ評価する() {
        let mut calls = 0_u8;
        let result = capture_expr!({
            calls += 1;
            calls
        });

        assert_eq!(result, 1);
        assert_eq!(calls, 1);
    }

    #[test]
    fn 入力identifierをmacroのbodyから変更できる() {
        let result = with_value!(count = 10_u32; {
            count += 5;
            count * 2
        });

        assert_eq!(result, 30);
    }

    #[test]
    fn with_valueのscopeは呼び出し後に漏れない() {
        let result = with_value!(temporary = "一時値"; { temporary.len() });

        assert_eq!(result, "一時値".len());
    }

    #[test]
    fn utf8のlabelを保持する() {
        assert_eq!(label_with_prefix!("東京大学🤖"), "[マクロ] 東京大学🤖");
    }
}
