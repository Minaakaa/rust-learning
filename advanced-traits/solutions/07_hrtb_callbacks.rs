//! # 解答 07: HRTBでborrowを返すcallbackを設計する
//!
//! `for<'a> F: Fn(&'a str) -> &'a str`は、同じcallbackが任意のlifetimeで
//! 入力を借用し、その入力のlifetimeに結び付いたsliceを返せることを表します
//! `dyn for<'a> Fn(&'a str) -> &'a str`へ型消去しても、この契約は保たれます
//! `'static`なsliceもこの境界を満たすため、値の由来が入力内であること自体は別の契約です

type DynTextView = dyn for<'a> Fn(&'a str) -> &'a str + Send + Sync;

fn apply_view<'input, F>(callback: &F, input: &'input str) -> &'input str
where
    for<'a> F: Fn(&'a str) -> &'a str,
    F: ?Sized,
{
    callback(input)
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
    (callback(left), callback(right))
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
        self.callbacks.push(Box::new(callback));
    }

    fn len(&self) -> usize {
        self.callbacks.len()
    }

    fn is_empty(&self) -> bool {
        self.callbacks.is_empty()
    }

    fn apply_all<'input>(&self, input: &'input str) -> Vec<&'input str> {
        self.callbacks
            .iter()
            .map(|callback| callback(input))
            .collect()
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
