#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

//! # 解答 05: panic-safeな配列builderを作る
//!
//! `PartialArray`をdrop guardとして使い、成功、`Err`、panicのcleanupを一元化します

use std::mem::{ManuallyDrop, MaybeUninit};

struct PartialArray<T, const N: usize> {
    values: [MaybeUninit<T>; N],
    initialized: usize,
}

impl<T, const N: usize> PartialArray<T, N> {
    fn new() -> Self {
        Self {
            values: [const { MaybeUninit::uninit() }; N],
            initialized: 0,
        }
    }

    fn push(&mut self, value: T) {
        assert!(self.initialized < N, "配列のcapacityを超えて初期化できない");
        self.values[self.initialized].write(value);
        self.initialized += 1;
    }

    fn finish(self) -> [T; N] {
        assert_eq!(self.initialized, N, "全slotの初期化が必要");
        let this = ManuallyDrop::new(self);

        // SAFETY: initialized == Nなので全slotが有効なTを1つずつ保持している
        // MaybeUninit<T>はTと同じsizeとalignmentを持ち、ManuallyDropにより元guardはdropされない
        // 配列全体を1回だけreadするため、返り値が各elementの唯一の所有者になる
        unsafe { this.values.as_ptr().cast::<[T; N]>().read() }
    }
}

impl<T, const N: usize> Drop for PartialArray<T, N> {
    fn drop(&mut self) {
        while self.initialized != 0 {
            self.initialized -= 1;
            // SAFETY: 不変条件により減算後のindexは初期化済みprefixの末尾だった
            // countを先に減らしたため、destructorがpanicしても同じelementを再度dropしない
            unsafe { self.values[self.initialized].assume_init_drop() };
        }
    }
}

fn try_build_array<T, E, F, const N: usize>(mut initializer: F) -> Result<[T; N], E>
where
    F: FnMut(usize) -> Result<T, E>,
{
    let mut partial = PartialArray::new();
    for index in 0..N {
        let value = initializer(index)?;
        partial.push(value);
    }
    Ok(partial.finish())
}

fn main() {
    let labels = try_build_array::<String, (), _, 3>(|index| Ok(format!("遠隔測定-{index}🤖")))
        .expect("全labelを作成できる");

    println!("{}", labels.join(", "));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::rc::Rc;
    use std::sync::atomic::{AtomicUsize, Ordering};

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

    #[derive(Debug, PartialEq, Eq)]
    struct BuildError {
        index: usize,
        message: String,
    }

    #[test]
    fn initializerをindex順に呼び固定長配列を返す() {
        let called = Rc::new(RefCell::new(Vec::new()));
        let called_by_initializer = Rc::clone(&called);

        let result = try_build_array::<String, (), _, 4>(|index| {
            called_by_initializer.borrow_mut().push(index);
            Ok(format!("sensor-{index}"))
        })
        .expect("全要素を初期化できる");

        assert_eq!(&*called.borrow(), &[0, 1, 2, 3]);
        assert_eq!(result, ["sensor-0", "sensor-1", "sensor-2", "sensor-3"]);
    }

    #[test]
    fn non_copy文字列のallocationとutf8を保ったまま配列へ移す() {
        let mut source = [
            Some(String::from("東京大学🏫")),
            Some(String::from("配送ロボット🤖")),
            Some(String::from("温度=24℃")),
        ];
        let pointers = source
            .each_ref()
            .map(|value| value.as_ref().expect("初期値がある").as_ptr());

        let result = try_build_array::<String, (), _, 3>(|index| {
            Ok(source[index].take().expect("各値を1回だけ取り出す"))
        })
        .expect("全要素を初期化できる");

        assert!(source.iter().all(Option::is_none));
        assert_eq!(
            result.each_ref().map(|value| value.as_str()),
            ["東京大学🏫", "配送ロボット🤖", "温度=24℃"]
        );
        assert_eq!(result.each_ref().map(|value| value.as_ptr()), pointers);
    }

    #[test]
    fn errでは初期化済みprefixを一度だけdropしerrorを変更せず返す() {
        let dropped = Rc::new(RefCell::new(Vec::new()));
        let message = String::from("sensor-3の初期化に失敗");
        let message_pointer = message.as_ptr();
        let mut message = Some(message);

        let error = try_build_array::<DropProbe, BuildError, _, 5>(|index| {
            if index == 3 {
                return Err(BuildError {
                    index,
                    message: message.take().expect("errorは1回だけ作る"),
                });
            }
            Ok(DropProbe::new(index, &dropped))
        })
        .expect_err("index 3で失敗する");

        assert_eq!(error.index, 3);
        assert_eq!(error.message, "sensor-3の初期化に失敗");
        assert_eq!(error.message.as_ptr(), message_pointer);
        assert_eq!(&*dropped.borrow(), &[2, 1, 0]);
    }

    #[test]
    fn initializerのpanicでも初期化済みprefixを一度だけdropする() {
        let dropped = Rc::new(RefCell::new(Vec::new()));
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _ = try_build_array::<DropProbe, (), _, 5>(|index| {
                if index == 3 {
                    panic!("initializer panic at {index}");
                }
                Ok(DropProbe::new(index, &dropped))
            });
        }));

        assert!(result.is_err());
        assert_eq!(&*dropped.borrow(), &[2, 1, 0]);
    }

    #[test]
    fn 成功時は配列が所有権を引き継ぎ各要素を一度だけdropする() {
        let dropped = Rc::new(RefCell::new(Vec::new()));
        let result =
            try_build_array::<DropProbe, (), _, 3>(|index| Ok(DropProbe::new(index, &dropped)))
                .expect("全要素を初期化できる");

        assert_eq!(result.each_ref().map(|probe| probe.id), [0, 1, 2]);
        assert!(dropped.borrow().is_empty());

        drop(result);
        let mut dropped_ids = dropped.borrow().clone();
        dropped_ids.sort_unstable();
        assert_eq!(dropped_ids, [0, 1, 2]);
    }

    #[test]
    fn capacity_zeroではinitializerを呼ばない() {
        let calls = AtomicUsize::new(0);

        let result = try_build_array::<String, (), _, 0>(|_| {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok(String::from("呼ばれない"))
        })
        .expect("空配列は直ちに完成する");

        assert!(result.is_empty());
        assert_eq!(calls.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn zero_sized_typeも成功後に各一度だけdropする() {
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
        let result =
            try_build_array::<ZstDrop, (), _, 4>(|_| Ok(ZstDrop)).expect("ZSTの配列を作れる");
        assert_eq!(DROPS.load(Ordering::Relaxed), 0);

        drop(result);
        assert_eq!(DROPS.load(Ordering::Relaxed), 4);
    }
}
