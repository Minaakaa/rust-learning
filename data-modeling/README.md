# Chapter 3 — 壊れにくいデータ設計

このコースでは、配送ロボットの業務ルールを Rust の型として表現します。単に値をまとめるのではなく、「不正な状態を作りにくいか」「操作の意味が API から分かるか」「メートルと秒を取り違えないか」を考えながら設計することが目標です。

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | 構造体 | 名前付きフィールドと入れ子の構造体で業務データを表す |
| 02 | `impl` とメソッド | 関連関数、`&self`、`&mut self`、`self` を使い分ける |
| 03 | newtype | 同じ内部表現を持つ単位を、異なる型として区別する |
| 04 | 不変条件 | 検証付きコンストラクタとメソッドで常に正しい状態を保つ |
| 05 | 総合課題 | 型安全なミッションモデルと配送 API を完成させる |

## この章の考え方

- 関係する値は、意味のある名前を持つ構造体にまとめる。
- オブジェクトを作る処理は `Type::new` のような関連関数に置く。
- 読む操作には `&self`、更新には `&mut self`、値を別の形へ変換し終える操作には `self` を使う。
- メートル、秒、バッテリー残量などを裸の整数だけで表さず、必要なら newtype で区別する。
- 検証はデータの入口で行い、その後のメソッドも不変条件を壊さないようにする。
- `Debug`、`Clone`、`Copy`、`PartialEq`、`Eq`、`Ord` などは、意味が合う場合に `derive` する。

この章では、フィールドを直接変更してテストだけを通すのではなく、指定されたコンストラクタとメソッドを実装してください。

## 進め方

プロジェクトのルートで、最初の問題を実行します。

```console
cargo test --example data_modeling_01_structs
```

未完成の問題は `not yet implemented` で失敗します。問題ファイルの仕様とテストを読み、`todo!()` を置き換えてください。

```console
cargo test --example data_modeling_02_methods
cargo test --example data_modeling_03_newtypes
cargo test --example data_modeling_04_invariants
cargo test --example data_modeling_05_mission
```

問題は Cargo の example target として分離されています。`cargo test` だけでは問題を実行しないため、必ず `--example data_modeling_...` を指定してください。

## 解答例

同名の解答例が `data-modeling/solutions` にあります。たとえば問題 04 の解答だけを検証するには、次を実行します。

```console
cargo test --test solution_data_modeling_04_invariants
```

全コースの解答例は、次のコマンドでまとめて検証できます。

```console
cargo test --tests
```

`cargo test --all-targets` は未完成の問題も実行するため、問題を解き終えるまでは失敗します。

## 公式リファレンス

- [構造体を使って関連するデータを構造化する — The Rust Programming Language](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [メソッド構文](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)
- [`derive` 属性 — The Rust Reference](https://doc.rust-lang.org/reference/attributes/derive.html)
- [newtype による型安全性と抽象化](https://doc.rust-lang.org/book/ch20-03-advanced-types.html#using-the-newtype-pattern-for-type-safety-and-abstraction)
- [Rust API Guidelines — 引数を検証する](https://rust-lang.github.io/api-guidelines/dependability.html#functions-validate-their-arguments-c-validate)
