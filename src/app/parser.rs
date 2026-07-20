use std::collections::HashMap;

use chrono::{Local, NaiveDate};

use crate::model::{Frontmatter, Group, Item, Lane, Logbook, Status};

enum ParseState {
    BeforeFrontmatter,
    InFrontmatter,
    Body,
}

fn parse(raw: &str) -> Logbook {
    let mut state = ParseState::BeforeFrontmatter;
    let mut blocks = vec![];
    let mut body_blocks = vec![];

    for line in raw.lines() {
        match &state {
            ParseState::BeforeFrontmatter if line.trim() == "---" => {
                blocks.push(line);
                state = ParseState::InFrontmatter
            }
            ParseState::BeforeFrontmatter => state = ParseState::Body,
            ParseState::InFrontmatter => {
                // Frontmatterが終わりをしめす合図
                if line.trim() == "---" {
                    state = ParseState::Body
                }
                blocks.push(line.trim())
            }
            ParseState::Body => {
                body_blocks.push(line);
            }
        }
    }

    let frontmatter = parse_frontmatter(&blocks);
    let lanes = parse_lanes(&body_blocks);

    Logbook { frontmatter, lanes }
}

fn parse_frontmatter(header_text: &[&str]) -> Frontmatter {
    let mut date = NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d")
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .single()
        .unwrap();
    let mut kind = String::new();
    let mut extra_map = HashMap::new();

    for &text in header_text {
        match text.split_once(":") {
            Some((key, value)) => {
                match key.trim() {
                    "date" => {
                        date = NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
                            .unwrap()
                            .and_hms_opt(10, 00, 00)
                            .unwrap()
                            .and_local_timezone(Local)
                            .single()
                            .unwrap();
                    }
                    "type" => kind = String::from(value.trim()),
                    _ => {
                        extra_map.insert(String::from(key.trim()), String::from(value.trim()));
                    }
                };
            }
            None => {}
        };
    }

    Frontmatter {
        date,
        kind,
        extra: extra_map,
    }
}

fn parse_lanes(lines: &[&str]) -> Vec<Lane> {
    let mut lanes = vec![];
    let mut lane_block = vec![];

    for &line in lines {
        if line.starts_with("##") && !line.starts_with("### ") {
            if !lane_block.is_empty() {
                lanes.push(parse_lane(&lane_block));
            }
            lane_block.clear();
            lane_block.push(line);
        } else {
            lane_block.push(line);
        }
    }
    if !lane_block.is_empty() {
        lanes.push(parse_lane(&lane_block));
    }

    lanes
}

fn parse_lane(lines: &[&str]) -> Lane {
    let mut title = "";
    let mut group_block = vec![];

    for &line in lines {
        if line.starts_with("## ") {
            let body = line.strip_prefix("##");
            title = body.unwrap_or("unknown").trim();
        } else {
            group_block.push(line);
        }
    }

    let groups = parse_groups(&group_block);

    Lane {
        title: title.to_string(),
        groups,
    }
}

fn parse_groups(lines: &[&str]) -> Vec<Group> {
    let mut groups = vec![];
    let mut group_block = vec![];

    for line in lines {
        if line.starts_with("###") {
            if !group_block.is_empty() {
                groups.push(parse_group(&group_block));
            }
            group_block.clear();
            group_block.push(line);
        } else {
            group_block.push(line);
        }
    }
    if !group_block.is_empty() {
        groups.push(parse_group(&group_block));
    }

    groups
}

fn parse_group(lines: &[&str]) -> Group {
    let mut heading = "";
    let mut item_block: Vec<&str> = vec![];
    for &line in lines {
        if line.starts_with("### ") {
            let body = line.strip_prefix("###");
            heading = body.unwrap_or("unknown").trim();
        } else {
            item_block.push(line);
        }
    }

    let items = parse_items(&item_block);

    Group {
        heading: heading.to_string(),
        items,
    }
}

fn parse_items(lines: &[&str]) -> Vec<Item> {
    let mut items = vec![];
    let mut block = vec![];
    for line in lines {
        if line.starts_with("- ") {
            if !block.is_empty() {
                // 次ブロックを処理する前に溜めていたブロックをパースする
                items.push(parse_item(&block));
            }
            block.clear();
            block.push(line);
        } else {
            block.push(line);
        }
    }
    if !block.is_empty() {
        items.push(parse_item(&block));
    }
    items
}

