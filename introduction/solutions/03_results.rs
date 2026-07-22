//! 問題 03 の解答例。

#[derive(Debug, Clone, PartialEq, Eq)]
struct RouteStop {
    building: String,
    distance_m: u32,
}

fn parse_stop(input: &str) -> Result<RouteStop, String> {
    let (building, distance) = input
        .split_once(':')
        .ok_or_else(|| format!("区切り文字 ':' がありません: {input}"))?;

    let building = building.trim();
    if building.is_empty() {
        return Err("建物名が空です".to_string());
    }

    let distance = distance.trim();
    let distance_m = distance
        .parse::<u32>()
        .map_err(|_| format!("距離が整数ではありません: {distance}"))?;
    if distance_m == 0 {
        return Err("距離は 1 以上でなければなりません".to_string());
    }

    Ok(RouteStop {
        building: building.to_string(),
        distance_m,
    })
}

fn total_distance(inputs: &[&str]) -> Result<u32, String> {
    let mut total = 0_u32;

    for input in inputs {
        let stop = parse_stop(input)?;
        total = total
            .checked_add(stop.distance_m)
            .ok_or_else(|| "総距離が u32 の上限を超えました".to_string())?;
    }

    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 前後の空白を除いて停車地点を作る() {
        assert_eq!(
            parse_stop("  図書館 : 120 "),
            Ok(RouteStop {
                building: "図書館".to_string(),
                distance_m: 120,
            })
        );
    }

    #[test]
    fn 区切り文字がなければ入力も含めて報告する() {
        assert_eq!(
            parse_stop("図書館120"),
            Err("区切り文字 ':' がありません: 図書館120".to_string())
        );
    }

    #[test]
    fn 空の建物名を拒否する() {
        assert_eq!(parse_stop("  :120"), Err("建物名が空です".to_string()));
    }

    #[test]
    fn 整数でない距離を元の文字列とともに報告する() {
        assert_eq!(
            parse_stop("研究棟:遠い"),
            Err("距離が整数ではありません: 遠い".to_string())
        );
    }

    #[test]
    fn 距離ゼロを拒否する() {
        assert_eq!(
            parse_stop("研究棟:0"),
            Err("距離は 1 以上でなければなりません".to_string())
        );
    }

    #[test]
    fn すべて成功したときだけ距離を合計する() {
        assert_eq!(
            total_distance(&["図書館:120", "研究棟:350", "食堂:80"]),
            Ok(550)
        );
    }

    #[test]
    fn 途中の解析エラーをそのまま伝播する() {
        assert_eq!(
            total_distance(&["図書館:120", "研究棟:不明", "食堂:80"]),
            Err("距離が整数ではありません: 不明".to_string())
        );
    }

    #[test]
    fn 合計のオーバーフローをエラーにする() {
        assert_eq!(
            total_distance(&["遠方施設:4294967295", "図書館:1"]),
            Err("総距離が u32 の上限を超えました".to_string())
        );
    }
}
