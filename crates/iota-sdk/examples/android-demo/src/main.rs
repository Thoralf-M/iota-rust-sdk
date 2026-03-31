// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use freya::prelude::{LaunchConfig, WindowConfig, launch};

mod app;

fn main() {
    env_logger::init();
    launch(
        LaunchConfig::new().with_window(
            WindowConfig::new(app::app)
                .with_title("IOTA PTB Viewer")
                .with_size(420., 800.)
                .with_resizable(true),
        ),
    )
}
