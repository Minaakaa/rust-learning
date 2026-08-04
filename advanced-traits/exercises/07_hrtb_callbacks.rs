//! # 問題 07: HRTBでborrowを返すcallbackを設計する
//!
//! robotのtelemetry文字列から、所有権を移さずに必要な部分だけを選ぶcallbackを扱います
//! callbackは入力の`&str`を受け取り、同じ入力内の`&str`を返します
//!
//! ## なぜ`for<'a>`が必要か
//!
//! `F: Fn(&'input str) -> &'input str`は、1回の呼び出しについて呼び出し側が選んだ
//! `'input`で`F`を使えることだけを表します
//! 一方、`for<'a> F: Fn(&'a str) -> &'a str`は、`F`が**すべての**`'a`で
//! 呼び出せることを表すhigher-ranked trait bound（HRTB）です
//! そのため、同じcallbackを長く生きる文字列にも、関数内で作った短命な`String`にも使えます
//!
//! 入力と出力に同じ`'a`を書くことで、APIから見える戻り値のlifetimeは各入力へ結び付きます
//! localな`String`から得たsliceを、その`String`より外へ持ち出すことはできません
//! ただし`'static`な文字列も任意の`'a`へ短縮できるため、sliceの由来が入力内であること
//! そのものは型だけでは証明されず、この演習ではcallbackの振る舞いとして要求します
//!
//! ## trait objectとの組み合わせ
//!
//! `dyn for<'a> Fn(&'a str) -> &'a str`は、具体的なcallback型を消去した後も
//! 「任意の短いborrowで呼べる」という契約を保持します
//! この問題の`CallbackRack`は異なるfunction itemを登録順に所有し、同じ入力へ順番に適用します
//!
//! 仕様:
//! - `apply_view`はHRTBを持つcallbackを1つの入力へ適用する
//! - `apply_pair`はlifetimeが独立した2つの入力へ同じcallbackを適用する
//! - `CallbackRack::register`は`Send + Sync + 'static`なcallbackを型消去して所有する
//! - `CallbackRack::apply_all`は全callbackを同じ入力へ適用し、登録順を保つ
//! - 各callbackは新しい`String`を作らず、必ず入力内のsliceを返す
//!
//! ヒント:
//! - `for<'a>`は「ある1つの`'a`」ではなく「すべての`'a`」を表す
//! - `F: ?Sized`を加えると、generic関数へ`dyn Fn`のborrowも渡せる
//! - `Box`は異なるcallback型をrackが所有するために使い、入力文字列は所有しない
//! - borrowを返すclosureを期待型なしで先に変数へ束縛すると、入出力lifetimeの関係を
//!   意図どおり推論できない場合がある
//! - この演習のようにnamed functionを使うか、`for<'a> fn(&'a str) -> &'a str`と
//!   function pointer型を明示すると、HRTBの契約が明確になる

type DynTextView = dyn for<'a> Fn(&'a str) -> &'a str + Send + Sync;

fn apply_view<'input, F>(callback: &F, input: &'input str) -> &'input str
where
    for<'a> F: Fn(&'a str) -> &'a str,
    F: ?Sized,
{
    let _ = (callback, input);
    todo!("HRTBを持つcallbackを入力へ適用してください")
}

fn apply_pair<'left, 'right, F>(
    callback: &F,
    left: &'left str,
    right: &'right str,
) -> (&'left str, &'right str)
where
    for<'a> F: Fn(&'a str) -> &'a str,
    F: ?Sized,
{
    let _ = (callback, left, right);
    todo!("lifetimeが独立した2つの入力へ同じcallbackを適用してください")
}

#[derive(Default)]
struct CallbackRack {
    callbacks: Vec<Box<DynTextView>>,
}

impl CallbackRack {
    const fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    fn register<F>(&mut self, callback: F)
    where
        for<'a> F: Fn(&'a str) -> &'a str,
        F: Send + Sync + 'static,
    {
        let _ = callback;
        todo!("callbackをtrait objectへ型消去して登録してください")
    }

    fn len(&self) -> usize {
        self.callbacks.len()
    }

    fn is_empty(&self) -> bool {
        self.callbacks.is_empty()
    }

    fn apply_all<'input>(&self, input: &'input str) -> Vec<&'input str> {
        let _ = input;
        todo!("全callbackを登録順に動的dispatchしてください")
    }
}

fn identity(input: &str) -> &str {
    input
}

fn trim_edges(input: &str) -> &str {
    input.trim()
}

fn first_line(input: &str) -> &str {
    input
        .split_once('\n')
        .map_or(input, |(line, _remaining)| line)
}

