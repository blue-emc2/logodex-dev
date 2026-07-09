use crate::model::{Group, Item, Lane, Status};

fn parse_lanes(line: &str) -> Vec<Lane> {
    let mut lanes = vec![];
    let mut lane_block = vec![];

    for l in line.lines() {
        if l.starts_with("##") && !l.starts_with("### ") {
            if !lane_block.is_empty() {
                lanes.push(parse_lane(lane_block.join("\n").as_str()));
            }
            lane_block.clear();
            lane_block.push(l);
        } else {
            lane_block.push(l);
        }
    }
    if !lane_block.is_empty() {
        lanes.push(parse_lane(lane_block.join("\n").as_str()));
    }

    lanes
}

fn parse_lane(line: &str) -> Lane {
    let mut title = "hoge";
    let mut group_block = vec![];

    for line in line.lines() {
        if line.starts_with("## ") {
            let body = line.strip_prefix("##");
            title = body.unwrap_or("unknown").trim();
        } else {
            group_block.push(line);
        }
    }

    let groups = parse_groups(group_block.join("\n").as_str());

    Lane {
        title: title.to_string(),
        groups,
    }
}

fn parse_groups(line: &str) -> Vec<Group> {
    let mut groups = vec![];
    let mut group_block = vec![];

    for l in line.lines() {
        if l.starts_with("###") {
            if !group_block.is_empty() {
                groups.push(parse_group(group_block.join("\n").as_str()));
            }
            group_block.clear();
            group_block.push(l);
        } else {
            group_block.push(l);
        }
    }
    if !group_block.is_empty() {
        groups.push(parse_group(group_block.join("\n").as_str()));
    }

    groups
}

fn parse_group(line: &str) -> Group {
    let mut heading = "";
    let mut item_block: Vec<&str> = vec![];
    for line in line.lines() {
        if line.starts_with("### ") {
            let body = line.strip_prefix("###");
            heading = body.unwrap_or("unknown").trim();
        } else {
            item_block.push(line);
        }
    }

    let items = parse_items(item_block.join("\n").as_str());

    Group {
        heading: heading.to_string(),
        items,
    }
}

fn parse_items(line: &str) -> Vec<Item> {
    let mut items = vec![];
    let mut block = vec![];
    for l in line.lines() {
        if l.starts_with("- ") {
            if !block.is_empty() {
                // 次ブロックを処理する前に溜めていたブロックをパースする
                items.push(parse_item(block.join("\n").as_str()));
            }
            block.clear();
            block.push(l);
        } else {
            block.push(l);
        }
    }
    if !block.is_empty() {
        items.push(parse_item(block.join("\n").as_str()));
    }
    items
}

fn parse_item(line: &str) -> Item {
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    let title = tokens[1];
    let id = tokens[2];
    let status = line
        .lines()
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
    for l in line.lines().skip(1) {
        let l = l.trim();
        if let Some((k, v)) = l.split_once("::") {
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
    fn 複数のレーンをパースしてレーンとグループの組み合わせができること() {
        let raw = "## 仕事管理\n### mugenup\n- 反社チェック確認 ^t-1\n  状態:: 待ち\n## 人間管理\n### 振り返り・気付き\n- 定例会で報告できた ^r-1\n  種別:: 良かった";
        let lanes = parse_lanes(raw);

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
        let lane = parse_lane(raw);

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
        let groups = parse_groups(raw);

        assert_eq!(groups[0].heading, "test_group1");
        assert_eq!(groups[0].items.len(), 1);
        assert_eq!(groups[1].heading, "test_group2");
        assert_eq!(groups[1].items.len(), 1);
    }

    #[test]
    fn 単一のグループとアイテムをパースしてグループ1つとアイテム1つを取り出す() {
        let raw = "### test_group1\n- 資料確認 ^t-1\n  状態:: 待ち";
        let group = parse_group(raw);

        assert_eq!(group.heading, "test_group1");
        assert_eq!(group.items.len(), 1);
    }

    #[test]
    fn 単一のアイテム行からタイトルとidを取り出す() {
        let item = parse_item("- ライブラリ調査 ^t-0701-1");

        assert_eq!(item.title, "ライブラリ調査");
        assert_eq!(item.id, "t-0701-1");
    }

    #[test]
    fn 状態付きのアイテムから状態を取り出す() {
        let item = parse_item("- 資料確認 ^t-0701-3\n  状態:: 待ち");

        assert_eq!(item.status, Some(Status::待ち));
    }

    #[test]
    fn 任意フィールドをfieldsに順序保持で格納する() {
        let item = parse_item(
            "- タスク着手前に目的を再確認した ^r-0701-1\n  種別:: 良かった\n  なぜ:: 確認するため",
        );

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
        let item = parse_item("- x ^t-1\n  状態:: 待ち\n  種別:: 良かった");

        assert_eq!(item.status, Some(Status::待ち));
        assert_eq!(
            item.fields,
            vec![("種別".to_string(), "良かった".to_string())]
        );
    }

    #[test]
    fn 複数のアイテムを分割してそれぞれパースする() {
        let items =
            parse_items("- ライブラリ調査 ^t-1\n  状態:: 着手中\n- 設定修正 ^t-2\n  状態:: 未着手");

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "t-1");
        assert_eq!(items[0].status, Some(Status::着手中));
        assert_eq!(items[1].id, "t-2");
        assert_eq!(items[1].status, Some(Status::未着手));
    }

    #[test]
    fn フィールドが複数行あってもアイテムは1つ生成される() {
        let items = parse_items("- タスク ^t-1\n  状態:: 着手中\n  ゴール:: 確認する");

        assert_eq!(items.len(), 1);
    }

    #[test]
    fn フィールドが無い場合でもアイテムは1つ生成される() {
        let items = parse_items("- タスク ^t-1");

        assert_eq!(items.len(), 1);
    }
}
