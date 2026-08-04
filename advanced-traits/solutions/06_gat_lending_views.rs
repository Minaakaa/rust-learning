//! # 解答 06: GATで所有logからborrowed viewを貸し出す
//!
//! `ViewLender`は、所有する記録を移動や複製せずに1件ずつ公開します
//! 通常の関連型が実装ごとに1つの型を選ぶのに対し、GATの`View<'a>`は
//! lifetimeごとに型を選ぶ「型族」を表します
//!
//! `next_view`の`&'a mut self`からは`Self: 'a`が導かれるため、Rust Referenceの
//! required where clause規則に従い、GAT側にも`where Self: 'a`が必要です
//! 戻り値のviewはsourceのmutable borrowへ結び付くので、viewを使っている間は
//! 同じsourceから次のviewを取得したり、sourceを変更したりできません
//!
//! `lend_next`は具体的なview型を列挙しないgeneric consumerです
//! viewを貸し出すための`Clone`境界や追加allocationは不要で、各実装は異なる
//! borrowed viewを選べます

trait SampleView {
    fn sequence(&self) -> u64;
    fn source(&self) -> &str;
    fn detail(&self) -> &str;
    fn requires_attention(&self) -> bool;
}

trait ViewLender {
    type View<'a>: SampleView
    where
        Self: 'a;

    fn next_view<'a>(&'a mut self) -> Option<Self::View<'a>>;
}

/// source固有のGATを保ったまま次のborrowed viewを返すgeneric consumer
fn lend_next<'a, S>(source: &'a mut S) -> Option<S::View<'a>>
where
    S: ViewLender,
{
    source.next_view()
}

struct TelemetrySample {
    sequence: u64,
    sensor: String,
    reading: String,
    alert: bool,
}

impl TelemetrySample {
    fn new(
        sequence: u64,
        sensor: impl Into<String>,
        reading: impl Into<String>,
        alert: bool,
    ) -> Self {
        Self {
            sequence,
            sensor: sensor.into(),
            reading: reading.into(),
            alert,
        }
    }
}

struct TelemetryView<'a> {
    sample: &'a TelemetrySample,
}

impl SampleView for TelemetryView<'_> {
    fn sequence(&self) -> u64 {
        self.sample.sequence
    }

    fn source(&self) -> &str {
        &self.sample.sensor
    }

    fn detail(&self) -> &str {
        &self.sample.reading
    }

    fn requires_attention(&self) -> bool {
        self.sample.alert
    }
}

struct TelemetryLog {
    samples: Vec<TelemetrySample>,
    cursor: usize,
}

impl TelemetryLog {
    fn new(samples: impl IntoIterator<Item = TelemetrySample>) -> Self {
        Self {
            samples: samples.into_iter().collect(),
            cursor: 0,
        }
    }

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn remaining(&self) -> usize {
        self.samples.len().saturating_sub(self.cursor)
    }
}

impl ViewLender for TelemetryLog {
    type View<'a>
        = TelemetryView<'a>
    where
        Self: 'a;

    fn next_view<'a>(&'a mut self) -> Option<Self::View<'a>> {
        let sample = self.samples.get(self.cursor)?;
        self.cursor += 1;
        Some(TelemetryView { sample })
    }
}

struct InspectionEntry {
    ticket: u64,
    component: String,
    note: String,
    passed: bool,
}

impl InspectionEntry {
    fn new(
        ticket: u64,
        component: impl Into<String>,
        note: impl Into<String>,
        passed: bool,
    ) -> Self {
        Self {
            ticket,
            component: component.into(),
            note: note.into(),
            passed,
        }
    }
}

struct InspectionView<'a> {
    entry: &'a InspectionEntry,
}

impl SampleView for InspectionView<'_> {
    fn sequence(&self) -> u64 {
        self.entry.ticket
    }

    fn source(&self) -> &str {
        &self.entry.component
    }

    fn detail(&self) -> &str {
        &self.entry.note
    }

    fn requires_attention(&self) -> bool {
        !self.entry.passed
    }
}

