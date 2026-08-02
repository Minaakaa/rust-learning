#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

//! # 解答 04: `MaybeUninit`で固定長bufferを作る
//!
//! `slots[..len]`だけが初期化済みという不変条件を、すべてのsafe methodで維持します

use std::mem::MaybeUninit;
use std::slice;

struct FixedBuffer<T, const N: usize> {
    slots: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> FixedBuffer<T, N> {
    fn new() -> Self {
        Self {
            slots: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    const fn len(&self) -> usize {
        self.len
    }

    const fn is_empty(&self) -> bool {
        self.len == 0
    }

    const fn is_full(&self) -> bool {
        self.len == N
    }

    fn push(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            return Err(value);
        }

        self.slots[self.len].write(value);
        self.len += 1;
        Ok(())
    }

    fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }

        // SAFETY: 不変条件によりindex < lenのslotは初期化済みで、この共有参照中はbufferを変更できない
        Some(unsafe { self.slots[index].assume_init_ref() })
    }

    fn as_slice(&self) -> &[T] {
        // SAFETY: 先頭len個はすべて初期化済みで、MaybeUninit<T>はTと同じsizeとalignmentを持つ
        // selfの共有borrowと同じ期間だけ参照を返すため、sliceの生存中にslotは変更されない
        unsafe { slice::from_raw_parts(self.slots.as_ptr().cast::<T>(), self.len) }
    }

    fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.len -= 1;
        // SAFETY: 減算前の不変条件により現在のlen番slotは初期化済み
        // lenを先に減らしたため、読み出した所有値をbufferが再度dropすることはない
        Some(unsafe { self.slots[self.len].assume_init_read() })
    }

    fn clear(&mut self) {
        while !self.is_empty() {
            self.len -= 1;
            // SAFETY: 減算前の不変条件により現在のlen番slotは初期化済み
            // drop前にlenから除外したため、destructorがpanicしても同じslotを再度dropしない
            unsafe { self.slots[self.len].assume_init_drop() };
        }
    }
}

impl<T, const N: usize> Drop for FixedBuffer<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

