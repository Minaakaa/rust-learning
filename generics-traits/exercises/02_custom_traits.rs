#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 02: トレイトで状態報告を統一する
//!
//! 管制画面には、ロボットのバッテリー状態と配送ミッションの進捗を同じ形式で表示します
//! 2つの構造体に継承関係はありませんが、`StatusReport` を実装すれば共通の操作で扱えます
//!
//! 仕様:
//! - `source` は報告元のロボット ID またはミッション ID を借用して返す
//! - `severity` は報告の重要度を返す
//! - `detail` は各型に固有の日本語メッセージを返す
//! - デフォルトの `summary` は `[重要度] 報告元: 詳細` の形式にする
//! - バッテリー残量が 20% 未満の場合だけ `Warning` にする
//! - `BatteryStatus` はデフォルトの `summary` をそのまま使う
//! - `DeliveryStatus` は `summary` を上書きし、配送向けの短い表示にする
//!
//! トレイトの必須メソッドとデフォルトメソッド、それぞれの型に固有の実装を完成させてください

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Severity {
    Info,
    Warning,
}

impl Severity {
    const fn label(self) -> &'static str {
        match self {
            Self::Info => "情報",
            Self::Warning => "警告",
        }
    }
}

trait StatusReport {
    fn source(&self) -> &str;

    fn severity(&self) -> Severity;

    fn detail(&self) -> String;

    fn summary(&self) -> String {
        todo!(
            "報告元 {} と重要度 {} と詳細を既定の形式でまとめてください",
            self.source(),
            self.severity().label()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BatteryStatus {
    robot_id: String,
    remaining_percent: u8,
}

impl BatteryStatus {
    fn new(robot_id: &str, remaining_percent: u8) -> Self {
        Self {
            robot_id: robot_id.to_string(),
            remaining_percent,
        }
    }
}

impl StatusReport for BatteryStatus {
    fn source(&self) -> &str {
        todo!("ロボット ID {:?} を借用してください", self.robot_id)
    }

    fn severity(&self) -> Severity {
        todo!(
            "バッテリー残量 {}% から重要度を決めてください",
            self.remaining_percent
        )
    }

    fn detail(&self) -> String {
        todo!(
            "バッテリー残量 {}% の詳細を作ってください",
            self.remaining_percent
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeliveryStatus {
    mission_id: String,
    destination: String,
    completed: bool,
}

impl DeliveryStatus {
    fn new(mission_id: &str, destination: &str, completed: bool) -> Self {
        Self {
            mission_id: mission_id.to_string(),
            destination: destination.to_string(),
            completed,
        }
    }
}

impl StatusReport for DeliveryStatus {
    fn source(&self) -> &str {
        todo!("ミッション ID {:?} を借用してください", self.mission_id)
    }

    fn severity(&self) -> Severity {
        todo!("通常の配送報告は情報として扱ってください")
    }

    fn detail(&self) -> String {
        todo!(
            "配送先 {:?} と完了状態 {} から詳細を作ってください",
            self.destination,
            self.completed
        )
    }

    fn summary(&self) -> String {
        todo!(
            "ミッション {} の配送向け summary を作ってください",
            self.mission_id
        )
    }
}

fn main() {
    let battery = BatteryStatus::new("RB-52", 18);
    let delivery = DeliveryStatus::new("M-520", "図書館", false);

    println!("{}", battery.summary());
    println!("{}", delivery.summary());
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DefaultReport;

    impl StatusReport for DefaultReport {
        fn source(&self) -> &str {
            "TEST-01"
        }

        fn severity(&self) -> Severity {
            Severity::Info
        }

        fn detail(&self) -> String {
            "デフォルト動作を確認".to_string()
        }
    }

    fn summary_of(report: &impl StatusReport) -> String {
        report.summary()
    }

    #[test]
    fn バッテリー報告の必須メソッドを実装する() {
        let status = BatteryStatus::new("RB-52", 65);

        assert_eq!(status.source(), "RB-52");
        assert_eq!(status.severity(), Severity::Info);
        assert_eq!(status.detail(), "バッテリー残量 65%");
    }

    #[test]
    fn 残量20パーセント未満だけを警告にする() {
        assert_eq!(BatteryStatus::new("RB-53", 100).severity(), Severity::Info);
        assert_eq!(BatteryStatus::new("RB-53", 20).severity(), Severity::Info);
        assert_eq!(
            BatteryStatus::new("RB-53", 19).severity(),
            Severity::Warning
        );
        assert_eq!(BatteryStatus::new("RB-53", 0).severity(), Severity::Warning);
    }

    #[test]
    fn バッテリー報告ではデフォルトsummaryを使う() {
        let normal = BatteryStatus::new("RB-54", 80);
        let warning = BatteryStatus::new("RB-55", 7);

        assert_eq!(normal.summary(), "[情報] RB-54: バッテリー残量 80%");
        assert_eq!(warning.summary(), "[警告] RB-55: バッテリー残量 7%");
    }

    #[test]
    fn 必須メソッドだけの実装でもデフォルトsummaryを使える() {
        assert_eq!(
            DefaultReport.summary(),
            "[情報] TEST-01: デフォルト動作を確認"
        );
    }

    #[test]
    fn 配送報告の必須メソッドを実装する() {
        let active = DeliveryStatus::new("M-521", "研究棟", false);
        let completed = DeliveryStatus::new("M-522", "食堂", true);

        assert_eq!(active.source(), "M-521");
        assert_eq!(active.severity(), Severity::Info);
        assert_eq!(active.detail(), "研究棟へ配送中");
        assert_eq!(completed.detail(), "食堂への配送完了");
    }

    #[test]
    fn 配送報告ではsummaryを上書きする() {
        let active = DeliveryStatus::new("M-523", "学生寮", false);
        let completed = DeliveryStatus::new("M-524", "図書館", true);

        assert_eq!(active.summary(), "配送 M-523｜学生寮｜進行中");
        assert_eq!(completed.summary(), "配送 M-524｜図書館｜完了");
    }

    #[test]
    fn 同じトレイト境界から型ごとのsummaryを呼び分ける() {
        let battery = BatteryStatus::new("RB-56", 40);
        let delivery = DeliveryStatus::new("M-525", "保健センター", true);

        assert_eq!(summary_of(&battery), "[情報] RB-56: バッテリー残量 40%");
        assert_eq!(summary_of(&delivery), "配送 M-525｜保健センター｜完了");
    }

    #[test]
    fn 日本語と絵文字を報告に保持する() {
        let battery = BatteryStatus::new("配送ロボ🤖-01", 12);
        let delivery = DeliveryStatus::new("緊急便🚚-02", "工学部Ａ棟・受付", false);

        assert_eq!(
            battery.summary(),
            "[警告] 配送ロボ🤖-01: バッテリー残量 12%"
        );
        assert_eq!(
            delivery.summary(),
            "配送 緊急便🚚-02｜工学部Ａ棟・受付｜進行中"
        );
    }
}
