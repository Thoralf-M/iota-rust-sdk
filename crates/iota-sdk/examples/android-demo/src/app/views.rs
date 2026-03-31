// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! UI views: timeline, row transaction list, transaction detail.

use freya::prelude::*;

use super::{
    data::{FnCallEntry, TimelineRow, TxRow, TxStore, build_timeline, truncate_str},
    network::tokio_runtime,
};

// ---------------------------------------------------------------------------
// Timeline view
// ---------------------------------------------------------------------------

pub fn timeline_view(
    store: State<TxStore>,
    selected_row: State<Option<TimelineRow>>,
    _loading: bool,
) -> Element {
    let s = store.read();
    if s.txs.is_empty() {
        return rect()
            .expanded()
            .center()
            .font_size(16.)
            .color((140, 140, 160))
            .child("Fetching transactions...")
            .into_element();
    }

    let (rows, fn_entries) = build_timeline(&s);
    drop(s);

    let mut children: Vec<Element> = Vec::new();

    // Column header.
    children.push(
        rect()
            .horizontal()
            .width(Size::fill())
            .padding((8., 12., 4., 12.))
            .child(
                rect()
                    .width(Size::px(150.))
                    .font_size(11.)
                    .font_weight(FontWeight::BOLD)
                    .color((140, 140, 160))
                    .child("Sender / fn call"),
            )
            .child(
                rect()
                    .width(Size::flex(1.))
                    .font_size(11.)
                    .font_weight(FontWeight::BOLD)
                    .color((140, 140, 160))
                    .child("Timeline"),
            )
            .into_element(),
    );

    for row in &rows {
        children.push(timeline_row(row, selected_row));
    }

    // Separator.
    children.push(
        rect()
            .width(Size::fill())
            .height(Size::px(1.))
            .background((50, 50, 70))
            .into_element(),
    );

    // Move Function Calls section.
    children.push(
        rect()
            .width(Size::fill())
            .padding((16., 12., 8., 12.))
            .font_size(16.)
            .font_weight(FontWeight::BOLD)
            .color((255, 255, 255))
            .child("Move Function Calls")
            .into_element(),
    );

    children.push(
        rect()
            .horizontal()
            .width(Size::fill())
            .padding((4., 12.))
            .background((30, 30, 42))
            .child(
                rect()
                    .width(Size::px(50.))
                    .font_size(11.)
                    .font_weight(FontWeight::BOLD)
                    .color((140, 140, 160))
                    .child("Count"),
            )
            .child(
                rect()
                    .width(Size::flex(1.))
                    .font_size(11.)
                    .font_weight(FontWeight::BOLD)
                    .color((140, 140, 160))
                    .child("Function"),
            )
            .into_element(),
    );

    for entry in fn_entries.iter().take(20) {
        children.push(fn_call_row(entry));
    }

    ScrollView::new()
        .width(Size::fill())
        .height(Size::fill())
        .child(
            rect()
                .width(Size::fill())
                .padding((0., 0., 16., 0.))
                .children(children),
        )
        .into_element()
}

fn timeline_row(row: &TimelineRow, mut selected_row: State<Option<TimelineRow>>) -> Element {
    let label = row.label.clone();
    let sublabel = row.sublabel.clone();
    let count = row.count;
    let mut positions = row.positions.clone();
    positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let row_snapshot = row.clone();

    rect()
        .width(Size::fill())
        .padding((2., 12., 2., 12.))
        .horizontal()
        .cross_align(Alignment::center())
        .on_press(move |_| {
            selected_row.set(Some(row_snapshot.clone()));
        })
        .child(
            rect()
                .width(Size::px(150.))
                .spacing(1.)
                .child(
                    rect()
                        .horizontal()
                        .spacing(6.)
                        .cross_align(Alignment::center())
                        .child(rect().font_size(12.).color((100, 180, 255)).child(label))
                        .child(
                            rect()
                                .font_size(10.)
                                .color((140, 140, 160))
                                .child(format!("({count})")),
                        ),
                )
                .child(if let Some(sub) = sublabel {
                    rect()
                        .font_size(10.)
                        .color((120, 120, 140))
                        .child(sub)
                        .into_element()
                } else {
                    rect().into_element()
                }),
        )
        .child(
            rect()
                .width(Size::flex(1.))
                .height(Size::px(24.))
                .cross_align(Alignment::center())
                .child(dots_line(&positions)),
        )
        .into_element()
}