struct InspectionLog {
    entries: Vec<InspectionEntry>,
    cursor: usize,
}

impl InspectionLog {
    fn new(entries: impl IntoIterator<Item = InspectionEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
            cursor: 0,
        }
    }

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn remaining(&self) -> usize {
        self.entries.len().saturating_sub(self.cursor)
    }
}

impl ViewLender for InspectionLog {
    type View<'a>
        = InspectionView<'a>
    where
        Self: 'a;

    fn next_view<'a>(&'a mut self) -> Option<Self::View<'a>> {
        let entry = self.entries.get(self.cursor)?;
        self.cursor += 1;
        Some(InspectionView { entry })
    }
}

fn main() {
    let mut telemetry = TelemetryLog::new([TelemetrySample::new(
        1201,
        "配送ロボット🤖",
        "battery=82%",
        false,
    )]);
    if let Some(view) = lend_next(&mut telemetry) {
        println!(
            "#{} {}: {} / 要確認={}",
            view.sequence(),
            view.source(),
            view.detail(),
            view.requires_attention()
        );
    }
    println!(
        "遠隔測定log: cursor={}, 残り={}件",
        telemetry.cursor(),
        telemetry.remaining()
    );

    let mut inspections =
        InspectionLog::new([InspectionEntry::new(77, "左アーム", "締結を再確認", false)]);
    if let Some(view) = lend_next(&mut inspections) {
        println!(
            "#{} {}: {} / 要確認={}",
            view.sequence(),
            view.source(),
            view.detail(),
            view.requires_attention()
        );
    }
    println!(
        "点検log: cursor={}, 残り={}件",
        inspections.cursor(),
        inspections.remaining()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 空のlogはviewを返さずcursorも進めない() {
        let mut telemetry = TelemetryLog::new([]);

        assert!(lend_next(&mut telemetry).is_none());
        assert_eq!(telemetry.cursor(), 0);
        assert_eq!(telemetry.remaining(), 0);
    }

    #[test]
    fn telemetryを入力順に1件ずつ貸し出す() {
        let mut telemetry = TelemetryLog::new([
            TelemetrySample::new(10, "front", "distance=20cm", false),
            TelemetrySample::new(11, "rear", "distance=4cm", true),
        ]);

        {
            let first = lend_next(&mut telemetry).expect("1件目がある");
            assert_eq!(first.sequence(), 10);
            assert_eq!(first.source(), "front");
            assert_eq!(first.detail(), "distance=20cm");
            assert!(!first.requires_attention());
        }
        {
            let second = lend_next(&mut telemetry).expect("2件目がある");
            assert_eq!(second.sequence(), 11);
            assert_eq!(second.source(), "rear");
            assert_eq!(second.detail(), "distance=4cm");
            assert!(second.requires_attention());
        }

        assert!(lend_next(&mut telemetry).is_none());
    }

    #[test]
    fn sequenceの境界値をそのままviewへ公開する() {
        let mut telemetry = TelemetryLog::new([
            TelemetrySample::new(0, "minimum", "開始", false),
            TelemetrySample::new(u64::MAX, "maximum", "終了", true),
        ]);

        assert_eq!(lend_next(&mut telemetry).unwrap().sequence(), 0);
        assert_eq!(lend_next(&mut telemetry).unwrap().sequence(), u64::MAX);
    }

    #[test]
    fn utf8のsourceとdetailをbyte境界で切らずに借用する() {
        let mut telemetry = TelemetryLog::new([TelemetrySample::new(
            42,
            "東京大学・配送ロボット🚚",
            "温度=24℃／状態=正常",
            false,
        )]);

        let view = lend_next(&mut telemetry).unwrap();

        assert_eq!(view.source(), "東京大学・配送ロボット🚚");
        assert_eq!(view.detail(), "温度=24℃／状態=正常");
    }

    #[test]
    fn viewは元のstringをcloneせず同じallocationを参照する() {
        let sensor = String::from("所有sensor");
        let reading = String::from("payload📡");
        let sensor_pointer = sensor.as_ptr();
        let reading_pointer = reading.as_ptr();
        let mut telemetry = TelemetryLog::new([TelemetrySample::new(1, sensor, reading, false)]);

        let view = lend_next(&mut telemetry).unwrap();

        assert_eq!(view.source().as_ptr(), sensor_pointer);
        assert_eq!(view.detail().as_ptr(), reading_pointer);
    }

    #[test]
    fn viewを使い終えると次のmutable_borrowを開始できる() {
        let mut telemetry = TelemetryLog::new([
            TelemetrySample::new(1, "one", "first", false),
            TelemetrySample::new(2, "two", "second", false),
        ]);

        {
            let first = telemetry.next_view().unwrap();
            assert_eq!(first.detail(), "first");
            // `first`を使用中は`telemetry.next_view()`を再び呼べない
        }
        assert_eq!(telemetry.cursor(), 1);
        assert_eq!(telemetry.remaining(), 1);

        let second = telemetry.next_view().unwrap();
        assert_eq!(second.detail(), "second");
    }

    #[test]
    fn inspection_logはtelemetryと異なるview型族を選べる() {
        fn accept_telemetry(_: TelemetryView<'_>) {}
        fn accept_inspection(_: InspectionView<'_>) {}

        let mut telemetry = TelemetryLog::new([TelemetrySample::new(5, "camera", "clear", false)]);
        let mut inspections = InspectionLog::new([InspectionEntry::new(
            90,
            "右ホイール",
            "ボルトを増し締め",
            false,
        )]);

        accept_telemetry(lend_next(&mut telemetry).unwrap());
        let inspection = lend_next(&mut inspections).unwrap();
        assert_eq!(inspection.sequence(), 90);
        assert_eq!(inspection.source(), "右ホイール");
        assert_eq!(inspection.detail(), "ボルトを増し締め");
        assert!(inspection.requires_attention());
        accept_inspection(inspection);
        assert_eq!(inspections.cursor(), 1);
        assert_eq!(inspections.remaining(), 0);
    }

    #[test]
    fn generic_consumerは両方のsource固有viewを返す() {
        let mut telemetry = TelemetryLog::new([TelemetrySample::new(7, "battery", "88%", false)]);
        let mut inspections = InspectionLog::new([InspectionEntry::new(
            8,
            "非常停止ボタン",
            "動作確認済み",
            true,
        )]);

        let telemetry_view = lend_next(&mut telemetry).unwrap();
        assert_eq!(telemetry_view.detail(), "88%");

        let inspection_view = lend_next(&mut inspections).unwrap();
        assert_eq!(inspection_view.detail(), "動作確認済み");
        assert!(!inspection_view.requires_attention());
    }

    #[test]
    fn viewにもsourceにもclone境界は不要() {
        struct NonCloneSource {
            message: String,
            yielded: bool,
        }

        struct NonCloneView<'a> {
            message: &'a str,
        }

        impl SampleView for NonCloneView<'_> {
            fn sequence(&self) -> u64 {
                1
            }

            fn source(&self) -> &str {
                "non-clone"
            }

            fn detail(&self) -> &str {
                self.message
            }

            fn requires_attention(&self) -> bool {
                false
            }
        }

        impl ViewLender for NonCloneSource {
            type View<'a>
                = NonCloneView<'a>
            where
                Self: 'a;

            fn next_view<'a>(&'a mut self) -> Option<Self::View<'a>> {
                if self.yielded {
                    return None;
                }
                self.yielded = true;
                Some(NonCloneView {
                    message: &self.message,
                })
            }
        }

        let mut source = NonCloneSource {
            message: String::from("複製不要"),
            yielded: false,
        };

        assert_eq!(lend_next(&mut source).unwrap().detail(), "複製不要");
        assert!(lend_next(&mut source).is_none());
    }
}
