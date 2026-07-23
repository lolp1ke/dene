// SPDX-License-Identifier: Apache-2.0

use std::{fs::OpenOptions, sync::Mutex};

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, Layer, filter::filter_fn};

const LOG_FIFO: &str = "/tmp/dene_tui_log_pipe";

pub(crate) fn init_tracing() {
  use tracing_subscriber::layer::SubscriberExt as _;
  use tracing_subscriber::util::SubscriberInitExt as _;

  let Ok(pty_pipe) = OpenOptions::new().read(true).write(true).open(LOG_FIFO)
  else {
    #[cfg(debug_assertions)]
    eprintln!("run `mkfifo {}` to enable logs", LOG_FIFO);
    return;
  };

  let env_filter = EnvFilter::builder()
    .with_default_directive(if cfg!(debug_assertions) {
      LevelFilter::TRACE.into()
    } else {
      LevelFilter::WARN.into()
    })
    .from_env_lossy();
  let layer = tracing_subscriber::fmt::layer()
    .event_format(tracing_subscriber::fmt::format().pretty())
    .with_writer(Mutex::new(pty_pipe))
    .with_filter(env_filter)
    // .with_filter(filter_fn(|metadata| metadata.target().starts_with("dene")))
    ;
  tracing_subscriber::registry().with(layer).init();
}
