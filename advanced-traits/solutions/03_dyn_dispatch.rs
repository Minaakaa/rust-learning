#![cfg_attr(not(test), allow(dead_code))]

//! # 解答 03: dyn互換なAnalyzerを動的dispatchする

#[derive(Debug, Clone, PartialEq, Eq)]
struct Telemetry {
    robot_id: String,
    battery_mv: Option<u16>,
    distance_cm: Option<u32>,
}

impl Telemetry {
    fn new(robot_id: &str, battery_mv: Option<u16>, distance_cm: Option<u32>) -> Self {
        Self {
            robot_id: robot_id.to_owned(),
            battery_mv,
            distance_cm,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Decision {
    Recharge { robot_id: String, millivolts: u16 },
    EmergencyStop { robot_id: String, distance_cm: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Analysis<O> {
    analyzer: String,
    output: O,
}

trait Named {
    fn name(&self) -> &str;
}

trait Analyzer: Named {
    type Output;

    fn analyze(&mut self, telemetry: &Telemetry) -> Option<Self::Output>;

    fn analyze_all<'a, I>(&mut self, telemetry: I) -> Vec<Self::Output>
    where
        Self: Sized,
        I: IntoIterator<Item = &'a Telemetry>,
    {
        telemetry
            .into_iter()
            .filter_map(|item| self.analyze(item))
            .collect()
    }
}

trait AnalyzerExt: Analyzer {
    fn analyze_named(&mut self, telemetry: &Telemetry) -> Option<Analysis<Self::Output>> {
        let analyzer = self.name().to_owned();
        let output = self.analyze(telemetry)?;

        Some(Analysis { analyzer, output })
    }
}

impl<T> AnalyzerExt for T where T: Analyzer + ?Sized {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BatteryAnalyzer {
    name: String,
    minimum_mv: u16,
    checks: usize,
}

impl BatteryAnalyzer {
    fn new(name: &str, minimum_mv: u16) -> Self {
        Self {
            name: name.to_owned(),
            minimum_mv,
            checks: 0,
        }
    }

    const fn checks(&self) -> usize {
        self.checks
    }
}

impl Named for BatteryAnalyzer {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Analyzer for BatteryAnalyzer {
    type Output = Decision;

    fn analyze(&mut self, telemetry: &Telemetry) -> Option<Self::Output> {
        self.checks += 1;
        let millivolts = telemetry.battery_mv?;

        (millivolts <= self.minimum_mv).then(|| Decision::Recharge {
            robot_id: telemetry.robot_id.clone(),
            millivolts,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DistanceAnalyzer {
    name: String,
    stop_at_cm: u32,
    checks: usize,
}

impl DistanceAnalyzer {
    fn new(name: &str, stop_at_cm: u32) -> Self {
        Self {
            name: name.to_owned(),
            stop_at_cm,
            checks: 0,
        }
    }

    const fn checks(&self) -> usize {
        self.checks
    }
}

impl Named for DistanceAnalyzer {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Analyzer for DistanceAnalyzer {
    type Output = Decision;

    fn analyze(&mut self, telemetry: &Telemetry) -> Option<Self::Output> {
        self.checks += 1;
        let distance_cm = telemetry.distance_cm?;

        (distance_cm <= self.stop_at_cm).then(|| Decision::EmergencyStop {
            robot_id: telemetry.robot_id.clone(),
            distance_cm,
        })
    }
}

type DynDecisionAnalyzer = dyn Analyzer<Output = Decision> + Send + Sync;

fn named_view(analyzer: &DynDecisionAnalyzer) -> &dyn Named {
    analyzer
}

struct AnalyzerRack {
    analyzers: Vec<Box<DynDecisionAnalyzer>>,
}

impl AnalyzerRack {
    const fn new() -> Self {
        Self {
            analyzers: Vec::new(),
        }
    }

    fn register<A>(&mut self, analyzer: A)
    where
        A: Analyzer<Output = Decision> + Send + Sync + 'static,
    {
        self.analyzers.push(Box::new(analyzer));
    }

    fn len(&self) -> usize {
        self.analyzers.len()
    }

    fn is_empty(&self) -> bool {
        self.analyzers.is_empty()
    }

    fn analyzer_names(&self) -> Vec<&str> {
        self.analyzers
            .iter()
            .map(|analyzer| named_view(analyzer.as_ref()).name())
            .collect()
    }

    fn analyze(&mut self, telemetry: &Telemetry) -> Vec<Analysis<Decision>> {
        self.analyzers
            .iter_mut()
            .filter_map(|analyzer| analyzer.as_mut().analyze_named(telemetry))
            .collect()
    }
}

fn analyze_statically<A>(analyzer: &mut A, telemetry: &Telemetry) -> Option<Analysis<Decision>>
where
    A: Analyzer<Output = Decision>,
{
    analyzer.analyze_named(telemetry)
}

fn analyze_borrowed(
    analyzers: &mut [&mut DynDecisionAnalyzer],
    telemetry: &Telemetry,
) -> Vec<Analysis<Decision>> {
    analyzers
        .iter_mut()
        .filter_map(|analyzer| analyzer.analyze_named(telemetry))
        .collect()
}

fn main() {
    let mut rack = AnalyzerRack::new();
    rack.register(BatteryAnalyzer::new("電池監視", 3_300));
    rack.register(DistanceAnalyzer::new("距離監視", 20));

    let telemetry = Telemetry::new("配送ロボット-1201", Some(3_250), Some(12));
    for analysis in rack.analyze(&telemetry) {
        println!("{}: {:?}", analysis.analyzer, analysis.output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alerting_telemetry() -> Telemetry {
        Telemetry::new("R-1201", Some(3_200), Some(10))
    }

    #[test]
    fn 異種analyzerを登録順に動的dispatchする() {
        let mut rack = AnalyzerRack::new();
        rack.register(BatteryAnalyzer::new("battery", 3_300));
        rack.register(DistanceAnalyzer::new("distance", 20));

        let analyses = rack.analyze(&alerting_telemetry());

        assert_eq!(rack.len(), 2);
        assert_eq!(rack.analyzer_names(), ["battery", "distance"]);
        assert_eq!(
            analyses,
            [
                Analysis {
                    analyzer: "battery".to_owned(),
                    output: Decision::Recharge {
                        robot_id: "R-1201".to_owned(),
                        millivolts: 3_200,
                    },
                },
                Analysis {
                    analyzer: "distance".to_owned(),
                    output: Decision::EmergencyStop {
                        robot_id: "R-1201".to_owned(),
                        distance_cm: 10,
                    },
                },
            ]
        );
    }

    #[test]
    fn noneを返したanalyzerだけを結果から除く() {
        let mut rack = AnalyzerRack::new();
        rack.register(BatteryAnalyzer::new("battery", 3_300));
        rack.register(DistanceAnalyzer::new("distance", 20));
        let telemetry = Telemetry::new("R-1202", None, Some(5));

        let analyses = rack.analyze(&telemetry);

        assert_eq!(
            analyses,
            [Analysis {
                analyzer: "distance".to_owned(),
                output: Decision::EmergencyStop {
                    robot_id: "R-1202".to_owned(),
                    distance_cm: 5,
                },
            }]
        );
    }

    #[test]
    fn thresholdの境界値でもdecisionを返す() {
        let telemetry = Telemetry::new("R-1203", Some(3_300), Some(20));
        let mut battery = BatteryAnalyzer::new("battery", 3_300);
        let mut distance = DistanceAnalyzer::new("distance", 20);

        assert!(matches!(
            battery.analyze(&telemetry),
            Some(Decision::Recharge {
                millivolts: 3_300,
                ..
            })
        ));
        assert!(matches!(
            distance.analyze(&telemetry),
            Some(Decision::EmergencyStop {
                distance_cm: 20,
                ..
            })
        ));
    }

    #[test]
    fn analyzerはdecisionの有無にかかわらず検査回数を保持する() {
        let mut battery = BatteryAnalyzer::new("battery", 3_300);
        let mut distance = DistanceAnalyzer::new("distance", 20);
        let normal = Telemetry::new("R-1204", Some(3_500), Some(30));
        let missing = Telemetry::new("R-1204", None, None);
        let alerting = alerting_telemetry();

        for telemetry in [&normal, &missing, &alerting] {
            battery.analyze(telemetry);
            distance.analyze(telemetry);
        }

        assert_eq!(battery.checks(), 3);
        assert_eq!(distance.checks(), 3);
    }

    #[test]
    fn 空rackは空の名前と解析結果を返す() {
        let mut rack = AnalyzerRack::new();

        assert!(rack.is_empty());
        assert_eq!(rack.len(), 0);
        assert!(rack.analyzer_names().is_empty());
        assert!(rack.analyze(&alerting_telemetry()).is_empty());
    }

    #[test]
    fn utf8のanalyzer名とrobot_idを保持する() {
        let mut rack = AnalyzerRack::new();
        rack.register(BatteryAnalyzer::new("電池監視🔋", 3_300));
        let telemetry = Telemetry::new("本郷🤖-壱", Some(3_000), None);

        let analyses = rack.analyze(&telemetry);

        assert_eq!(rack.analyzer_names(), ["電池監視🔋"]);
        assert_eq!(analyses[0].analyzer, "電池監視🔋");
        assert_eq!(
            analyses[0].output,
            Decision::Recharge {
                robot_id: "本郷🤖-壱".to_owned(),
                millivolts: 3_000,
            }
        );
    }

    #[test]
    fn 関連型の等値境界で静的dispatchとgeneric_helperを使う() {
        let mut battery = BatteryAnalyzer::new("battery", 3_300);
        let alert = alerting_telemetry();
        let normal = Telemetry::new("R-1205", Some(3_600), None);

        let analysis: Analysis<Decision> =
            analyze_statically(&mut battery, &alert).expect("境界値未満なので充電判断になる");
        let batch: Vec<Decision> = battery.analyze_all([&normal, &alert]);

        assert_eq!(analysis.analyzer, "battery");
        assert_eq!(batch.len(), 1);
        assert!(matches!(batch[0], Decision::Recharge { .. }));
    }

    #[test]
    fn sizedでないdyn_analyzerにもextension_methodを提供する() {
        let mut battery = BatteryAnalyzer::new("borrowed", 3_300);
        let analyzer: &mut dyn Analyzer<Output = Decision> = &mut battery;

        let analysis = analyzer
            .analyze_named(&alerting_telemetry())
            .expect("dyn Analyzerでも拡張methodを呼べる");

        assert_eq!(analysis.analyzer, "borrowed");
        assert!(matches!(analysis.output, Decision::Recharge { .. }));
    }

    #[test]
    fn supertraitへupcastできdyn境界はsend_syncを要求する() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}

        assert_send_sync::<DynDecisionAnalyzer>();

        let analyzer: Box<DynDecisionAnalyzer> =
            Box::new(BatteryAnalyzer::new("upcast対象", 3_300));
        let named: &dyn Named = named_view(analyzer.as_ref());

        assert_eq!(named.name(), "upcast対象");
    }

    #[test]
    fn trait_objectはborrowでも異種collectionを作れる() {
        let mut battery = BatteryAnalyzer::new("stack-battery", 3_300);
        let mut distance = DistanceAnalyzer::new("stack-distance", 20);
        let mut analyzers: [&mut DynDecisionAnalyzer; 2] = [&mut battery, &mut distance];

        let analyses = analyze_borrowed(&mut analyzers, &alerting_telemetry());

        let names: Vec<_> = analyses
            .iter()
            .map(|analysis| analysis.analyzer.as_str())
            .collect();
        assert_eq!(names, ["stack-battery", "stack-distance"]);
    }
}
