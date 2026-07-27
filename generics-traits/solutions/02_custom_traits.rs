//! 問題 02 の解答例

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
        format!(
            "[{}] {}: {}",
            self.severity().label(),
            self.source(),
            self.detail()
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
        &self.robot_id
    }

    fn severity(&self) -> Severity {
        if self.remaining_percent < 20 {
            Severity::Warning
        } else {
            Severity::Info
        }
    }

    fn detail(&self) -> String {
        format!("バッテリー残量 {}%", self.remaining_percent)
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
        &self.mission_id
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn detail(&self) -> String {
        if self.completed {
            format!("{}への配送完了", self.destination)
        } else {
            format!("{}へ配送中", self.destination)
        }
    }

    fn summary(&self) -> String {
        let state = if self.completed {
            "完了"
        } else {
            "進行中"
        };
        format!("配送 {}｜{}｜{state}", self.mission_id, self.destination)
    }
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
