# Chapter 2 — 所有権を実践する

このコースでは、配送ロボットの荷物を題材に、Rust の所有権を実際のデータ構造で練習します。単にコンパイラエラーを消すのではなく、「今この値を所有しているのは誰か」「借用で十分か」「複製は本当に必要か」を説明できることが目標です。

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | ムーブ | 値を関数やコレクションへ移し、必要なら所有権を戻す |
| 02 | `Copy` と `Clone` | 暗黙のコピーと明示的な複製を使い分ける |
| 03 | 共有借用とスライス | 所有権を奪わずに値を読み、コレクションの一部を借用する |
| 04 | 可変借用 | `&mut T` で排他的に更新し、同じ参照を順番に再借用する |
| 05 | 総合課題 | 値を複製せず、待機列・ロボット・配送済み記録の間で移動する |

## この章の考え方

- 値は原則として 1 つの所有者を持つ。
- 代入や関数呼び出しでは、`Copy` 型でなければ所有権がムーブする。
- 読むだけなら `&T`、更新するなら `&mut T` を使う。
- 同時に存在できるのは、複数の共有参照か、1 つの可変参照のどちらかである。
- `clone()` は借用エラーを黙らせる道具ではない。独立した複製が必要なときだけ使う。

問題 02 以外の解答では、荷物を複製するための `Clone` を使いません。値がどこへムーブしたかを追跡してください。

## 進め方

プロジェクトのルートで、最初の問題を実行します。

```console
cargo test --example ownership_01_moves
```

未完成の問題は `not yet implemented` で失敗します。問題ファイルの説明とテストを読み、`todo!()` を置き換えてください。順番に実行するコマンドは次のとおりです。

```console
cargo test --example ownership_02_copy_clone
cargo test --example ownership_03_borrowing_slices
cargo test --example ownership_04_mutable_borrowing
cargo test --example ownership_05_handoff
```

問題は Cargo の example target として分離されています。`cargo test` だけでは問題を実行しないため、必ず `--example ownership_...` を指定してください。

所有権エラーが出たときは、エラー番号の説明も役立ちます。

```console
rustc --explain E0382
rustc --explain E0499
rustc --explain E0502
```

## 解答例

同名の解答例が `ownership/solutions` にあります。たとえば問題 03 の解答だけを検証するには、次を実行します。

```console
cargo test --test solution_ownership_03_borrowing_slices
```

全コースの解答例は、次のコマンドでまとめて検証できます。

```console
cargo test --tests
```

`cargo test --all-targets` は未完成の問題も実行するため、問題を解き終えるまでは失敗します。

## 公式リファレンス

- [所有権を理解する — The Rust Programming Language](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [参照と借用](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [スライス型](https://doc.rust-lang.org/book/ch04-03-slices.html)
- [`Copy` トレイト](https://doc.rust-lang.org/std/marker/trait.Copy.html)
- [`Clone` トレイト](https://doc.rust-lang.org/std/clone/trait.Clone.html)
