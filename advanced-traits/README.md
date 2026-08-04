# Chapter 12 — 高度なトレイト設計と動的多相

このコースでは、実装ごとに変わる型を関連型で契約へ組み込み、実行時に異なる具体型を同じコレクションへ格納する方法を学びます
関連型、blanket implementation、coherence、newtype から始め、dyn 互換な interface、trait object、clone 可能な型消去、`Any` による限定的な型復元へ進みます
最後に GAT、HRTB、RPITIT を使い、借用に応じて変わる型と opaque な戻り値を設計します

trait upcasting は Rust 1.86 で安定化されています
このリポジトリは他章で使う API も含め、Rust 1.87 以上を必要とします

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | 関連型 | 実装ごとの `Reading` と `Error` を trait の契約へ結び付け、projection と等値制約を使う |
| 02 | blanket impl と coherence | extension trait をまとめて実装し、orphan rule を newtype で解決する |
| 03 | dyn dispatch | dyn 互換な trait を設計し、関連型を固定した異種実装を1つの engine で実行する |
| 04 | clone 可能な trait object | `clone_box` で型消去した所有値を具体型の `Clone` 意味論に従って複製する |
| 05 | `Any` と typed registry | `TypeId` で型ごとに登録し、必要な管理操作だけで安全に downcast する |
| 06 | GAT と lending view | 借用の lifetime ごとに変わる view 型を関連型の型族として表す |
| 07 | HRTB | callback が任意の短い lifetime の借用を受け取れることを `for<'a>` で表す |
| 08 | RPITIT | trait 実装ごとに異なる iterator 型を隠し、static dispatch と dyn 境界を設計する |

## この章の考え方

- 関連型は implementor が具体型を1つ選ぶ trait の契約で、`S::Reading` や `<S as Sensor>::Reading` という projection で参照する
- trait の型引数は同じ型へ複数の組み合わせを実装できる一方、関連型は1つの trait 実装に1つの対応を与える
- `dyn Trait` で通常の関連型を使う時は、`dyn Trait<Output = Decision>` のように具体型を固定する
- blanket implementation は境界を満たすすべての型へ適用され、後から重なる個別 impl を追加できないため公開 API の設計判断になる
- coherence は同じ trait と型の組み合わせに実装を1つだけ定め、overlap と orphan implementation を禁止する
- 外部 trait を外部型へ直接実装できない場合は、local な newtype で包むと orphan rule を守りながら実装できる
- 静的 dispatch では具体型ごとに monomorphize でき、動的 dispatch では実行時に vtable から method を選べる
- trait object は dynamically sized type なので、`&dyn Trait`、`Box<dyn Trait>`、`Arc<dyn Trait>` など pointer 越しに扱う
- trait object は具体値への data pointer と method 解決用 vtable を使うが、vtable の配置は Rust の安定した ABI として公開されていない
- dyn 互換な trait は `Self: Sized` を supertrait にせず、dispatch 対象 method に型 parameter、receiver 以外の `Self`、opaque return type を持たせない
- generic method や constructor など具体型が必要な method は `where Self: Sized` を付けると、trait object から呼べない method として分離できる
- supertrait は複数の能力を1つの base trait へまとめ、trait upcasting で `dyn SubTrait` を `dyn SuperTrait` へ coerce できる
- generic API や blanket impl へ `?Sized` を付けると、暗黙の `Sized` 制約を緩めて `dyn Trait` も受け取れる
- trait object は heap allocation を必須とせず、既存値を `&dyn Trait` として借用するだけでも動的 dispatch を利用できる
- `Box<dyn Trait>` の object lifetime は型位置では通常 `'static` になり、借用を保持する場合は `Box<dyn Trait + 'a>` のように明示する
- trait object へ追加できる非 auto trait は base trait 1つで、`Send`、`Sync` などの auto trait は追加境界として組み合わせられる
- `Clone` は `Sized` を要求して `Self` を返すため dyn 互換ではなく、object-safe な `clone_box` を別の supertrait として用意する
- `clone_box` も各具体型の `Clone` 実装に従うため、`String` は独立して複製され、`Arc` は同じ値の共有を維持する
- `Any` は `'static` な具体型の identity を提供し、`downcast_ref`、`downcast_mut`、owned downcast で exact type だけを復元する
- `Box<dyn Any>` 自体へ `.type_id()` を呼ぶと container の型を調べるため、registry の key には `TypeId::of::<E>()` を使う
- `Any` は「別の trait を実装しているか」を判定できないため、通常の振る舞いは trait method で表し、downcast は型固有の管理操作へ限定する
- generic associated type（GAT）は `type View<'a>` のようにparameterを持つ関連型で、1つの型ではなくlifetimeごとの型族を表す
- `fn next_view<'a>(&'a mut self) -> Self::View<'a>` のようにborrowを返す場合、GATへ `where Self: 'a` が必要になる
- higher-ranked trait bound（HRTB）の `for<'a>` は、呼び出し側が選ぶ1つのlifetimeではなく、すべてのlifetimeで境界を満たすことを表す
- `for<'a> F: Fn(&'a str) -> &'a str` と `F: for<'a> Fn(&'a str) -> &'a str` は量化する範囲の書き方が異なり、どちらも任意の短いborrowで `F` を呼べる
- `dyn for<'a> Fn(&'a str) -> &'a str` のように、HRTBを持つcallbackもtrait objectとして型消去できる
- return-position `impl Trait` in trait（RPITIT）はanonymous associated typeへdesugarされ、各implが公開boundを満たすhidden concrete typeを選ぶ
- RPITITのcallerが利用できるのは宣言されたboundだけで、実装が返す具体型や追加の能力には依存できない
- 1つのRPITIT methodの各return pathは同じconcrete typeへ解決され、戻り値はBox化せずstatic dispatchできる
- opaque return typeを持つmethodはvtable dispatchできないが、`where Self: Sized`でstatic-onlyに分離すればtrait全体のdyn互換性を保てる

