// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Main app component and header bar.

mod data;
mod network;
mod views;

use data::{TimelineRow, TxRow, TxStore};
use freya::prelude::*;
use network::{NETWORKS, client_for_network, fetch_transactions, tokio_runtime};
use views::{detail_view, row_txs_view, timeline_view};

pub fn app() -> impl IntoElement {
    use_init_theme(|| DARK_THEME);

    let mut store = use_state(TxStore::default);
    let mut loading = use_state(|| true);
    let selected_tx = use_state::<Option<TxRow>>(|| None);
    let selected_row = use_state::<Option<TimelineRow>>(|| None);
    let paused = use_state(|| false);
    let copy_msg = use_state(String::new);
    let network_idx = use_state(|| 0usize);

    use_hook(move || {
        spawn_forever(async move {
            let mut current_net = *network_idx.read();
            let mut client = client_for_network(current_net);
            loop {
                let net = *network_idx.read();
                if net != current_net {
                    current_net = net;
                    client = client_for_network(current_net);
                    *store.write() = TxStore::default();
                    log::info!("Switched to {}", NETWORKS[current_net]);
                }

                if !*paused.read() {
                    loading.set(true);
                    match fetch_transactions(client.clone()).await {
                        Ok(txs) if !txs.is_empty() => {
                            log::info!("Fetched {} PTBs", txs.len());
                            store.write().append(txs);
                        }
                        Ok(_) => log::warn!("Fetched 0 PTBs"),
                        Err(e) => log::error!("Fetch join error: {e}"),
                    }
                    loading.set(false);
                }
                let _ = tokio_runtime()
                    .spawn(async {
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    })
                    .await;
            }
        });
    });

    let total = store.read().txs.len();
    let is_loading = *loading.read();
    let is_paused = *paused.read();
    let cur_network = *network_idx.read();

    let content: Element = if let Some(tx) = selected_tx.read().clone() {
        let siblings = selected_row
            .read()
            .as_ref()
            .map(|r| r.txs.clone())
            .unwrap_or_default();
        detail_view(tx, selected_tx, copy_msg, siblings)
    } else if let Some(row) = selected_row.read().clone() {
        row_txs_view(row, selected_row, selected_tx)
    } else {
        timeline_view(store, selected_row, is_loading)
    };

    // Copy feedback banner.
    let banner = {
        let msg = copy_msg.read().clone();
        if msg.is_empty() {
            rect().into_element()
        } else {
            rect()
                .width(Size::fill())
                .padding((6., 12.))
                .background((40, 120, 60))
                .center()
                .font_size(12.)
                .color((255, 255, 255))
                .child(msg)
                .into_element()
        }
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .content(Content::flex())
        .background((18, 18, 24))
        .child(header_bar(
            total,
            is_loading,
            is_paused,
            paused,
            cur_network,
            network_idx,
        ))
        .child(banner)
        .child(
            rect()
                .width(Size::fill())
                .height(Size::flex(1.))
                .child(content),
        )
}

fn header_bar(
    total: usize,
    loading: bool,
    is_paused: bool,
    mut paused: State<bool>,
    cur_network: usize,
    mut network_idx: State<usize>,
) -> Element {
    let status_text = if is_paused {
        "Paused"
    } else if loading {
        "Loading..."
    } else {
        "Live"
    };
    let status_color: Color = if is_paused {
        (180, 80, 80).into()
    } else if loading {
        (255, 200, 50).into()
    } else {
        (80, 220, 100).into()
    };
    let pause_label = if is_paused { "Resume" } else { "Pause" };

    rect()
        .width(Size::fill())
        .padding((12., 16., 8., 16.))
        .background((26, 26, 36))
        .child(
            rect()
                .horizontal()
                .width(Size::fill())
                .main_align(Alignment::space_between())
                .cross_align(Alignment::center())
                .child(
                    rect()
                        .spacing(2.)
                        .color((255, 255, 255))
                        .child(
                            rect()
                                .font_size(18.)
                                .font_weight(FontWeight::BOLD)
                                .child("IOTA PTB Visualizer"),
                        )
                        .child(Select::new().selected_item(NETWORKS[cur_network]).children(
                            NETWORKS.iter().enumerate().map(|(i, name)| {
                                MenuItem::new()
                                    .selected(cur_network == i)
                                    .on_press(move |_| network_idx.set(i))
                                    .child(name.to_string())
                                    .into()
                            }),
                        )),
                )
                .child(
                    rect()
                        .horizontal()
                        .spacing(8.)
                        .cross_align(Alignment::center())
                        .child(
                            Button::new()
                                .on_press(move |_| paused.set(!is_paused))
                                .child(pause_label),
                        )
                        .child(
                            rect()
                                .font_size(13.)
                                .color((180, 180, 200))
                                .child(format!("({total})")),
                        )
                        .child(
                            rect()
                                .width(Size::px(8.))
                                .height(Size::px(8.))
                                .corner_radius(4.)
                                .background(status_color),
                        )
                        .child(rect().font_size(12.).color(status_color).child(status_text)),
                ),
        )
        .into_element()
}
