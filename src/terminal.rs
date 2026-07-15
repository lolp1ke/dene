// SPDX-License-Identifier: Apache-2.0

use std::{
  io::{Stdout, stdout},
  sync::OnceLock,
};

use crossterm::{cursor, event, execute, terminal};
use parking_lot::RwLock;

pub(crate) static TERM: OnceLock<RwLock<Terminal>> = OnceLock::new();

#[inline]
pub(crate) fn get_terminal() -> &'static RwLock<Terminal> {
  TERM.get().expect("call `Terminal::new()` first")
}

#[derive(Debug)]
pub(crate) struct Terminal {
  stdout: Stdout,
}
impl Terminal {
  pub(crate) fn new() -> Self {
    let mut stdout = stdout();
    _ = terminal::enable_raw_mode();
    _ = execute!(
      stdout,
      terminal::EnterAlternateScreen,
      cursor::Hide,
      terminal::Clear(terminal::ClearType::All)
    );
    _ = execute!(
      stdout,
      event::PushKeyboardEnhancementFlags(
        event::KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
          | event::KeyboardEnhancementFlags::REPORT_EVENT_TYPES
          | event::KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
          | event::KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
      )
    );

    Self { stdout }
  }
  pub(crate) fn restore(&mut self) {
    _ = terminal::disable_raw_mode();
    _ = execute!(
      self.stdout,
      cursor::MoveTo(0, 0),
      terminal::Clear(terminal::ClearType::All)
    );
    _ = execute!(self.stdout, cursor::Show, terminal::LeaveAlternateScreen);
  }
  pub(crate) fn size(&self) -> (u16, u16) {
    terminal::size().unwrap_or((0, 0))
  }
}