fn parse_item(lines: &[&str]) -> Item {
    let tokens = &lines[0].split_whitespace().collect::<Vec<_>>();
    let title = tokens[1];
    let id = tokens[2];
    let status = lines
        .iter() // linesに&は不要.イテレータは借用することを明示
        .skip(1)
        .find_map(|l| l.trim().strip_prefix("状態::"))
        .map(str::trim)
        .and_then(|status_field| match status_field {
            "未着手" => Some(Status::未着手),
            "着手中" => Some(Status::着手中),
            "待ち" => Some(Status::待ち),
            "順延" => Some(Status::順延),
            "完了" => Some(Status::完了),
            _ => None,
        });
    let mut fields = vec![];
    for line in lines.iter().skip(1) {
        if let Some((k, v)) = line.trim().split_once("::") {
            let key = k.trim().to_string();
            let value = v.trim().to_string();
            if key == "状態" {
                continue;
            }
            fields.push((key, value));
        }
    }

    Item {
        id: id.trim_start_matches("^").into(),
        title: title.into(),
        status,
        fields,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatterからdateとtypeを取り出す() {
        let raw = "---\ndate: 2026-06-26\ntype: logbook\n---\n";
        let lines = raw.lines().collect::<Vec<_>>();
        let frontmatter = parse_frontmatter(&lines);

        assert_eq!(
            frontmatter.date.date_naive(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 26).unwrap()
        );
        assert_eq!(frontmatter.kind, "logbook");
    }

    #[test]
    fn date_type以外のキーはextraに格納する() {
        let raw = "---\ndate: 2026-06-26\ntype: logbook\nmood: 良い\n体調: 普通\n---";
        let lines = raw.lines().collect::<Vec<_>>();
        let frontmatter = parse_frontmatter(&lines);

        assert_eq!(frontmatter.extra.get("mood"), Some(&"良い".to_string()));
        assert_eq!(frontmatter.extra.get("体調"), Some(&"普通".to_string()));
        assert_eq!(frontmatter.extra.len(), 2);
    }

    #[test]
    fn parseでfrontmatterとlaneの両方を取り出す() {
        let raw = "---\ndate: 2026-06-26\ntype: logbook\n---\n## 仕事管理\n### mugenup\n- 反社チェック確認 ^t-1\n  状態:: 待ち\n## 人間管理\n### 振り返り・気付き\n- 定例会で報告できた ^r-1\n  種別:: 良かった";
        let logbook = parse(raw);

        // frontmatter が取れていること
        assert_eq!(
            logbook.frontmatter.date.date_naive(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 26).unwrap()
        );
        assert_eq!(logbook.frontmatter.kind, "logbook");

        // lanes が取れていること（frontmatter 部分がレーンに混ざっていないこと）
        assert_eq!(logbook.lanes.len(), 2);
        assert_eq!(logbook.lanes[0].title, "仕事管理");
        assert_eq!(logbook.lanes[0].groups[0].items[0].id, "t-1");
        assert_eq!(logbook.lanes[1].title, "人間管理");
        assert_eq!(logbook.lanes[1].groups[0].items[0].id, "r-1");
    }

    #[test]
    fn 複数のレーンをパースしてレーンとグループの組み合わせができること() {
        let raw = "## 仕事管理\n### mugenup\n- 反社チェック確認 ^t-1\n  状態:: 待ち\n## 人間管理\n### 振り返り・気付き\n- 定例会で報告できた ^r-1\n  種別:: 良かった";
        let lines = raw.lines().collect::<Vec<_>>();
        let lanes = parse_lanes(&lines);

        assert_eq!(lanes.len(), 2);
        assert_eq!(lanes[0].title, "仕事管理");
        assert_eq!(lanes[0].groups[0].heading, "mugenup");
        assert_eq!(lanes[0].groups[0].items[0].id, "t-1");
        assert_eq!(lanes[1].title, "人間管理");
        assert_eq!(lanes[1].groups[0].heading, "振り返り・気付き");
        assert_eq!(lanes[1].groups[0].items[0].id, "r-1");
    }

    #[test]
    fn 単一のレーンとグループとアイテムをパースして各種要素が取り出せること() {
        let raw = "## 個人プロジェクト\n### test_group1\n- モデル設計 ^t-2\n  状態:: 着手中";
        let lines = raw.lines().collect::<Vec<_>>();
        let lane = parse_lane(&lines);

        assert_eq!(lane.title, "個人プロジェクト");
        assert_eq!(lane.groups.len(), 1);
        assert_eq!(lane.groups[0].heading, "test_group1");
        assert_eq!(lane.groups[0].items.len(), 1);
        assert_eq!(lane.groups[0].items[0].title, "モデル設計");
        assert_eq!(lane.groups[0].items[0].id, "t-2");
        assert_eq!(lane.groups[0].items[0].status, Some(Status::着手中));
    }

    #[test]
    fn 複数のグループとアイテムをパースしてグループとアイテムの組み合わせができること() {
        let raw = "### test_group1\n- 資料確認 ^t-1\n  状態:: 待ち\n### test_group2\n- README確認 ^t-2\n  状態:: 着手中";
        let lines = raw.lines().collect::<Vec<_>>();
        let groups = parse_groups(&lines);

        assert_eq!(groups[0].heading, "test_group1");
        assert_eq!(groups[0].items.len(), 1);
        assert_eq!(groups[1].heading, "test_group2");
        assert_eq!(groups[1].items.len(), 1);
    }

    #[test]
    fn 単一のグループとアイテムをパースしてグループ1つとアイテム1つを取り出す() {
        let raw = "### test_group1\n- 資料確認 ^t-1\n  状態:: 待ち";
        let lines = raw.lines().collect::<Vec<_>>();
        let group = parse_group(&lines);

        assert_eq!(group.heading, "test_group1");
        assert_eq!(group.items.len(), 1);
    }

    #[test]
    fn 単一のアイテム行からタイトルとidを取り出す() {
        let raw = "- ライブラリ調査 ^t-0701-1";
        let lines = raw.lines().collect::<Vec<_>>();
        let item = parse_item(&lines);

        assert_eq!(item.title, "ライブラリ調査");
        assert_eq!(item.id, "t-0701-1");
    }

    #[test]
    fn 状態付きのアイテムから状態を取り出す() {
        let raw = "- 資料確認 ^t-0701-3\n  状態:: 待ち";
        let lines = raw.lines().collect::<Vec<_>>();
        let item = parse_item(&lines);

        assert_eq!(item.status, Some(Status::待ち));
    }

    #[test]
    fn 任意フィールドをfieldsに順序保持で格納する() {
        let raw =
            "- タスク着手前に目的を再確認した ^r-0701-1\n  種別:: 良かった\n  なぜ:: 確認するため";
        let lines = raw.lines().collect::<Vec<_>>();
        let item = parse_item(&lines);

        assert_eq!(
            item.fields,
            vec![
                ("種別".to_string(), "良かった".to_string()),
                ("なぜ".to_string(), "確認するため".to_string()),
            ]
        );
    }

    #[test]
    fn 状態はfieldsに含めない() {
        let raw = "- x ^t-1\n  状態:: 待ち\n  種別:: 良かった";
        let lines = raw.lines().collect::<Vec<_>>();
        let item = parse_item(&lines);

        assert_eq!(item.status, Some(Status::待ち));
        assert_eq!(
            item.fields,
            vec![("種別".to_string(), "良かった".to_string())]
        );
    }

    #[test]
    fn 複数のアイテムを分割してそれぞれパースする() {
        let raw = "- ライブラリ調査 ^t-1\n  状態:: 着手中\n- 設定修正 ^t-2\n  状態:: 未着手";
        let lines = raw.lines().collect::<Vec<_>>();
        let items = parse_items(&lines);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "t-1");
        assert_eq!(items[0].status, Some(Status::着手中));
        assert_eq!(items[1].id, "t-2");
        assert_eq!(items[1].status, Some(Status::未着手));
    }

    #[test]
    fn フィールドが複数行あってもアイテムは1つ生成される() {
        let raw = "- タスク ^t-1\n  状態:: 着手中\n  ゴール:: 確認する";
        let lines = raw.lines().collect::<Vec<_>>();
        let items = parse_items(&lines);

        assert_eq!(items.len(), 1);
    }

    #[test]
    fn フィールドが無い場合でもアイテムは1つ生成される() {
        let raw = "- タスク ^t-1";
        let lines = raw.lines().collect::<Vec<_>>();
        let items = parse_items(&lines);

        assert_eq!(items.len(), 1);
    }
}
