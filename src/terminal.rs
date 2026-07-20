// SPDX-License-Identifier: Apache-2.0

use std::{
  io::{Stdout, Write as _, stdout},
  sync::OnceLock,
};

use crossterm::{cursor, event, execute, queue, style, terminal};
use parking_lot::RwLock;

pub(crate) static TERM: OnceLock<RwLock<Terminal>> = OnceLock::new();

#[inline]
pub(crate) fn get_terminal() -> &'static RwLock<Terminal> {
  TERM.get().expect("call `Terminal::new()` first")
}

#[derive(Debug)]
pub(crate) struct Terminal {
  stdout: Stdout,
  front_buffer: Vec<char>,
  back_buffer: Vec<char>,
  width: u16,
  height: u16,
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
    let (width, height) = Self::size();
    let buf_len = width as usize * height as usize;
    let front_buffer = vec![' '; buf_len];
    let back_buffer = vec![' '; buf_len];

    Self {
      stdout,
      front_buffer,
      back_buffer,
      width,
      height,
    }
  }

  pub(crate) fn clear(&mut self) {
    self.back_buffer.fill(' ');
  }
  pub(crate) fn render(&mut self) {
    let w = self.width as usize;
    let total = self.back_buffer.len();
    let mut i = 0;
    while i < total {
      if self.back_buffer[i] == self.front_buffer[i] {
        i += 1;
        continue;
      };
      let start = (i % w) as u16;
      let y = (i / w) as u16;
      let mut buf = String::with_capacity(8);
      while i < total && self.back_buffer[i] != self.front_buffer[i] {
        buf.push(self.back_buffer[i]);
        i += 1;
        if i % w == 0 {
          break;
        };
      }
      _ = queue!(self.stdout, cursor::MoveTo(start, y));
      _ = queue!(self.stdout, style::Print(&buf));
    }

    _ = self.stdout.flush();
    std::mem::swap(&mut self.front_buffer, &mut self.back_buffer);
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
  pub(crate) fn size() -> (u16, u16) {
    terminal::size().unwrap_or((0, 0))
  }

  pub(crate) fn write_at(&mut self, x: u16, y: u16, buf: &str) {
    let w = self.width as usize;
    let h = self.height as usize;
    let col = x as usize;
    let row = y as usize;
    if col >= w || row >= h {
      return;
    };

    let start = row * w + col;
    for (i, ch) in buf.chars().enumerate() {
      let idx = start + i;
      if idx > self.back_buffer.len() || (idx % w) < col {
        break;
      };
      self.back_buffer[idx] = ch;
    }
  }
}
