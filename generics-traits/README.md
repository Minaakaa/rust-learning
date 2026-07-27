# Chapter 5 — ジェネリクスとトレイトで配送ルールを拡張する

このコースでは、荷物の中身や配送規則が変わっても再利用できる型と関数を設計します。`Cargo<T>` から始め、独自トレイト、トレイト境界、標準変換トレイトを段階的に学び、最後に交換可能な配送ポリシーを持つ `Dispatcher<P>` を完成させます

この章の `Cargo<T>` は荷物を表す教材内の型で、Rust のビルドツール Cargo とは別のものです

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | ジェネリック型 | `Cargo<T>` で異なる型を安全に保持し、借用・所有権・型の変換を使い分ける |
| 02 | 独自トレイト | 異なる型へ共通の振る舞いを実装し、デフォルトメソッドを活用する |
| 03 | トレイト境界 | `impl Trait`、名前付き型引数、`where` を目的に応じて使い分ける |
| 04 | 標準変換トレイト | `From` / `Into` と `TryFrom` / `TryInto` で変換 API を設計する |
| 05 | 静的多相 | `Dispatcher<P>` に複数の配送ポリシーを差し替え、同点と順序を決定的に扱う |

## この章の考え方

- ジェネリクスは、型だけが異なる重複を型パラメータへ置き換える仕組み
- 境界のない `impl<T>` ではどの `T` にも使える操作だけを書き、必要な操作だけを境界付きの関数や `impl` に置く
- トレイトは複数の型が共有する振る舞いの契約であり、デフォルトメソッドから必須メソッドを呼び出せる
- 引数位置の `impl Trait` は単純な境界を短く表し、名前付きの `T` は複数の引数や戻り値を同じ具体型へ結び付ける
- 失敗しない自然な変換には `From`、検証が必要な変換には `TryFrom` を実装する
- `From` と `TryFrom` を実装すると、標準ライブラリの blanket implementation により対応する `Into` と `TryInto` も利用できる
- `Dispatcher<P>` の `P` はコンパイル時に具体型へ決まり、モノモーフィゼーションによる静的ディスパッチになる

この章では基礎を固めるため、明示的なライフタイム、関連型、`dyn Trait` は扱いません。これらは後の章で取り上げます

## 進め方

プロジェクトのルートで、最初の問題を実行します

```console
cargo test --example generics_traits_01_generic_cargo
```

未完成の問題は `not yet implemented` で失敗します。問題ファイルの仕様とテストを読み、`todo!()` を置き換えてください

```console
cargo test --example generics_traits_02_custom_traits
cargo test --example generics_traits_03_trait_bounds
cargo test --example generics_traits_04_conversions
cargo test --example generics_traits_05_dispatch_policy
```

問題は Cargo の example target として分離されています。`cargo test` だけでは問題を実行しないため、必ず `--example generics_traits_...` を指定してください

## 解答例

同名の解答例が `generics-traits/solutions` にあります。たとえば問題 04 の解答だけを検証するには、次を実行します

```console
cargo test --test solution_generics_traits_04_conversions
```

全コースの解答例は、次のコマンドでまとめて検証できます

```console
cargo test --tests
```

`cargo test --all-targets` は未完成の問題も実行するため、問題を解き終えるまでは失敗します

## 公式リファレンス

- [ジェネリックなデータ型 — The Rust Programming Language](https://doc.rust-lang.org/book/ch10-01-syntax.html)
- [トレイトで共通の振る舞いを定義する — The Rust Programming Language](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [トレイト境界 — Rust Reference](https://doc.rust-lang.org/reference/trait-bounds.html)
- [`impl Trait` — Rust Reference](https://doc.rust-lang.org/reference/types/impl-trait.html)
- [型変換トレイト — Rust 標準ライブラリ](https://doc.rust-lang.org/std/convert/)
- [`From`](https://doc.rust-lang.org/std/convert/trait.From.html)
- [`TryFrom`](https://doc.rust-lang.org/std/convert/trait.TryFrom.html)
