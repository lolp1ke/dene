// SPDX-License-Identifier: Apache-2.0

use std::{
  io::{Stdout, Write as _, stdout},
  sync::{Arc, OnceLock},
};

use crossterm::{cursor, event, execute, queue, style, terminal};
use parking_lot::RwLock;

pub(crate) static TERM: OnceLock<RwLock<Terminal>> = OnceLock::new();

#[inline]
pub(crate) fn get_terminal() -> &'static RwLock<Terminal> {
  TERM.get().expect("call `Terminal::new()` first")
}

#[derive(Debug)]
pub(crate) struct AnsiOverlay {
  x: u16,
  y: u16,
  ansi: Arc<str>,
  text: Arc<str>,
}

#[derive(Debug)]
pub(crate) struct Terminal {
  pub(crate) stdout: Stdout,

  front: Buffer,
  back: Buffer,

  ansi_overlays: Vec<AnsiOverlay>,
  prev_ansi_overlays: Vec<AnsiOverlay>,
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

    Self {
      stdout,
      front: Buffer::new(buf_len),
      back: Buffer::new(buf_len),
      ansi_overlays: Vec::new(),
      prev_ansi_overlays: Vec::new(),
      width,
      height,
    }
  }

  pub(crate) fn clear(&mut self) {
    self.back.clear();
    self.ansi_overlays.clear();
  }
  pub(crate) fn render(&mut self) {
    for overlay in self.prev_ansi_overlays.iter() {
      _ = queue!(self.stdout, cursor::MoveTo(overlay.x, overlay.y));
      _ = queue!(self.stdout, style::Print(&*overlay.text));
    }

    let w = self.width as usize;
    let total = self.back.cells.len();
    let mut i = 0;
    let mut cur_fg = Color::Reset;
    let mut cur_bg = Color::Reset;
    let mut changed = false;

    while i < total {
      if self.back.cells[i] == self.front.cells[i] {
        i += 1;
        continue;
      }

      changed = true;
      let x = (i % w) as u16;
      let y = (i / w) as u16;
      let run_start = i;
      let mut text = String::with_capacity(8);

      while i < total && self.back.cells[i] != self.front.cells[i] {
        text.push(self.back.cells[i].ch);
        i += 1;
        if i % w == 0 {
          break;
        };
      }

      let cell = &self.back.cells[run_start];

      _ = queue!(self.stdout, cursor::MoveTo(x, y));

      if cell.fg != cur_fg {
        _ = queue!(self.stdout, style::SetForegroundColor(cell.fg.into()));
        cur_fg = cell.fg;
      };
      if cell.bg != cur_bg {
        _ = queue!(self.stdout, style::SetBackgroundColor(cell.bg.into()));
        cur_bg = cell.bg;
      };

      _ = queue!(self.stdout, style::Print(&text));
    }
    if changed {
      _ = queue!(self.stdout, style::ResetColor);
      _ = queue!(self.stdout, style::SetAttribute(style::Attribute::Reset));
    };

    for overlay in self.ansi_overlays.iter() {
      _ = queue!(self.stdout, cursor::MoveTo(overlay.x, overlay.y));
      _ = queue!(self.stdout, style::Print(&*overlay.ansi));
    }

    _ = self.stdout.flush();
    std::mem::swap(&mut self.front, &mut self.back);
    std::mem::swap(&mut self.ansi_overlays, &mut self.prev_ansi_overlays);
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

  pub(crate) fn write_at<S>(&mut self, x: u16, y: u16, buf: S)
  where
    S: AsRef<str>,
  {
    self.back.write_chars(x, y, buf.as_ref(), self.width);
  }
  pub(crate) fn write_ansi_at(
    &mut self,
    x: u16,
    y: u16,
    ansi: &str,
    text: &str,
  ) {
    self.ansi_overlays.push(AnsiOverlay {
      x,
      y,
      ansi: ansi.into(),
      text: text.into(),
    });
  }
}

#[derive(Debug)]
struct Buffer {
  cells: Vec<Cell>,
}
impl Buffer {
  fn new(len: usize) -> Self {
    Self {
      cells: vec![
        Cell {
          ch: ' ',
          fg: Color::Reset,
          bg: Color::Reset,
        };
        len
      ],
    }
  }

  fn clear(&mut self) {
    for cell in self.cells.iter_mut() {
      cell.ch = ' ';
      cell.fg = Color::Reset;
      cell.bg = Color::Reset;
    }
  }
  fn write_chars(&mut self, x: u16, y: u16, text: &str, w: u16) {
    let start = (y as usize) * (w as usize) + (x as usize);
    for (i, ch) in text.chars().enumerate() {
      let idx = start + i;
      if idx >= self.cells.len() {
        break;
      }
      self.cells[idx].ch = ch;
    }
  }
  fn write_styled(
    &mut self,
    x: u16,
    y: u16,
    text: &str,
    fg: Color,
    bg: Color,
    w: u16,
  ) {
    let start = (y as usize) * (w as usize) + (x as usize);
    for (i, ch) in text.chars().enumerate() {
      let idx = start + i;
      if idx >= self.cells.len() {
        break;
      }
      let cell = &mut self.cells[idx];
      cell.ch = ch;
      cell.fg = fg;
      cell.bg = bg;
    }
  }
}
#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
struct Cell {
  ch: char,
  fg: Color,
  bg: Color,
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub enum Color {
  Reset,
  Rgb { r: u8, g: u8, b: u8 },
  Ansi(u8),
}
impl From<Color> for crossterm::style::Color {
  fn from(value: Color) -> Self {
    use crossterm::style::Color::*;
    match value {
      Color::Reset => Reset,
      Color::Rgb { r, g, b } => Rgb { r, g, b },
      Color::Ansi(ansi) => AnsiValue(ansi),
    }
  }
}
