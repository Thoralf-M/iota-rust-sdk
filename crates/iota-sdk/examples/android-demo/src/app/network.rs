// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Network selection, Tokio runtime bridge, and transaction fetching.

use std::sync::OnceLock;

use base64ct::Encoding;
use iota_sdk::{graphql_client::Client, types::TransactionKind};

use super::data::{TxRow, command_summary, extract_move_calls};

pub const NETWORKS: [&str; 3] = ["Mainnet", "Testnet", "Devnet"];

pub fn client_for_network(idx: usize) -> Client {
    match idx {
        1 => Client::new_testnet(),
        2 => Client::new_devnet(),
        _ => Client::new_mainnet(),
    }
}

pub fn tokio_runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create Tokio runtime")
    })
}

const TX_QUERY: &str = r#"
query ($last: Int) {
  transactionBlocks(last: $last, filter: { kind: PROGRAMMABLE_TX }) {
    nodes {
      bcs
      effects {
        timestamp
      }
    }
  }
}
"#;

pub fn fetch_transactions(client: Client) -> tokio::task::JoinHandle<Vec<TxRow>> {
    tokio_runtime().spawn(async move {
        let query = serde_json::json!({
            "query": TX_QUERY,
            "variables": { "last": 50 }
        });
        let map = match query.as_object() {
            Some(m) => m.clone(),
            None => return Vec::new(),
        };

        let response = match client.run_query_from_json(map).await {
            Ok(r) => r,
            Err(e) => {
                log::error!("GraphQL query failed: {e}");
                return Vec::new();
            }
        };

        let Some(data) = response.data else {
            log::error!("No data in GraphQL response");
            return Vec::new();
        };

        let nodes = match data
            .get("transactionBlocks")
            .and_then(|tb| tb.get("nodes"))
            .and_then(|n| n.as_array())
        {
            Some(n) => n,
            None => {
                log::error!("Unexpected response shape");
                return Vec::new();
            }
        };

        let mut result = Vec::new();
        for node in nodes {
            let Some(bcs_b64) = node.get("bcs").and_then(|v| v.as_str()) else {
                continue;
            };
            let timestamp_str = node
                .get("effects")
                .and_then(|e| e.get("timestamp"))
                .and_then(|t| t.as_str())
                .unwrap_or("");

            let timestamp_ms = chrono::DateTime::parse_from_rfc3339(timestamp_str)
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0);

            let Ok(bcs_bytes) = base64ct::Base64::decode_vec(bcs_b64) else {
                continue;
            };
            let Ok(sender_signed) =
                bcs::from_bytes::<iota_sdk::types::SenderSignedTransaction>(&bcs_bytes)
            else {
                continue;
            };
            let stx = &sender_signed.0;

            let v1 = stx.transaction.as_v1();
            let digest = format!("{}", stx.transaction.digest());
            let sender = format!("{}", v1.sender);
            let move_calls = extract_move_calls(&stx.transaction);
            let summary = if let TransactionKind::ProgrammableTransaction(ptb) = &v1.kind {
                command_summary(ptb)
            } else {
                "N/A".to_string()
            };
            let json = serde_json::to_string_pretty(stx).unwrap_or_default();

            result.push(TxRow {
                timestamp_ms,
                digest,
                sender,
                move_calls,
                summary,
                json,
            });
        }

        result
    })
}