fn dots_line(positions: &[f32]) -> Element {
    if positions.is_empty() {
        return rect()
            .width(Size::fill())
            .height(Size::px(1.))
            .background((40, 40, 55))
            .into_element();
    }

    let mut children: Vec<Element> = Vec::new();
    let mut prev_pct = 0.0f32;

    for &pos in positions {
        let pct = (pos * 100.0).clamp(0.0, 100.0);
        let gap = (pct - prev_pct).max(0.0);

        if gap > 0.01 {
            children.push(
                rect()
                    .width(Size::percent(gap))
                    .height(Size::px(1.))
                    .background((40, 40, 55))
                    .into_element(),
            );
        }

        children.push(
            rect()
                .width(Size::px(6.))
                .height(Size::px(6.))
                .corner_radius(3.)
                .background((100, 180, 255))
                .into_element(),
        );

        prev_pct = pct;
    }

    let remaining = (100.0 - prev_pct).max(0.0);
    if remaining > 0.01 {
        children.push(
            rect()
                .width(Size::percent(remaining))
                .height(Size::px(1.))
                .background((40, 40, 55))
                .into_element(),
        );
    }

    rect()
        .horizontal()
        .width(Size::fill())
        .height(Size::px(24.))
        .cross_align(Alignment::center())
        .children(children)
        .into_element()
}

fn fn_call_row(entry: &FnCallEntry) -> Element {
    rect()
        .horizontal()
        .width(Size::fill())
        .padding((4., 12.))
        .child(
            rect()
                .width(Size::px(50.))
                .font_size(12.)
                .color((100, 180, 255))
                .child(format!("{}", entry.count)),
        )
        .child(
            rect()
                .width(Size::flex(1.))
                .font_size(12.)
                .color((200, 200, 220))
                .child(entry.name.clone()),
        )
        .into_element()
}

// ---------------------------------------------------------------------------
// Row transactions list
// ---------------------------------------------------------------------------

pub fn row_txs_view(
    row: TimelineRow,
    mut selected_row: State<Option<TimelineRow>>,
    mut selected_tx: State<Option<TxRow>>,
) -> Element {
    let tx_cards: Vec<Element> = row
        .txs
        .iter()
        .enumerate()
        .map(|(i, tx)| {
            let digest = truncate_str(&tx.digest, 8, 6);
            let sender = truncate_str(&tx.sender, 8, 6);
            let summary = tx.summary.clone();
            let calls = if tx.move_calls.is_empty() {
                "No Move calls".to_string()
            } else {
                tx.move_calls.join(", ")
            };
            let tx_clone = tx.clone();

            rect()
                .key(i)
                .width(Size::fill())
                .padding(12.)
                .corner_radius(8.)
                .background((32, 32, 44))
                .spacing(6.)
                .on_press(move |_| {
                    selected_tx.set(Some(tx_clone.clone()));
                })
                .child(
                    rect()
                        .horizontal()
                        .width(Size::fill())
                        .main_align(Alignment::space_between())
                        .child(
                            rect()
                                .font_size(14.)
                                .font_weight(FontWeight::SEMI_BOLD)
                                .color((100, 180, 255))
                                .child(digest),
                        )
                        .child(rect().font_size(11.).color((140, 140, 160)).child(summary)),
                )
                .child(
                    rect()
                        .font_size(12.)
                        .color((180, 180, 200))
                        .child(format!("Sender: {sender}")),
                )
                .child(rect().font_size(12.).color((160, 200, 140)).child(calls))
                .into_element()
        })
        .collect();

    let title = format!("{} ({} txs)", row.label, row.count);

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .content(Content::flex())
        .child(
            rect()
                .width(Size::fill())
                .padding(12.)
                .background((26, 26, 36))
                .horizontal()
                .cross_align(Alignment::center())
                .spacing(12.)
                .child(
                    Button::new()
                        .on_press(move |_| selected_row.set(None))
                        .child("Back"),
                )
                .child(
                    rect()
                        .font_size(16.)
                        .font_weight(FontWeight::BOLD)
                        .color((255, 255, 255))
                        .child(title),
                ),
        )
        .child(
            ScrollView::new()
                .width(Size::fill())
                .height(Size::flex(1.))
                .child(
                    rect()
                        .width(Size::fill())
                        .padding((8., 8.))
                        .spacing(4.)
                        .children(tx_cards),
                ),
        )
        .into_element()
}

// ---------------------------------------------------------------------------
// Detail view
// ---------------------------------------------------------------------------

