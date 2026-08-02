#![cfg_attr(not(test), allow(dead_code))]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

//! # 問題 02: UnsafeCell から小さな内部可変性APIを設計する
//!
//! `LocalSlot<T>` は、1つの値を出し入れできるsingle-thread向けslotです
//! 複数の共有参照 `&LocalSlot<T>` から値を交換できますが、共有参照を受け取る
//! メソッドから内部への参照は返しません
//!
//! `UnsafeCell` が緩和するのは共有参照 `&T` の不変性だけです
//! `&mut T` の一意性やdata raceの禁止は緩和しません
//! `UnsafeCell` は `Sync` ではないため、この型に `unsafe impl Sync` を追加してはいけません
//! thread間で共有したい場合は `Mutex` などの同期primitiveを使います
//!
//! 仕様:
//! - `new` は値が入ったslotを、`empty` は空のslotを作る
//! - `is_empty(&self)` は現在の状態だけを返す
//! - `replace(&self, value)` は新しい値を入れ、以前の値を所有権ごと返す
//! - `take(&self)` は現在の値を所有権ごと取り出して空にする
//! - `get_mut(&mut self)` は一意な参照がある場合だけ内部への可変参照を返す
//! - `into_inner(self)` はslotを消費して内部の `Option<T>` を返す
//! - `&self` を受け取るメソッドは内部への参照を返さず、unsafe区間でcallbackやdropを実行しない
//! - 各unsafe blockへ、その時点でaliasing規則が成立する根拠を書く
//!
//! TODO:
//! - 7つのメソッドを実装する
//! - unsafeが必要なメソッドと、安全な `UnsafeCell` APIだけで書けるメソッドを区別する

use std::cell::UnsafeCell;

#[allow(
    dead_code,
    reason = "TODO完成前は操作methodがUnsafeCellのvalueを読み取らないため"
)]
struct LocalSlot<T> {
    value: UnsafeCell<Option<T>>,
}

impl<T> LocalSlot<T> {
    const fn new(value: T) -> Self {
        let _ = value;
        panic!("値が入ったUnsafeCell<Option<T>>を作ってください")
    }

    const fn empty() -> Self {
        panic!("空のUnsafeCell<Option<T>>を作ってください")
    }

    fn is_empty(&self) -> bool {
        todo!("一時的な共有参照でOptionの状態を確認してください")
    }

    fn replace(&self, value: T) -> Option<T> {
        let _ = value;
        todo!("一時的な可変参照で値を交換し、以前の値を返してください")
    }

    fn take(&self) -> Option<T> {
        todo!("一時的な可変参照で値を取り出してください")
    }

    fn get_mut(&mut self) -> Option<&mut T> {
        todo!("一意なselfの借用を使い、安全なAPIだけで可変参照を返してください")
    }

    fn into_inner(self) -> Option<T> {
        todo!("selfを消費し、安全なAPIだけでOption<T>を返してください")
    }
}

fn main() {
    let slot = LocalSlot::new(String::from("配送待ち📦"));
    let old = slot.replace(String::from("点検待ち🔧"));

    println!("交換前: {old:?}");
    println!("交換後: {:?}", slot.take());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        thread,
    };

    #[derive(Debug, PartialEq, Eq)]
    struct Telemetry {
        robot_id: String,
        millivolts: u32,
    }

    impl Telemetry {
        fn new(robot_id: &str, millivolts: u32) -> Self {
            Self {
                robot_id: robot_id.to_owned(),
                millivolts,
            }
        }
    }

    struct DropProbe {
        drops: Arc<AtomicUsize>,
    }

    impl DropProbe {
        fn new(drops: &Arc<AtomicUsize>) -> Self {
            Self {
                drops: Arc::clone(drops),
            }
        }
    }

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn aliased_shared_refsからnon_clone値を交換して取り出す() {
        let slot = LocalSlot::new(Telemetry::new("配送ロボット🤖-初号", 3_300));
        let first_view = &slot;
        let second_view = &slot;
        let old = first_view
            .replace(Telemetry::new("点検ロボット-二号", 3_150))
            .expect("初期値がある");
        let current = second_view.take().expect("交換後の値がある");

        assert_eq!(old, Telemetry::new("配送ロボット🤖-初号", 3_300));
        assert_eq!(current, Telemetry::new("点検ロボット-二号", 3_150));
        assert!(slot.is_empty());
    }

    #[test]
    fn emptyから値ありと空へ遷移する() {
        let slot = LocalSlot::<String>::empty();

        assert!(slot.is_empty());
        assert_eq!(slot.take(), None);
        assert_eq!(slot.replace(String::from("研究棟A🧪")), None);
        assert!(!slot.is_empty());
        assert_eq!(slot.take().as_deref(), Some("研究棟A🧪"));
        assert!(slot.is_empty());
    }

    #[test]
    fn unique_refがあるときだけ内部への可変refを返す() {
        let mut slot = LocalSlot::new(Telemetry::new("観測機-甲", 2_800));

        let reading = slot.get_mut().expect("値がある");
        reading.robot_id.push('🌊');
        reading.millivolts += 25;

        assert_eq!(
            slot.into_inner(),
            Some(Telemetry::new("観測機-甲🌊", 2_825))
        );
    }

    #[test]
    fn 空slotのget_mutはnoneを返す() {
        let mut slot = LocalSlot::<Telemetry>::empty();

        assert_eq!(slot.get_mut(), None);
    }

    #[test]
    fn replaceは新旧の値をそれぞれ一度だけdropする() {
        let old_drops = Arc::new(AtomicUsize::new(0));
        let new_drops = Arc::new(AtomicUsize::new(0));
        let slot = LocalSlot::new(DropProbe::new(&old_drops));

        let old = slot
            .replace(DropProbe::new(&new_drops))
            .expect("初期値がある");

        assert_eq!(old_drops.load(Ordering::SeqCst), 0);
        assert_eq!(new_drops.load(Ordering::SeqCst), 0);
        drop(old);
        assert_eq!(old_drops.load(Ordering::SeqCst), 1);
        drop(slot);
        assert_eq!(old_drops.load(Ordering::SeqCst), 1);
        assert_eq!(new_drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn takeした値のdrop責任は呼び出し側へ移る() {
        let drops = Arc::new(AtomicUsize::new(0));
        let slot = LocalSlot::new(DropProbe::new(&drops));

        let taken = slot.take().expect("値がある");
        drop(slot);
        assert_eq!(drops.load(Ordering::SeqCst), 0);

        drop(taken);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn into_innerは値をdropせず所有権を返す() {
        let drops = Arc::new(AtomicUsize::new(0));
        let slot = LocalSlot::new(DropProbe::new(&drops));

        let inner = slot.into_inner().expect("値がある");
        assert_eq!(drops.load(Ordering::SeqCst), 0);

        drop(inner);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn local_slotは値がsendなら別threadへ所有権を移せる() {
        fn assert_send<T: Send>() {}
        assert_send::<LocalSlot<String>>();

        let slot = LocalSlot::new(String::from("遠隔観測局📡"));
        let value = thread::spawn(move || slot.into_inner())
            .join()
            .expect("worker threadが完了する");

        assert_eq!(value.as_deref(), Some("遠隔観測局📡"));
    }
}
