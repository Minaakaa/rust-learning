# Chapter 4 — コレクションで倉庫とログを整理する

このコースでは、配送ロボットの荷物棚、配送待ちキュー、部品在庫、能力一覧、運行ログに合ったコレクションを選びます。値を保存できれば終わりではなく、順序、検索方法、重複、所有権、出力の決定性を考えてデータ構造を使い分けることが目標です。

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | `Vec<T>` | 順序付きの荷物を追加・参照・削除し、所有権を安全に受け渡す |
| 02 | `VecDeque<T>` | 前後への追加と削除を使い、配送待ちキューを実装する |
| 03 | `HashMap<K, V>` | キーで在庫を検索し、`entry` API で数量を安全に更新する |
| 04 | `HashSet<T>` | 重複のない能力一覧を作り、和・積・差を計算する |
| 05 | UTF-8 `String` | バイト境界を壊さず日本語ログを短縮し、複数のコレクションで集計する |

## この章の考え方

- 順序付きデータにはまず `Vec` を検討し、先頭の追加・削除が多いキューには `VecDeque` を使う。
- 名前から値を検索するなら `HashMap`、重複を除いて所属だけを調べるなら `HashSet` を使う。
- `HashMap` と `HashSet` の反復順序は決まっていない。順序のあるレポートが必要なら、要素を集めて明示的に並べ替える。
- `String` と `&str` は UTF-8 である。`len()` はバイト数を返し、`chars()` は Unicode スカラー値を列挙する。
- Unicode スカラー値と、人が 1 文字と感じる書記素クラスタは同じとは限らない。この章では書記素クラスタの分割は扱わない。
- コレクションへ所有値を入れると所有権はコレクションへ移る。読むだけならスライスや共有参照を使い、独立した記録が必要な場合だけ複製する。

## 進め方

プロジェクトのルートで、最初の問題を実行します。

```console
cargo test --example collections_01_vec
```

未完成の問題は `not yet implemented` で失敗します。問題ファイルの仕様とテストを読み、`todo!()` を置き換えてください。

```console
cargo test --example collections_02_vec_deque
cargo test --example collections_03_hash_map
cargo test --example collections_04_hash_set
cargo test --example collections_05_utf8_logs
```

問題は Cargo の example target として分離されています。`cargo test` だけでは問題を実行しないため、必ず `--example collections_...` を指定してください。

## 解答例

同名の解答例が `collections/solutions` にあります。たとえば問題 03 の解答だけを検証するには、次を実行します。

```console
cargo test --test solution_collections_03_hash_map
```

全コースの解答例は、次のコマンドでまとめて検証できます。

```console
cargo test --tests
```

`cargo test --all-targets` は未完成の問題も実行するため、問題を解き終えるまでは失敗します。

## 公式リファレンス

- [コレクション — Rust 標準ライブラリ](https://doc.rust-lang.org/std/collections/)
- [`Vec<T>`](https://doc.rust-lang.org/std/vec/struct.Vec.html)
- [`VecDeque<T>`](https://doc.rust-lang.org/std/collections/struct.VecDeque.html)
- [`HashMap<K, V>` と `Entry`](https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html)
- [`HashSet<T>`](https://doc.rust-lang.org/std/collections/struct.HashSet.html)
- [UTF-8 文字列を格納する — The Rust Programming Language](https://doc.rust-lang.org/book/ch08-02-strings.html)
- [`String`](https://doc.rust-lang.org/std/string/struct.String.html)
