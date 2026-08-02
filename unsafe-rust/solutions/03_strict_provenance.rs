#![cfg_attr(not(test), allow(dead_code))]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

//! # 解答 03: strict provenance を保つoffset handleを作る

use std::{marker::PhantomData, mem::size_of, ptr::NonNull};

#[derive(Debug, PartialEq, Eq)]
struct Telemetry {
    robot_id: String,
    millivolts: i32,
}

impl Telemetry {
    fn new(robot_id: &str, millivolts: i32) -> Self {
        Self {
            robot_id: robot_id.to_owned(),
            millivolts,
        }
    }

    fn robot_id(&self) -> &str {
        &self.robot_id
    }

    const fn millivolts(&self) -> i32 {
        self.millivolts
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TelemetryHandle {
    byte_offset: usize,
}

impl TelemetryHandle {
    const fn from_byte_offset(byte_offset: usize) -> Self {
        Self { byte_offset }
    }

    const fn byte_offset(self) -> usize {
        self.byte_offset
    }
}

struct TelemetryRegion<'a> {
    base: NonNull<Telemetry>,
    len: usize,
    _borrow: PhantomData<&'a [Telemetry]>,
}

impl<'a> TelemetryRegion<'a> {
    fn from_slice(telemetry: &'a [Telemetry]) -> Self {
        let base = telemetry
            .first()
            .map_or_else(NonNull::dangling, NonNull::from);

        Self {
            base,
            len: telemetry.len(),
            _borrow: PhantomData,
        }
    }

    const fn len(&self) -> usize {
        self.len
    }

    const fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn handle_at(&self, index: usize) -> Option<TelemetryHandle> {
        if index >= self.len {
            return None;
        }

        index
            .checked_mul(size_of::<Telemetry>())
            .map(TelemetryHandle::from_byte_offset)
    }

    fn resolve(&self, handle: TelemetryHandle) -> Option<&'a Telemetry> {
        let element_size = size_of::<Telemetry>();
        if !handle.byte_offset.is_multiple_of(element_size) {
            return None;
        }

        let index = handle.byte_offset / element_size;
        if index >= self.len {
            return None;
        }

        let address = self.base.as_ptr().addr().checked_add(handle.byte_offset)?;
        let pointer = self.base.as_ptr().with_addr(address);

        // SAFETY: from_sliceで借用した同じsliceのbase provenanceを保持している
        // offsetは要素境界かつindex < lenと確認済みで、Telemetryは'aの共有borrow中有効である
        Some(unsafe { &*pointer })
    }
}

fn main() {
    let telemetry = vec![
        Telemetry::new("配送ロボット-1101", 3_300),
        Telemetry::new("配送ロボット-1102", 3_280),
    ];
    let region = TelemetryRegion::from_slice(&telemetry);
    let handle = region.handle_at(1).expect("2件目のhandleを作れる");
    let reading = region.resolve(handle).expect("handleを解決できる");

    println!("{}: {} mV", reading.robot_id(), reading.millivolts());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn readings() -> Vec<Telemetry> {
        vec![
            Telemetry::new("本郷🤖-1", 3_300),
            Telemetry::new("駒場🚚-2", 3_280),
            Telemetry::new("柏📦-3", -120),
        ]
    }

    #[test]
    fn indexをbyte_offsetへ変換する() {
        let telemetry = readings();
        let region = TelemetryRegion::from_slice(&telemetry);
        let element_size = size_of::<Telemetry>();

        assert_eq!(region.handle_at(0).unwrap().byte_offset(), 0);
        assert_eq!(region.handle_at(1).unwrap().byte_offset(), element_size);
        assert_eq!(region.handle_at(2).unwrap().byte_offset(), element_size * 2);
        assert_eq!(region.handle_at(3), None);
    }

    #[test]
    fn handleを同じallocation内の参照へ戻す() {
        let telemetry = readings();
        let id_pointer = telemetry[1].robot_id.as_ptr();
        let region = TelemetryRegion::from_slice(&telemetry);
        let handle = region.handle_at(1).unwrap();
        let resolved = region.resolve(handle).expect("2件目を解決できる");

        assert_eq!(resolved.robot_id(), "駒場🚚-2");
        assert_eq!(resolved.millivolts(), 3_280);
        assert_eq!(resolved.robot_id.as_ptr(), id_pointer);
        assert!(std::ptr::eq(resolved, &telemetry[1]));
    }

    #[test]
    fn 要素境界でないoffsetを拒否する() {
        let telemetry = readings();
        let region = TelemetryRegion::from_slice(&telemetry);

        assert_eq!(region.resolve(TelemetryHandle::from_byte_offset(1)), None);
        assert_eq!(
            region.resolve(TelemetryHandle::from_byte_offset(
                size_of::<Telemetry>() + 1
            )),
            None
        );
    }

    #[test]
    fn 範囲外と極端なoffsetを拒否する() {
        let telemetry = readings();
        let region = TelemetryRegion::from_slice(&telemetry);

        assert_eq!(
            region.resolve(TelemetryHandle::from_byte_offset(
                size_of::<Telemetry>() * telemetry.len()
            )),
            None
        );
        assert_eq!(
            region.resolve(TelemetryHandle::from_byte_offset(usize::MAX)),
            None
        );
    }

    #[test]
    fn 空regionはどのhandleも拒否する() {
        let telemetry = Vec::<Telemetry>::new();
        let region = TelemetryRegion::from_slice(&telemetry);

        assert!(region.is_empty());
        assert_eq!(region.len(), 0);
        assert_eq!(region.handle_at(0), None);
        assert_eq!(region.resolve(TelemetryHandle::from_byte_offset(0)), None);
    }

    #[test]
    fn handleはregion相対でprovenanceを所有しない() {
        let first = readings();
        let second = vec![
            Telemetry::new("別region-0", 10),
            Telemetry::new("別region-1", 20),
            Telemetry::new("別region-2", 30),
        ];
        let first_region = TelemetryRegion::from_slice(&first);
        let second_region = TelemetryRegion::from_slice(&second);
        let handle = first_region.handle_at(2).unwrap();

        assert_eq!(first_region.resolve(handle).unwrap().robot_id(), "柏📦-3");
        assert_eq!(
            second_region.resolve(handle).unwrap().robot_id(),
            "別region-2"
        );
    }
}