## 発展トピックとの境界

- genericな関連型を持つtraitはdyn互換ではないため、問題06はstatic dispatchで利用する
- trait内 `async fn` はRPITITと同じくanonymous associated typeを使うが、async traitの型消去は扱わない
- 動的境界でopaque return typeが必要なら、`where Self: Sized`で分離するか別interfaceで型消去する設計を選ぶ
- trait alias、specialization、一般の negative impl、`Unsize` / `CoerceUnsized` の直接実装は unstable なので扱わない

この章では derive macro の作成、async trait の型消去、独自 smart pointer の unsizing、vtable ABI の解析、dispatch 性能 benchmark は扱いません

## 進め方

プロジェクトのルートで、最初の問題を実行します

```console
cargo test --example advanced_traits_01_associated_types
```

未完成の問題は `not yet implemented` または未実装部分に対応するテスト失敗になります
問題ファイルの仕様、型境界、テストを読み、`todo!()` と必要な処理を変更してください

```console
cargo test --example advanced_traits_02_blanket_impls
cargo test --example advanced_traits_03_dyn_dispatch
cargo test --example advanced_traits_04_cloneable_trait_objects
cargo test --example advanced_traits_05_any_registry
cargo test --example advanced_traits_06_gat_lending_views
cargo test --example advanced_traits_07_hrtb_callbacks
cargo test --example advanced_traits_08_rpitit_iterators
```

問題は Cargo の example target として分離されています
`cargo test` だけでは問題を実行しないため、必ず `--example advanced_traits_...` を指定してください

## 解答例

同名の解答例が `advanced-traits/solutions` にあります
たとえば問題08の解答だけを検証するには、次を実行します

```console
cargo test --test solution_advanced_traits_08_rpitit_iterators
```

全コースの解答例は、次のコマンドでまとめて検証できます

```console
cargo test --tests
```

`cargo test --all-targets` は未完成の問題も実行するため、問題を解き終えるまでは失敗します

## 公式リファレンス

- [高度なトレイト — The Rust Programming Language](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html)
- [trait objectによる共通動作 — The Rust Programming Language](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
- [関連 item と GAT — The Rust Reference](https://doc.rust-lang.org/reference/items/associated-items.html)
- [trait implementation coherence と orphan rule — The Rust Reference](https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence)
- [dyn 互換性 — The Rust Reference](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility)
- [trait object の型と lifetime — The Rust Reference](https://doc.rust-lang.org/reference/types/trait-object.html)
- [`?Sized` と trait bound — The Rust Reference](https://doc.rust-lang.org/reference/trait-bounds.html#sized)
- [higher-ranked trait bound — The Rust Reference](https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds)
- [trait upcasting coercion — The Rust Reference](https://doc.rust-lang.org/reference/type-coercions.html#unsized-coercions)
- [`impl Trait`とRPITIT — The Rust Reference](https://doc.rust-lang.org/reference/types/impl-trait.html)
- [`Clone` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/clone/trait.Clone.html)
- [`Any` と downcast — Rust 標準ライブラリ](https://doc.rust-lang.org/std/any/)
- [`async fn`とRPITITのstable化 — Rust Blog](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html)
- [trait upcasting の stable 化 — Rust 1.86.0](https://blog.rust-lang.org/2025/04/03/Rust-1.86.0.html#trait-upcasting)