pub fn detail_view(
    tx: TxRow,
    mut selected_tx: State<Option<TxRow>>,
    copy_msg: State<String>,
    siblings: Vec<TxRow>,
) -> Element {
    let move_calls_display = if tx.move_calls.is_empty() {
        "None".to_string()
    } else {
        tx.move_calls.join("\n")
    };

    let cur_idx = siblings.iter().position(|t| t.digest == tx.digest);
    let prev_tx = cur_idx
        .and_then(|i| i.checked_sub(1))
        .and_then(|i| siblings.get(i).cloned());
    let next_tx = cur_idx.and_then(|i| siblings.get(i + 1).cloned());
    let pos_label = cur_idx.map(|i| format!("{}/{}", i + 1, siblings.len()));

    let disabled_btn = |label: &str| -> Element {
        rect()
            .padding((6., 10.))
            .corner_radius(6.)
            .background((40, 40, 50))
            .font_size(14.)
            .color((80, 80, 90))
            .child(label)
            .into_element()
    };

    let prev_btn: Element = if let Some(ptx) = prev_tx {
        Button::new()
            .on_press(move |_| selected_tx.set(Some(ptx.clone())))
            .child("< Prev")
            .into_element()
    } else {
        disabled_btn("< Prev")
    };

    let next_btn: Element = if let Some(ntx) = next_tx {
        Button::new()
            .on_press(move |_| selected_tx.set(Some(ntx.clone())))
            .child("Next >")
            .into_element()
    } else {
        disabled_btn("Next >")
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .content(Content::flex())
        // Top bar: Back + title.
        .child(
            rect()
                .width(Size::fill())
                .padding((12., 12., 4., 12.))
                .background((26, 26, 36))
                .horizontal()
                .cross_align(Alignment::center())
                .spacing(12.)
                .child(
                    Button::new()
                        .on_press(move |_| selected_tx.set(None))
                        .child("Back"),
                )
                .child(
                    rect()
                        .font_size(16.)
                        .font_weight(FontWeight::BOLD)
                        .color((255, 255, 255))
                        .child("Transaction Details"),
                ),
        )
        // Navigation bar: Prev / position / Next.
        .child(
            rect()
                .width(Size::fill())
                .padding((4., 12., 8., 12.))
                .background((26, 26, 36))
                .horizontal()
                .main_align(Alignment::center())
                .cross_align(Alignment::center())
                .spacing(12.)
                .child(prev_btn)
                .child(if let Some(label) = pos_label {
                    rect()
                        .font_size(13.)
                        .color((180, 180, 200))
                        .child(label)
                        .into_element()
                } else {
                    rect().into_element()
                })
                .child(next_btn),
        )
        .child(
            ScrollView::new()
                .width(Size::fill())
                .height(Size::flex(1.))
                .child(
                    rect()
                        .width(Size::fill())
                        .padding(16.)
                        .spacing(16.)
                        .child(detail_section("Digest", &tx.digest, copy_msg))
                        .child(detail_section("Sender", &tx.sender, copy_msg))
                        .child(detail_section("Move Calls", &move_calls_display, copy_msg))
                        .child(detail_section("Commands", &tx.summary, copy_msg))
                        .child(detail_section("Raw JSON", &tx.json, copy_msg)),
                ),
        )
        .into_element()
}

fn flash_copy(mut copy_msg: State<String>, field: &str) {
    copy_msg.set(format!("Copied: {field}"));
    spawn(async move {
        let _ = tokio_runtime()
            .spawn(async {
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            })
            .await;
        copy_msg.set(String::new());
    });
}

fn detail_section(title: &str, value: &str, copy_msg: State<String>) -> Element {
    let value_owned = value.to_string();
    let title_owned = title.to_string();

    rect()
        .width(Size::fill())
        .padding(12.)
        .corner_radius(8.)
        .background((32, 32, 44))
        .spacing(6.)
        .on_press(move |_| {
            let _ = Clipboard::set(value_owned.clone());
            flash_copy(copy_msg, &title_owned);
        })
        .child(
            rect()
                .horizontal()
                .width(Size::fill())
                .main_align(Alignment::space_between())
                .child(
                    rect()
                        .font_size(12.)
                        .font_weight(FontWeight::BOLD)
                        .color((140, 140, 160))
                        .child(title),
                )
                .child(
                    rect()
                        .font_size(10.)
                        .color((100, 100, 120))
                        .child("Tap to copy"),
                ),
        )
        .child(rect().font_size(14.).color((220, 220, 240)).child(value))
        .into_element()
}
