// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Data types, helpers, and timeline grouping logic.

use std::collections::{BTreeMap, HashSet};

use iota_sdk::types::{Command, ProgrammableTransaction, Transaction, TransactionKind};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn truncate_str(s: &str, prefix: usize, suffix: usize) -> String {
    if s.len() <= prefix + suffix + 3 {
        return s.to_string();
    }
    format!("{}...{}", &s[..prefix], &s[s.len() - suffix..])
}

pub fn extract_move_calls(tx: &Transaction) -> Vec<String> {
    let v1 = tx.as_v1();
    let TransactionKind::ProgrammableTransaction(ptb) = &v1.kind else {
        return Vec::new();
    };
    ptb.commands
        .iter()
        .filter_map(|cmd| {
            if let Command::MoveCall(mc) = cmd {
                Some(format!("{}::{}", mc.module, mc.function))
            } else {
                None
            }
        })
        .collect()
}

pub fn command_summary(ptb: &ProgrammableTransaction) -> String {
    let mut move_calls = 0u32;
    let mut transfers = 0u32;
    let mut splits = 0u32;
    let mut merges = 0u32;
    let mut other = 0u32;
    for cmd in &ptb.commands {
        match cmd {
            Command::MoveCall(_) => move_calls += 1,
            Command::TransferObjects(_) => transfers += 1,
            Command::SplitCoins(_) => splits += 1,
            Command::MergeCoins(_) => merges += 1,
            _ => other += 1,
        }
    }
    let mut parts = Vec::new();
    if move_calls > 0 {
        parts.push(format!("{move_calls} MoveCall"));
    }
    if transfers > 0 {
        parts.push(format!("{transfers} Transfer"));
    }
    if splits > 0 {
        parts.push(format!("{splits} Split"));
    }
    if merges > 0 {
        parts.push(format!("{merges} Merge"));
    }
    if other > 0 {
        parts.push(format!("{other} Other"));
    }
    parts.join(", ")
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct TxRow {
    pub timestamp_ms: i64,
    pub digest: String,
    pub sender: String,
    pub move_calls: Vec<String>,
    pub summary: String,
    pub json: String,
}

impl TxRow {
    pub fn first_call(&self) -> Option<&str> {
        self.move_calls.first().map(|s| s.as_str())
    }
}

/// Accumulated state across polls.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TxStore {
    pub txs: Vec<TxRow>,
    seen: HashSet<String>,
}

impl TxStore {
    pub fn append(&mut self, new: Vec<TxRow>) {
        for row in new {
            if self.seen.insert(row.digest.clone()) {
                self.txs.push(row);
            }
        }
        self.txs.sort_by_key(|t| t.timestamp_ms);
        if self.txs.len() > 500 {
            let drain = self.txs.len() - 500;
            for tx in self.txs.drain(..drain) {
                self.seen.remove(&tx.digest);
            }
        }
    }
}

/// A row in the timeline visualization.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineRow {
    pub label: String,
    pub sublabel: Option<String>,
    pub count: usize,
    pub positions: Vec<f32>,
    pub txs: Vec<TxRow>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FnCallEntry {
    pub name: String,
    pub count: usize,
}

// ---------------------------------------------------------------------------
// Timeline grouping
// ---------------------------------------------------------------------------

pub fn build_timeline(store: &TxStore) -> (Vec<TimelineRow>, Vec<FnCallEntry>) {
    if store.txs.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let ts_min = store.txs.first().unwrap().timestamp_ms;
    let ts_max = store.txs.last().unwrap().timestamp_ms;
    let range = (ts_max - ts_min).max(1) as f64;
    let normalize = |ts: i64| -> f32 { ((ts - ts_min) as f64 / range) as f32 };

    // Group by sender.
    let mut by_sender: BTreeMap<String, Vec<&TxRow>> = BTreeMap::new();
    for tx in &store.txs {
        by_sender.entry(tx.sender.clone()).or_default().push(tx);
    }

    let mut rows: Vec<TimelineRow> = Vec::new();
    let mut single_txs: Vec<&TxRow> = Vec::new();

    for (sender, txs) in &by_sender {
        if txs.len() >= 2 {
            let mut call_counts: BTreeMap<&str, usize> = BTreeMap::new();
            for tx in txs {
                for c in &tx.move_calls {
                    *call_counts.entry(c.as_str()).or_default() += 1;
                }
            }
            let top_call = call_counts
                .into_iter()
                .max_by_key(|(_, c)| *c)
                .map(|(name, count)| format!("{name} ({count})"));

            rows.push(TimelineRow {
                label: truncate_str(sender, 6, 4),
                sublabel: top_call,
                count: txs.len(),
                positions: txs.iter().map(|tx| normalize(tx.timestamp_ms)).collect(),
                txs: txs.iter().map(|tx| (*tx).clone()).collect(),
            });
        } else {
            single_txs.extend(txs);
        }
    }

    rows.sort_by(|a, b| b.count.cmp(&a.count));

    // Group single-tx senders by first Move call.
    let mut by_call: BTreeMap<String, Vec<&TxRow>> = BTreeMap::new();
    let mut other_txs: Vec<&TxRow> = Vec::new();
    for tx in &single_txs {
        if let Some(call) = tx.first_call() {
            by_call.entry(call.to_string()).or_default().push(tx);
        } else {
            other_txs.push(tx);
        }
    }

    for (call, txs) in &by_call {
        rows.push(TimelineRow {
            label: call.clone(),
            sublabel: None,
            count: txs.len(),
            positions: txs.iter().map(|tx| normalize(tx.timestamp_ms)).collect(),
            txs: txs.iter().map(|tx| (*tx).clone()).collect(),
        });
    }

    if !other_txs.is_empty() {
        rows.push(TimelineRow {
            label: "Other Txs".to_string(),
            sublabel: None,
            count: other_txs.len(),
            positions: other_txs
                .iter()
                .map(|tx| normalize(tx.timestamp_ms))
                .collect(),
            txs: other_txs.iter().map(|tx| (*tx).clone()).collect(),
        });
    }

    // Function call table.
    let mut fn_counts: BTreeMap<String, usize> = BTreeMap::new();
    for tx in &store.txs {
        for c in &tx.move_calls {
            *fn_counts.entry(c.clone()).or_default() += 1;
        }
    }
    let mut fn_entries: Vec<FnCallEntry> = fn_counts
        .into_iter()
        .map(|(name, count)| FnCallEntry { name, count })
        .collect();
    fn_entries.sort_by(|a, b| b.count.cmp(&a.count));

    (rows, fn_entries)
}
