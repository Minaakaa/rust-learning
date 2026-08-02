#![cfg_attr(not(test), allow(dead_code))]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

//! # 解答 02: UnsafeCell から小さな内部可変性APIを設計する
//!
//! `UnsafeCell` が緩和するのは共有参照の不変性だけです
//! 可変参照の一意性とdata race禁止は維持されるため、`LocalSlot` は `Sync` にしません

use std::cell::UnsafeCell;

struct LocalSlot<T> {
    value: UnsafeCell<Option<T>>,
}

impl<T> LocalSlot<T> {
    const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(Some(value)),
        }
    }

    const fn empty() -> Self {
        Self {
            value: UnsafeCell::new(None),
        }
    }

    fn is_empty(&self) -> bool {
        // SAFETY: 共有参照はこの式の間だけ存在し、変更もuser codeの呼び出しも行わない
        // &selfを受け取るAPIは内部参照を返さず、UnsafeCellによりLocalSlotはSyncではない
        unsafe { (&*self.value.get()).is_none() }
    }

    fn replace(&self, value: T) -> Option<T> {
        // SAFETY: 一時的な可変参照はこの呼び出しから外へ出ず、Option::replaceは
        // user codeやdropを実行しないため、別の内部参照が途中で作られることはない
        // UnsafeCellによりLocalSlotはSyncではなく、別threadとのdata raceも起こせない
        unsafe { (&mut *self.value.get()).replace(value) }
    }

    fn take(&self) -> Option<T> {
        // SAFETY: 一時的な可変参照はこの呼び出しから外へ出ず、Option::takeは
        // user codeやdropを実行しないため、別の内部参照が途中で作られることはない
        // UnsafeCellによりLocalSlotはSyncではなく、別threadとのdata raceも起こせない
        unsafe { (&mut *self.value.get()).take() }
    }

    fn get_mut(&mut self) -> Option<&mut T> {
        self.value.get_mut().as_mut()
    }

    fn into_inner(self) -> Option<T> {
        self.value.into_inner()
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
