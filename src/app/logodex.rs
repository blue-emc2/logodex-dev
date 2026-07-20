use crate::{
    app::parser,
    model::{Group, Item, Lane, Logbook, Status},
};

use gpui::{
    App, Application, Bounds, Context, FontWeight, Rgba, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};

pub struct Logodex;
struct LogodexWindow {
    log_book: Logbook,
}

impl Render for LogodexWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xeeeeee))
            .flex()
            .flex_row()
            .gap_3()
            .p_4()
            .children(self.log_book.lanes.iter().map(render_lane))
    }
}

fn get_bg_color(status: &Status) -> Rgba {
    match status {
        Status::未着手 => rgb(0xaaaaaa),
        Status::着手中 => rgb(0x5aa6f0),
        Status::待ち => rgb(0xf0a85a),
        Status::順延 => rgb(0xb79af0),
        Status::完了 => rgb(0x6cd07a),
    }
}

fn render_lane(lane: &Lane) -> impl IntoElement {
    div()
        .flex_1()
        .bg(rgb(0x2f2f33))
        .border_1()
        .border_color(rgb(0x444444))
        .rounded_md()
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .child(lane.title.clone()),
        )
        .children(lane.groups.iter().map(render_group))
}

fn render_group(group: &Group) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x99aaff))
                .child(group.heading.clone()),
        )
        .children(group.items.iter().map(render_item))
}

fn render_item(item: &Item) -> impl IntoElement {
    let row = div()
        .flex()
        .flex_row()
        .bg(rgb(0x383840))
        .rounded_md()
        .px_2()
        .py_1p5()
        .text_sm()
        .justify_between()
        .items_center()
        .child(item.title.clone());

    match &item.status {
        None => row,
        Some(s) => {
            let t = match s {
                Status::未着手 => "未着手",
                Status::着手中 => "着手中",
                Status::待ち => "待ち",
                Status::順延 => "順延",
                Status::完了 => "完了",
            };

            row.child(
                div()
                    .bg(get_bg_color(s))
                    .text_color(rgb(0x111111))
                    .text_xs()
                    .px_2()
                    .py_0p5()
                    .rounded_full()
                    .child(t),
            )
        }
    }
}

fn mock_raw() -> &'static str {
    r#"---
    date: 2026-06-26
    type: logbook
    mood: 天気のせいか足が痛む
    ---

    ## 仕事管理

    ### mugenup
    - 反社チェック確認 ^t-abc123
      状態:: 待ち
      ゴール:: チェック状況を確認し、未実施なら実施
    - 定例会で REDIS 調査結果を報告 ^t-def456
      状態:: 完了
      完了:: 10:37
    - 精算バッチのエラー調査 ^t-jkl012
      状態:: 着手中

    ### 社内
    - 精算処理系を追加 ^t-ghi789
      状態:: 完了
      完了:: 15:20
    - 経費精算を出す ^t-mno345
      状態:: 未着手
    - 勉強会の日程調整 ^t-pqr678
      状態:: 順延
      メモ:: 来週に持ち越し

    ## 人間管理

    ### 振り返り・気付き
    - 定例会で結果を伝えられた ^r-001
      種別:: 良かった
      なぜ:: 事前に練習した
      習慣化:: 事前準備
    - タスクを詰め込みすぎた ^r-002
      種別:: 改善したい

    ### 気分・体調
    - 足が痛む（天候由来）。無理せず ^m-001

    ### やりたいこと
    - 『いい質問が人を動かす』読書感想 ^w-001
      状態:: 未着手
    "#
}

impl Logodex {
    pub fn run() {
        Application::new().run(|cx: &mut App| {
            let bounds = Bounds::centered(None, size(px(800.0), px(640.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|_| LogodexWindow {
                        log_book: Self::parse_logbook(),
                    })
                },
            )
            .unwrap();
            cx.activate(true);
        });
    }

    fn parse_logbook() -> Logbook {
        let dummy_text = mock_raw();
        parser::parse(dummy_text)
    }
}
