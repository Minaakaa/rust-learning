# Chapter 13 — マクロシステムとメタプログラミング

このコースでは、Rust の構文を入力として別のRustコードを生成する `macro_rules!` を学びます
関数との違い、fragment specifier、反復、TT muncher、hygiene、`$crate`、macro-generated impl を小さな配送システムへ組み込みます

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | 式マクロ | `expr` と反復を使い、型を保った式や値の構築器を作る |
| 02 | 項目生成 | `ident`、`ty`、`path` を受け取り、enum とメソッドをまとめて生成する |
| 03 | TT muncher | `tt` を少しずつ消費する再帰マクロで小さなDSLを解析する |
| 04 | hygiene と `$crate` | マクロ内部の名前衝突を避け、定義元のhelperへ安定して参照する |
| 05 | macro-generated impl | 複数の型へ同じtrait実装を生成し、重複とcoherenceを管理する |

## この章の考え方

- macro invocation はコンパイル時に展開され、式、文、型、pattern、itemなどの位置へコードを置き換える
- 関数は値を受け取って実行するが、macro_rules! はtoken treeの形にmatchして構文を生成する
- `$expr`、`$ident`、`$ty`、`$path`、`$tt` などfragment specifierは、受け取れる構文の範囲を決める
- `$(...),*`、`$(...),+`、`$(...)?` の反復は、入力の個数とseparatorを保ったまま転記する
- 反復の中のmetavariableはmatcherとtranscriberで同じ深さ・順序で使う必要がある
- TT muncherは先頭のtoken列を1件ずつ処理し、残りを再帰呼び出しへ渡してDSLを解析する
- `macro_rules!`のhygieneはmixed-siteで、local variableやlabelは定義位置、それ以外の名前は呼び出し位置のscopeに影響される
- マクロ内部で生成したlocal variableは呼び出し側の同名variableと衝突しにくいが、入力tokenとして渡したidentifierは呼び出し側の名前になる
- `$crate`はmacro定義元crateを指す特別なfragmentで、外部crateから呼ばれた場合もhelperのpathを安定させる
- macro-generated implは便利だが、生成されるtraitと型の組み合わせが重複しないようcoherenceを設計する
- macroは型検査とborrow checkの前に構文を生成するため、展開後のコードを読める命名と小さな規則に保つ
- `cargo expand`は展開結果の確認に便利だが、この章の検証はstable compilerと通常の`cargo test`だけで完結する

## procedural macroとの境界

procedural macroは`TokenStream`を受け取り`TokenStream`を返す関数として、function-like、custom derive、attribute macroの3種類を提供します
ただし定義にはcrate type `proc-macro`の別crateが必要で、通常のcrate自身から同じmacroを定義して直ちに使うことはできません
またprocedural macroはunhygienicなので、絶対pathや衝突しにくい生成名が必要です
この章では依存を増やさず`macro_rules!`の展開規則とhygieneを先に固め、proc-macro crateの構成・`TokenStream`解析・custom deriveの実装は発展課題に分離します

## 進め方

```console
cargo test --example macro_systems_01_expression_macros
cargo test --example macro_systems_02_item_generation
cargo test --example macro_systems_03_tt_muncher
cargo test --example macro_systems_04_hygiene
cargo test --example macro_systems_05_generated_impls
```

問題はCargoのexample targetとして分離されています
`cargo test`だけでは問題を実行しないため、必ず`--example macro_systems_...`を指定してください

同名の解答例が`macro-systems/solutions`にあります

```console
cargo test --test solution_macro_systems_05_generated_impls
cargo test --tests
```

## 公式リファレンス

- [Macros — The Rust Programming Language](https://doc.rust-lang.org/book/ch19-06-macros.html)
- [Macros — The Rust Reference](https://doc.rust-lang.org/reference/macros.html)
- [Macros by example — The Rust Reference](https://doc.rust-lang.org/reference/macros-by-example.html)
- [Repetitions — The Rust Reference](https://doc.rust-lang.org/reference/macros-by-example.html#repetitions)
- [Hygiene — The Rust Reference](https://doc.rust-lang.org/reference/macros-by-example.html#hygiene)
- [Procedural macros — The Rust Reference](https://doc.rust-lang.org/reference/procedural-macros.html)
