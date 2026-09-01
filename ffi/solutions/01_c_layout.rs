//! # 解答 01: `repr(C)` と ABI 安全なデータ型

use std::mem::{align_of, size_of};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PacketKind {
    Battery = 1,
    Position = 2,
}

impl PacketKind {
    fn from_code(code: u32) -> Option<Self> {
        match code {
            1 => Some(Self::Battery),
            2 => Some(Self::Position),
            _ => None,
        }
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
    let reading_count = u32::try_from(readings.len()).ok()?;
    Some(TelemetryPacket {
        kind_code: kind.code(),
        robot_id,
        reading_count,
    })
}

fn packet_is_valid(packet: &TelemetryPacket) -> bool {
    PacketKind::from_code(packet.kind_code).is_some()
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
