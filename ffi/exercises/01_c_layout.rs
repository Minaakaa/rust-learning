#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 01: `repr(C)` と ABI 安全なデータ型
//!
//! C と共有する struct は Rust 独自の layout に依存できません
//! 固定幅整数と `#[repr(C)]`、明示的な enum 表現を使って配送記録の header を作ってください
//!
//! 仕様:
//! - `PacketKind` は C から渡される `u32` code と相互変換する
//! - `TelemetryPacket` は `#[repr(C)]` とし、field の順序を固定する
//! - `packet_from_readings` は長さを `u32` へ変換できる場合だけ packet を返す
//! - `packet_is_valid` は未定義の kind code を受け付けない
//! - C ABI の境界へ Rust の `String` や `Vec` を置かない

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PacketKind {
    Battery = 1,
    Position = 2,
}

impl PacketKind {
    fn from_code(code: u32) -> Option<Self> {
        let _ = code;
        todo!("Cから届いたcodeをPacketKindへ検証付きで変換してください")
    }

    const fn code(self) -> u32 {
        self as u32
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TelemetryPacket {
    kind_code: u32,
    robot_id: u32,
    reading_count: u32,
}

fn packet_from_readings(
    kind: PacketKind,
    robot_id: u32,
    readings: &[u16],
) -> Option<TelemetryPacket> {
    let _ = (kind, robot_id, readings);
    todo!("測定値の長さをu32へ変換してTelemetryPacketを構築してください")
}

fn packet_is_valid(packet: &TelemetryPacket) -> bool {
    let _ = packet;
    todo!("kindの値が定義済みかを検証してください")
}

fn main() {
    let readings = [3_300_u16, 3_280, 3_275];
    let packet = packet_from_readings(PacketKind::Battery, 1301, &readings)
        .expect("通常の測定値からpacketを作れるはずです");
    println!(
        "kind={} robot={} readings={} valid={}",
        packet.kind_code,
        packet.robot_id,
        packet.reading_count,
        packet_is_valid(&packet)
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    #[test]
    fn c_layoutのfield順とサイズを固定する() {
        assert_eq!(size_of::<PacketKind>(), size_of::<u32>());
        assert_eq!(align_of::<TelemetryPacket>(), align_of::<u32>());
        assert_eq!(size_of::<TelemetryPacket>(), 12);
    }

    #[test]
    fn kind_codeを検証付きで変換する() {
        assert_eq!(PacketKind::from_code(1), Some(PacketKind::Battery));
        assert_eq!(PacketKind::from_code(2), Some(PacketKind::Position));
        assert_eq!(PacketKind::from_code(0), None);
        assert_eq!(PacketKind::from_code(99), None);
    }

    #[test]
    fn packetへ測定件数を記録する() {
        let packet = packet_from_readings(PacketKind::Position, 7, &[10, 20, 30])
            .expect("u32へ変換できる長さです");

        assert_eq!(packet.kind_code, PacketKind::Position.code());
        assert_eq!(packet.robot_id, 7);
        assert_eq!(packet.reading_count, 3);
        assert!(packet_is_valid(&packet));
    }

    #[test]
    fn zero件のpacketも表現できる() {
        let packet = packet_from_readings(PacketKind::Battery, 0, &[])
            .expect("空sliceの長さはu32へ変換できます");

        assert_eq!(packet.reading_count, 0);
        assert!(packet_is_valid(&packet));
    }
}
