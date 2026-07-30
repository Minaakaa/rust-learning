# Chapter 8 — スマートポインタで所有権の形を広げる

このコースでは、通常の参照や単一所有だけでは表しにくいデータ構造と共有関係を設計します
`Box<T>` の再帰型から始め、`Deref`・`Drop` によるポインタらしい操作と自動解放、`Rc<T>` の共有所有、`RefCell<T>` の内部可変性へ進み、最後に `Weak<T>` で循環しないロボット群を完成させます

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | `Box<T>` | 再帰型を有限サイズにし、所有値を複製せず経路へつなぐ |
| 02 | `Deref` と `Drop` | ガードを参照のように操作し、RAII でドックを確実に解放する |
| 03 | `Rc<T>` | 同じキャンパスマップを複製せず、複数の所有者で共有する |
| 04 | `RefCell<T>` | 共有参照から状態を更新し、借用競合を panic ではなくエラーにする |
| 05 | `Weak<T>` | 強い所有と非所有の逆参照を組み合わせ、参照循環を防ぐ |

## この章の考え方

- 通常の参照は値を借用し、スマートポインタは多くの場合、指している値を所有する
- `Box<T>` は値をヒープへ置き、現在の型には固定サイズのポインタだけを保持する
- 再帰型の再帰部分を `Box<T>` で間接化すると、コンパイラが型のサイズを決定できる
- `Deref` の関連型 `Target` と `deref` を実装すると、参照を受け取る API へ deref coercion で渡せる
- `Drop` は値がスコープを抜けると自動実行され、早く破棄したい場合は `drop(value)` を使う
- `Rc<T>` は単一スレッド内で値に複数の所有者を持たせ、最後の強参照がなくなると値を破棄する
- `Rc::clone` は内部の `T` を複製せず、同じ allocation の strong count を増やす
- `RefCell<T>` は借用規則を実行時に検査し、`Ref` と `RefMut` がガードの生存中だけ借用を維持する
- `borrow_mut` は競合時に panic するため、回復可能な API では `try_borrow_mut` でエラーへ変換できる
- `Weak<T>` は値を所有せず、`upgrade` は対象が生存している場合だけ `Some(Rc<T>)` を返す
- 強参照の戻り道を `Weak<T>` にすると、`Rc<T>` 同士の循環によるメモリ保持を避けられる

この章では単一スレッド向けの所有モデルに集中し、`Arc<T>`、`Mutex<T>`、raw pointer、`unsafe`、`Pin`、独自 allocator は扱いません

## 進め方

プロジェクトのルートで、最初の問題を実行します

```console
cargo test --example smart_pointers_01_boxed_routes
```

未完成の問題は `not yet implemented` または未実装部分に対応するテスト失敗になります
問題ファイルの仕様、ヒント、テストを読み、`todo!()`、TODO コメント、必要な型を変更してください

```console
cargo test --example smart_pointers_02_deref_drop_guards
cargo test --example smart_pointers_03_shared_maps
cargo test --example smart_pointers_04_refcell_console
cargo test --example smart_pointers_05_weak_fleet
```

問題は Cargo の example target として分離されています
`cargo test` だけでは問題を実行しないため、必ず `--example smart_pointers_...` を指定してください

## 解答例

同名の解答例が `smart-pointers/solutions` にあります
たとえば問題 04 の解答だけを検証するには、次を実行します

```console
cargo test --test solution_smart_pointers_04_refcell_console
```

全コースの解答例は、次のコマンドでまとめて検証できます

```console
cargo test --tests
```

`cargo test --all-targets` は未完成の問題も実行するため、問題を解き終えるまでは失敗します

## 公式リファレンス

- [スマートポインタ — The Rust Programming Language](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- [`Box<T>` でヒープ上のデータを指す](https://doc.rust-lang.org/book/ch15-01-box.html)
- [`Deref` で通常の参照のように扱う](https://doc.rust-lang.org/book/ch15-02-deref.html)
- [`Drop` で後片付けを実行する](https://doc.rust-lang.org/book/ch15-03-drop.html)
- [`Rc<T>` による参照カウント](https://doc.rust-lang.org/book/ch15-04-rc.html)
- [`RefCell<T>` と内部可変性](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)
- [参照循環と `Weak<T>`](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)
- [`Rc` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/rc/struct.Rc.html)
- [`RefCell` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/cell/struct.RefCell.html)
- [`Weak` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/rc/struct.Weak.html)
