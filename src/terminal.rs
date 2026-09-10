// SPDX-License-Identifier: Apache-2.0

use std::{
  io::{Stdout, Write as _, stdout},
  sync::{Arc, OnceLock},
};

use crossterm::{cursor, event, execute, queue, style, terminal};
use parking_lot::RwLock;

use crate::Rect;

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
  width: u16,
  height: u16,

  pub(crate) clip_rect_stack: Vec<Rect>,
  pub(crate) draw_offset: (i64, i64),

  front: Buffer,
  back: Buffer,

  ansi_overlays: Vec<AnsiOverlay>,
  prev_ansi_overlays: Vec<AnsiOverlay>,
}
impl Terminal {
  pub(crate) fn new() -> Self {
    let mut stdout = stdout();
    _ = terminal::enable_raw_mode();
    _ = execute!(
      stdout,
      terminal::EnterAlternateScreen,
      terminal::Clear(terminal::ClearType::All),
      cursor::Hide,
    );
    _ = execute!(
      stdout,
      event::PushKeyboardEnhancementFlags(
        event::KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
          | event::KeyboardEnhancementFlags::REPORT_EVENT_TYPES
          | event::KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
          | event::KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
      ),
      event::EnableMouseCapture,
    );
    let (width, height) = Self::size();
    let buf_len = width as usize * height as usize;

    Self {
      stdout,
      width,
      height,
      clip_rect_stack: Vec::new(),
      draw_offset: (0, 0),
      front: Buffer::new(buf_len),
      back: Buffer::new(buf_len),
      ansi_overlays: Vec::new(),
      prev_ansi_overlays: Vec::new(),
    }
  }

  pub(crate) fn resize(&mut self, width: u16, height: u16) {
    self.width = width;
    self.height = height;
    let len = usize::from(width) * usize::from(height);
    self.front = Buffer::new(len);
    for cell in &mut self.front.cells {
      cell.ch = '\0';
    }
    self.back = Buffer::new(len);
    self.ansi_overlays.clear();
    self.prev_ansi_overlays.clear();
    self.clip_rect_stack.clear();
    self.draw_offset = (0, 0);
  }

  pub(crate) fn visible_bounds(&self, bounds: Rect) -> Rect {
    let x = self.draw_offset.0.saturating_add(i64::from(bounds.x));
    let y = self.draw_offset.1.saturating_add(i64::from(bounds.y));
    let mut left = x.clamp(0, i64::from(self.width));
    let mut top = y.clamp(0, i64::from(self.height));
    let mut right = x
      .saturating_add(i64::from(bounds.width))
      .clamp(0, i64::from(self.width));
    let mut bottom = y
      .saturating_add(i64::from(bounds.height))
      .clamp(0, i64::from(self.height));
    for clip in &self.clip_rect_stack {
      left = left.max(i64::from(clip.x).min(i64::from(self.width)));
      top = top.max(i64::from(clip.y).min(i64::from(self.height)));
      right = right.min(i64::from(clip.x) + i64::from(clip.width));
      bottom = bottom.min(i64::from(clip.y) + i64::from(clip.height));
    }
    Rect {
      x: left as u16,
      y: top as u16,
      width: (right - left).max(0) as u16,
      height: (bottom - top).max(0) as u16,
    }
  }

  pub(crate) fn push_clip(&mut self, bounds: Rect) {
    let clip = self.visible_bounds(bounds);
    self.clip_rect_stack.push(clip);
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
      _ = queue!(self.stdout, style::ResetColor);
      _ = queue!(self.stdout, style::SetAttribute(style::Attribute::Reset));
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
      terminal::Clear(terminal::ClearType::All),
      event::DisableMouseCapture,
    );
    _ = execute!(self.stdout, cursor::Show, terminal::LeaveAlternateScreen,);
  }
  pub(crate) fn size() -> (u16, u16) {
    terminal::size().unwrap_or((0, 0))
  }

  pub(crate) fn write_at<S>(&mut self, x: u16, y: u16, buf: S)
  where
    S: AsRef<str>,
  {
    let mut x = self.draw_offset.0.saturating_add(i64::from(x));
    let y = self.draw_offset.1.saturating_add(i64::from(y));
    if y < 0 || y >= i64::from(self.height) {
      return;
    }
    for ch in buf.as_ref().chars() {
      if x >= i64::from(self.width) {
        break;
      }
      if !self.is_clipped(x, y) {
        let index = y as usize * usize::from(self.width) + x as usize;
        self.back.cells[index].ch = if ch.is_control() { ' ' } else { ch };
      }
      x = x.saturating_add(1);
    }
  }
  pub(crate) fn write_ansi_at(
    &mut self,
    x: u16,
    y: u16,
    ansi: &str,
    text: &str,
  ) {
    self.write_at(x, y, text);
    let width = text.chars().count();
    if width == 0 || width > usize::from(u16::MAX) {
      return;
    }
    let visible = self.visible_bounds(Rect {
      x,
      y,
      width: width as u16,
      height: 1,
    });
    if usize::from(visible.width) != width || visible.height != 1 {
      return;
    }

    let mut chars = ansi.chars();
    let mut plain = text.chars();
    while let Some(ch) = chars.next() {
      if ch == '\u{1b}' {
        if chars.next() != Some('[') {
          return;
        }
        loop {
          match chars.next() {
            Some('m') => break,
            Some('0'..='9' | ';' | ':') => {}
            _ => return,
          }
        }
      } else if !(' '..='~').contains(&ch) || plain.next() != Some(ch) {
        return;
      }
    }
    if plain.next().is_some() {
      return;
    }
    self.ansi_overlays.push(AnsiOverlay {
      x: visible.x,
      y: visible.y,
      ansi: ansi.into(),
      text: text.into(),
    });
  }

  fn is_clipped(&self, x: i64, y: i64) -> bool {
    if x < 0
      || x >= i64::from(self.width)
      || y < 0
      || y >= i64::from(self.height)
    {
      return true;
    }
    for rect in &self.clip_rect_stack {
      if x < i64::from(rect.x)
        || x >= i64::from(rect.x) + i64::from(rect.width)
        || y < i64::from(rect.y)
        || y >= i64::from(rect.y) + i64::from(rect.height)
      {
        return true;
      };
    }
    false
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