fn first_word(input: &str) -> &str {
    match input.split_whitespace().next() {
        Some(word) => word,
        None => &input[..0],
    }
}

fn main() {
    let mut rack = CallbackRack::new();
    rack.register(trim_edges);
    rack.register(first_line);
    rack.register(first_word);
    rack.register(identity);

    let telemetry = String::from("  温度: 42 ℃\n状態: 正常  ");
    let direct = apply_view(&trim_edges, &telemetry);
    let (heading, status) = apply_pair(&first_line, &telemetry, "状態: 正常\n次の行");

    println!("登録callback数: {}", rack.len());
    println!("空のrack: {}", rack.is_empty());
    println!("直接適用: {direct}");
    println!("2つの入力: {heading:?}, {status:?}");
    println!("動的dispatch: {:?}", rack.apply_all(&telemetry));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genericなcallbackを異なるlifetimeへ適用できる() {
        let outer = String::from("outer\nignored");
        let outer_view;

        {
            let local = String::from("local\nignored");
            let (left, right) = apply_pair(&first_line, &outer, &local);
            outer_view = left;
            assert_eq!(right, "local");
        }

        assert_eq!(outer_view, "outer");
    }

    #[test]
    fn 複数callbackを登録順に実行する() {
        let mut rack = CallbackRack::new();
        rack.register(trim_edges);
        rack.register(first_line);
        rack.register(first_word);
        rack.register(identity);

        let input = "  温度: 高\n停止  ";

        assert_eq!(
            rack.apply_all(input),
            ["温度: 高\n停止", "  温度: 高", "温度:", input]
        );
        assert_eq!(rack.len(), 4);
    }

    #[test]
    fn 空のrackは空の結果を返す() {
        let rack = CallbackRack::new();

        assert!(rack.is_empty());
        assert_eq!(rack.apply_all("telemetry"), Vec::<&str>::new());
    }

    #[test]
    fn localなstringの短いborrowを処理できる() {
        let mut rack = CallbackRack::new();
        rack.register(first_line);
        rack.register(first_word);

        let local = String::from("警告 温度上昇\n詳細");
        let views = rack.apply_all(&local);

        assert_eq!(views, ["警告 温度上昇", "警告"]);
    }

    #[test]
    fn temporaryなstringは同じ式の中で処理できる() {
        let selected = apply_view(&trim_edges, String::from("  ready  ").as_str()).to_owned();

        assert_eq!(selected, "ready");
    }

    #[test]
    fn utf8の文字境界を保ったsliceを返す() {
        let mut rack = CallbackRack::new();
        rack.register(trim_edges);
        rack.register(first_line);
        rack.register(first_word);

        let views = rack.apply_all("  東京大学 ロボット班\n🚀 発進  ");

        assert_eq!(
            views,
            [
                "東京大学 ロボット班\n🚀 発進",
                "  東京大学 ロボット班",
                "東京大学"
            ]
        );
    }

    #[test]
    fn 空文字列でも各callbackを呼べる() {
        let mut rack = CallbackRack::new();
        rack.register(identity);
        rack.register(trim_edges);
        rack.register(first_line);
        rack.register(first_word);

        assert_eq!(rack.apply_all(""), ["", "", "", ""]);
        assert_eq!(first_word("   \t"), "");
    }

    #[test]
    fn 戻り値は入力と同じallocation内のrangeである() {
        let input = String::from("  警告  ");
        let selected = apply_view(&trim_edges, &input);
        let expected = &input[2..input.len() - 2];

        assert!(std::ptr::eq(selected.as_ptr(), expected.as_ptr()));
        assert_eq!(selected.len(), expected.len());
        assert_eq!(selected, "警告");
    }

    #[test]
    fn dyn_callbackを複数のlocal入力へ再利用できる() {
        let callback: Box<DynTextView> = Box::new(first_line);

        {
            let first = String::from("alpha\nrest");
            assert_eq!(apply_view(callback.as_ref(), &first), "alpha");
        }

        {
            let second = String::from("beta\nrest");
            assert_eq!(apply_view(callback.as_ref(), &second), "beta");
        }
    }

    #[test]
    fn function_itemをhrtbなfunction_pointerへ変換できる() {
        let callback: for<'a> fn(&'a str) -> &'a str = first_line;

        assert_eq!(apply_view(&callback, "first\nsecond"), "first");
    }

    #[test]
    fn rackとcallbackはsendかつsyncである() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}

        assert_send_sync::<DynTextView>();
        assert_send_sync::<CallbackRack>();
    }
}
