#![cfg_attr(not(test), allow(dead_code))]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

//! # 解答 01: raw pointer を安全な読み取り専用sliceとして包む

use std::{marker::PhantomData, ptr::NonNull};

struct RawSlice<'a, T> {
    ptr: NonNull<T>,
    len: usize,
    _borrow: PhantomData<&'a [T]>,
}

impl<'a, T> RawSlice<'a, T> {
    fn from_slice(values: &'a [T]) -> Self {
        Self {
            ptr: NonNull::from(values).cast(),
            len: values.len(),
            _borrow: PhantomData,
        }
    }

    /// raw pointerと要素数から読み取り専用viewを作る
    ///
    /// # Safety
    ///
    /// 呼び出し側は次の条件をすべて保証する必要がある
    ///
    /// - `ptr` はnon-nullで、長さ0やzero-sized typeの場合も `T` に正しくalignされている
    /// - `ptr` から `len` 個の要素は、有効な `T` として初期化済みである
    /// - byte範囲が空でなければ、`ptr` はその範囲を含む1つのallocationのprovenanceを持つ
    /// - 参照先はlifetime `'a` の間、読み取り可能で有効である
    /// - `'a` の間、参照先を `UnsafeCell` の外から変更せず、共有参照のaliasing規則を守る
    /// - `len * size_of::<T>()` は `isize::MAX` 以下で、address計算がwrapしない
    unsafe fn from_raw_parts(ptr: NonNull<T>, len: usize) -> Self {
        Self {
            ptr,
            len,
            _borrow: PhantomData,
        }
    }

    const fn len(&self) -> usize {
        self.len
    }

    const fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.as_slice().get(index)
    }

    fn as_slice(&self) -> &[T] {
        // SAFETY: safe constructorは元sliceから条件を継承し、unsafe constructorは
        // 同じ条件を呼び出し側へ要求するため、保持中はfrom_raw_partsの全条件が成立する
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

fn main() {
    let readings = ["配送ロボット-1: 3300 mV", "点検ロボット🤖-2: 3270 mV"];
    let view = RawSlice::from_slice(&readings);

    println!("記録数: {}", view.len());
    if let Some(reading) = view.get(1) {
        println!("2件目: {reading}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn non_cloneのtelemetryを読み取れる() {
        let readings = vec![
            Telemetry::new("配送ロボット-1", 3_300),
            Telemetry::new("点検ロボット🤖-二号", 3_270),
        ];
        let view = RawSlice::from_slice(&readings);

        assert_eq!(view.len(), 2);
        assert!(!view.is_empty());
        assert_eq!(view.get(0), Some(&Telemetry::new("配送ロボット-1", 3_300)));
        assert_eq!(
            view.get(1),
            Some(&Telemetry::new("点検ロボット🤖-二号", 3_270))
        );
    }

    #[test]
    fn getは範囲外のindexを拒否する() {
        let readings = [10_u16, 20, 30];
        let view = RawSlice::from_slice(&readings);

        assert_eq!(view.get(2), Some(&30));
        assert_eq!(view.get(3), None);
        assert_eq!(view.get(usize::MAX), None);
    }

    #[test]
    fn sliceのpointerとallocationをそのまま使う() {
        let readings = vec![
            Telemetry::new("観測機-甲", 4_001),
            Telemetry::new("観測機-乙", 4_002),
        ];
        let outer_pointer = readings.as_ptr();
        let id_pointer = readings[1].robot_id.as_ptr();
        let view = RawSlice::from_slice(&readings);

        let slice = view.as_slice();

        assert_eq!(slice.as_ptr(), outer_pointer);
        assert_eq!(slice.len(), readings.len());
        assert_eq!(slice[1].robot_id.as_ptr(), id_pointer);
    }

    #[test]
    fn raw_partsから同じ読み取り専用sliceを復元する() {
        let readings = vec![
            Telemetry::new("海中探査機🌊-1", 2_900),
            Telemetry::new("海中探査機🌊-2", 2_850),
        ];
        let pointer =
            NonNull::new(readings.as_ptr().cast_mut()).expect("Vecのpointerはnullではない");
        let id_pointer = readings[0].robot_id.as_ptr();

        // SAFETY: pointerと長さは生存中のreadings全体から取得し、test中は変更しない
        let view = unsafe { RawSlice::from_raw_parts(pointer, readings.len()) };

        assert_eq!(view.as_slice(), readings.as_slice());
        assert_eq!(view.as_slice().as_ptr(), readings.as_ptr());
        assert_eq!(
            view.get(0).expect("先頭要素がある").robot_id.as_ptr(),
            id_pointer
        );
    }

    #[test]
    fn 空sliceとaligned_dangling_pointerを扱える() {
        let empty: [Telemetry; 0] = [];
        let from_slice = RawSlice::from_slice(&empty);

        assert!(from_slice.is_empty());
        assert_eq!(from_slice.len(), 0);
        assert_eq!(from_slice.as_slice().as_ptr(), empty.as_ptr());
        assert_eq!(from_slice.get(0), None);

        // SAFETY: 長さ0でpointerをdereferenceせず、danglingはnon-nullかつTelemetryにalign済みである
        let from_raw = unsafe { RawSlice::<'_, Telemetry>::from_raw_parts(NonNull::dangling(), 0) };

        assert!(from_raw.as_slice().is_empty());
        assert_eq!(
            from_raw.as_slice().as_ptr(),
            NonNull::<Telemetry>::dangling().as_ptr()
        );
    }

    #[test]
    fn non_emptyのzero_sized_typeを扱える() {
        #[derive(Debug, PartialEq, Eq)]
        struct Marker;

        let markers = [Marker, Marker, Marker];
        let from_slice = RawSlice::from_slice(&markers);

        assert_eq!(from_slice.len(), 3);
        assert_eq!(from_slice.as_slice(), &markers);

        // SAFETY: ZSTはmemoryを読み取らず、danglingはnon-nullかつMarkerにalign済みである
        let from_raw =
            unsafe { RawSlice::<'_, Marker>::from_raw_parts(NonNull::dangling(), markers.len()) };

        assert_eq!(from_raw.len(), 3);
        assert_eq!(from_raw.get(2), Some(&Marker));
    }

    #[test]
    fn 戻り値の参照はraw_sliceのborrowに結び付く() {
        fn first<'view, 'data: 'view>(
            view: &'view RawSlice<'data, Telemetry>,
        ) -> Option<&'view Telemetry> {
            view.get(0)
        }

        let readings = [Telemetry::new("寿命確認ロボット", 3_123)];
        let view = RawSlice::from_slice(&readings);

        assert_eq!(first(&view), readings.first());
    }
}
