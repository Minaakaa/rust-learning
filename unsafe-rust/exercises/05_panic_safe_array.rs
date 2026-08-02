#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 05: panic-safeな配列builderを作る
//!
//! 各indexの初期化が失敗し得るclosureから`[T; N]`を組み立てます
//! `Err`やpanicで途中終了しても初期化済みelementを各1回だけdropし、成功時は
//! 全elementの所有権を完成した配列へ移す`try_build_array`を完成させてください
//!
//! 仕様:
//! - `values[..initialized]`だけが初期化済みという不変条件を保つ
//! - initializerは`0..N`の各indexについて順番に1回ずつ呼ぶ
//! - `Ok(value)`をslotへwriteした後だけ`initialized`を増やす
//! - `Err(error)`は変更せず呼び出し側へ返す
//! - initializerが`Err`を返した場合もpanicした場合も、完成済みprefixを逆順でdropする
//! - 成功時は全elementを`[T; N]`へ1回だけmoveし、guardを明示的に無効化する
//! - `N == 0`ではinitializerを呼ばず空配列を返す
//! - safe functionの内部だけで`unsafe`を使い、各blockへ日本語の`SAFETY`根拠を書く
//!
//! ヒント:
//! - builder自身に`Drop`を実装すると`?`とunwindの両方を同じcleanup経路で扱える
//! - `MaybeUninit::write`が成功してから`initialized`を増やす
//! - drop時はcountを先に減らしてから`assume_init_drop`を呼ぶ
//! - 完成時は`ManuallyDrop`でguardを無効化してから、全slotを配列として1回だけ読む

use std::mem::{ManuallyDrop, MaybeUninit};

struct PartialArray<T, const N: usize> {
    values: [MaybeUninit<T>; N],
    initialized: usize,
}

impl<T, const N: usize> PartialArray<T, N> {
    fn new() -> Self {
        todo!("{N}個の未初期化slotとinitialized=0を用意してください")
    }

    fn push(&mut self, value: T) {
        let _ = &value;
        todo!(
            "index {} へ値をwriteしてからinitializedを増やしてください: slot数={}",
            self.initialized,
            self.values.len()
        )
    }

    fn finish(self) -> [T; N] {
        let _ = ManuallyDrop::new(self);
        todo!("全slotの初期化を確認し、guardを無効化して配列へ所有権を移してください")
    }
}

impl<T, const N: usize> Drop for PartialArray<T, N> {
    fn drop(&mut self) {
        // TODO: initializedを先に減らし、初期化済みprefixを逆順でdropする
        // starterの未実装Dropはunwind中の二重panicを避けるため意図的にpanicしない
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