fn main() {
    let mut messages = FixedBuffer::<String, 3>::new();
    messages.push(String::from("温度=24℃")).expect("空きがある");
    messages
        .push(String::from("状態=正常🤖"))
        .expect("空きがある");

    println!("保存件数={}", messages.len());
    for message in messages.as_slice() {
        println!("{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::rc::Rc;

    #[derive(Debug)]
    struct DropProbe {
        id: usize,
        dropped: Rc<RefCell<Vec<usize>>>,
    }

    impl DropProbe {
        fn new(id: usize, dropped: &Rc<RefCell<Vec<usize>>>) -> Self {
            Self {
                id,
                dropped: Rc::clone(dropped),
            }
        }
    }

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.dropped.borrow_mut().push(self.id);
        }
    }

    #[test]
    fn 新しいbufferは空で満杯ではない() {
        let buffer = FixedBuffer::<String, 3>::new();

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
        assert!(buffer.as_slice().is_empty());
        assert!(buffer.get(0).is_none());
        assert!(buffer.get(usize::MAX).is_none());
    }

    #[test]
    fn pushした順でgetとsliceから参照できる() {
        let mut buffer = FixedBuffer::<String, 3>::new();
        buffer.push(String::from("一号")).unwrap();
        buffer.push(String::from("二号")).unwrap();

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.get(0).map(String::as_str), Some("一号"));
        assert_eq!(buffer.get(1).map(String::as_str), Some("二号"));
        assert!(buffer.get(2).is_none());
        let values = buffer
            .as_slice()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(values, ["一号", "二号"]);
    }

    #[test]
    fn 満杯なら入力値とallocationをそのまま返す() {
        let mut buffer = FixedBuffer::<String, 1>::new();
        buffer.push(String::from("格納済み")).unwrap();
        let pending = String::from("返却対象📦");
        let pending_pointer = pending.as_ptr();

        let returned = buffer.push(pending).expect_err("満杯なので値を返す");

        assert_eq!(returned, "返却対象📦");
        assert_eq!(returned.as_ptr(), pending_pointer);
        assert_eq!(buffer.len(), 1);
        assert!(buffer.is_full());
        assert_eq!(buffer.get(0).map(String::as_str), Some("格納済み"));
    }

    #[test]
    fn popは逆順で所有値とallocationを返す() {
        let mut buffer = FixedBuffer::<String, 2>::new();
        let first = String::from("first");
        let second = String::from("second");
        let first_pointer = first.as_ptr();
        let second_pointer = second.as_ptr();
        buffer.push(first).unwrap();
        buffer.push(second).unwrap();

        let returned_second = buffer.pop().expect("末尾を取り出せる");
        assert_eq!(returned_second, "second");
        assert_eq!(returned_second.as_ptr(), second_pointer);
        let returned_first = buffer.pop().expect("先頭を取り出せる");
        assert_eq!(returned_first, "first");
        assert_eq!(returned_first.as_ptr(), first_pointer);
        assert!(buffer.pop().is_none());
        assert!(buffer.is_empty());
    }

    #[test]
    fn clearは全要素を一度だけdropしbufferを再利用できる() {
        let dropped = Rc::new(RefCell::new(Vec::new()));
        let mut buffer = FixedBuffer::<DropProbe, 3>::new();
        for id in 0..3 {
            buffer.push(DropProbe::new(id, &dropped)).unwrap();
        }

        buffer.clear();

        assert!(buffer.is_empty());
        assert_eq!(&*dropped.borrow(), &[2, 1, 0]);
        buffer.push(DropProbe::new(3, &dropped)).unwrap();
        drop(buffer);
        assert_eq!(&*dropped.borrow(), &[2, 1, 0, 3]);
    }

    #[test]
    fn bufferのdropは残った要素を各一度だけdropする() {
        let dropped = Rc::new(RefCell::new(Vec::new()));

        {
            let mut buffer = FixedBuffer::<DropProbe, 4>::new();
            for id in 0..4 {
                buffer.push(DropProbe::new(id, &dropped)).unwrap();
            }
            let removed = buffer.pop().unwrap();
            drop(removed);
            assert_eq!(&*dropped.borrow(), &[3]);
        }

        assert_eq!(&*dropped.borrow(), &[3, 2, 1, 0]);
    }

    #[test]
    fn capacity_zeroは常に満杯で値を変更せず返す() {
        let mut buffer = FixedBuffer::<String, 0>::new();
        let pending = String::from("保存できない");
        let pending_pointer = pending.as_ptr();

        let returned = buffer.push(pending).expect_err("容量0では保存できない");

        assert_eq!(returned.as_ptr(), pending_pointer);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert!(buffer.is_full());
        assert!(buffer.as_slice().is_empty());
        assert!(buffer.pop().is_none());
        buffer.clear();
    }

    #[test]
    fn zero_sized_typeもcapacityまで保持できる() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static DROPS: AtomicUsize = AtomicUsize::new(0);

        #[derive(Debug)]
        struct ZstDrop;

        impl Drop for ZstDrop {
            fn drop(&mut self) {
                DROPS.fetch_add(1, Ordering::Relaxed);
            }
        }

        DROPS.store(0, Ordering::Relaxed);
        assert_eq!(std::mem::size_of::<ZstDrop>(), 0);
        let mut buffer = FixedBuffer::<ZstDrop, 3>::new();
        assert!(buffer.push(ZstDrop).is_ok());
        assert!(buffer.push(ZstDrop).is_ok());
        assert!(buffer.push(ZstDrop).is_ok());
        let rejected = buffer.push(ZstDrop);
        assert!(rejected.is_err());
        drop(rejected);
        assert_eq!(DROPS.load(Ordering::Relaxed), 1);
        assert!(buffer.is_full());
        assert_eq!(buffer.as_slice().len(), 3);
        drop(buffer.pop());
        assert_eq!(DROPS.load(Ordering::Relaxed), 2);
        assert_eq!(buffer.len(), 2);
        drop(buffer);
        assert_eq!(DROPS.load(Ordering::Relaxed), 4);
    }

    #[test]
    fn utf8文字列をbyte境界と無関係に保持する() {
        let mut buffer = FixedBuffer::<String, 3>::new();
        buffer.push(String::from("東京大学🏫")).unwrap();
        buffer.push(String::from("配送ロボット🤖")).unwrap();
        buffer.push(String::from("温度=零下5℃")).unwrap();

        let values = buffer
            .as_slice()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(values, ["東京大学🏫", "配送ロボット🤖", "温度=零下5℃"]);
    }

    #[test]
    fn elementのdropがpanicしても同じslotを再dropしない() {
        #[derive(Debug)]
        struct PanicDropProbe {
            id: usize,
            panic_id: usize,
            dropped: Rc<RefCell<Vec<usize>>>,
        }

        impl Drop for PanicDropProbe {
            fn drop(&mut self) {
                self.dropped.borrow_mut().push(self.id);
                // test失敗のunwindとelementのpanicが重なってprocessをabortしない
                if self.id == self.panic_id && !std::thread::panicking() {
                    panic!("drop panic: {}", self.id);
                }
            }
        }

        let dropped = Rc::new(RefCell::new(Vec::new()));
        let mut buffer = FixedBuffer::<PanicDropProbe, 3>::new();
        for id in 0..3 {
            buffer
                .push(PanicDropProbe {
                    id,
                    panic_id: 1,
                    dropped: Rc::clone(&dropped),
                })
                .unwrap();
        }

        let result = catch_unwind(AssertUnwindSafe(|| buffer.clear()));

        assert!(result.is_err());
        assert_eq!(buffer.len(), 1);
        assert_eq!(&*dropped.borrow(), &[2, 1]);
        drop(buffer);
        assert_eq!(&*dropped.borrow(), &[2, 1, 0]);
    }
}
