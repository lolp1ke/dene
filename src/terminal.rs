// SPDX-License-Identifier: Apache-2.0

use std::{
  io::{Stdout, Write, stdout},
  sync::{Arc, OnceLock},
};

use crossterm::{cursor, event, execute, queue, style, terminal};
use parking_lot::RwLock;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{Color, Rect, Rgb};

pub(crate) static TERMINAL: OnceLock<RwLock<Terminal>> = OnceLock::new();

#[inline]
pub(crate) fn get_terminal() -> &'static RwLock<Terminal> {
  TERMINAL.get().expect("call `Terminal::new()` first")
}

#[derive(Debug)]
pub(crate) struct AnsiOverlay {
  x: u16,
  y: u16,
  width: u16,
  ansi: Arc<str>,
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
      style::ResetColor,
      style::SetAttribute(style::Attribute::Reset),
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

    Self::with_size(stdout, width, height)
  }

  fn with_size(stdout: Stdout, width: u16, height: u16) -> Self {
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
      cell.symbol = "\0".into();
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
    self.push_clip_axes(bounds, true, true);
  }

  pub(crate) fn push_clip_axes(&mut self, bounds: Rect, x: bool, y: bool) {
    let mut clip = self.visible_bounds(bounds);
    if !x {
      clip.x = 0;
      clip.width = self.width;
    }
    if !y {
      clip.y = 0;
      clip.height = self.height;
    }
    self.clip_rect_stack.push(clip);
  }

  pub(crate) fn clear(&mut self) {
    self.back.clear();
    self.ansi_overlays.clear();
  }
  pub(crate) fn render(&mut self) {
    for overlay in &self.prev_ansi_overlays {
      let start = usize::from(overlay.y) * usize::from(self.width)
        + usize::from(overlay.x);
      for cell in
        &mut self.front.cells[start..start + usize::from(overlay.width)]
      {
        cell.symbol = "\0".into();
      }
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
      let cell = &self.back.cells[run_start];
      let mut text = String::with_capacity(8);

      while i < total
        && self.back.cells[i] != self.front.cells[i]
        && self.back.cells[i].fg == cell.fg
        && self.back.cells[i].bg == cell.bg
      {
        text.push_str(&self.back.cells[i].symbol);
        i += 1;
        if i % w == 0 {
          break;
        };
      }

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
      let index = usize::from(overlay.y) * w + usize::from(overlay.x);
      let cell = &self.back.cells[index];
      _ = queue!(
        self.stdout,
        cursor::MoveTo(overlay.x, overlay.y),
        style::SetForegroundColor(cell.fg.into()),
        style::SetBackgroundColor(cell.bg.into()),
        style::Print(&*overlay.ansi),
        style::ResetColor,
        style::SetAttribute(style::Attribute::Reset),
      );
    }

    _ = self.stdout.flush();
    std::mem::swap(&mut self.front, &mut self.back);
    std::mem::swap(&mut self.ansi_overlays, &mut self.prev_ansi_overlays);
  }
  pub(crate) fn restore(&mut self) {
    _ = terminal::disable_raw_mode();
    _ = execute!(
      self.stdout,
      style::ResetColor,
      style::SetAttribute(style::Attribute::Reset),
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
    for grapheme in buf.as_ref().graphemes(true) {
      if x >= i64::from(self.width) {
        break;
      }
      let grapheme = if grapheme.chars().any(char::is_control) {
        " "
      } else {
        grapheme
      };
      let width = grapheme.width();
      if width == 0 {
        continue;
      }
      // A grapheme is indivisible: never paint half a wide character.
      if !self.is_clipped(x, y)
        && !self.is_clipped(x.saturating_add(width as i64 - 1), y)
      {
        let index = y as usize * usize::from(self.width) + x as usize;
        self.invalidate_overlays(Rect {
          x: x as u16,
          y: y as u16,
          width: width as u16,
          height: 1,
        });
        self.back.write(index, grapheme, width);
      }
      x = x.saturating_add(width as i64);
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
    if text.chars().any(char::is_control) {
      return;
    }
    let mut chars = ansi.chars();
    let mut painted = String::new();
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
      } else if ch.is_control() {
        return;
      } else {
        painted.push(ch);
      }
    }
    if painted.graphemes(true).any(|g| g.width() == 0)
      || text.graphemes(true).any(|g| g.width() == 0)
    {
      return;
    }
    let width = painted.width().max(text.width());
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
    self.ansi_overlays.push(AnsiOverlay {
      x: visible.x,
      y: visible.y,
      width: visible.width,
      ansi: ansi.into(),
    });
  }

  pub(crate) fn fill_background(&mut self, bounds: Rect, color: Rgb) {
    let visible = self.visible_bounds(bounds);
    self.invalidate_overlays(visible);

    let left = usize::from(visible.x);
    let top = usize::from(visible.y);
    let height = usize::from(visible.height);
    let width = usize::from(visible.width);

    for y in top..(top + height) {
      for x in left..(left + width) {
        let idx = y * usize::from(self.width) + x;
        self.back.write(idx, " ", 1);
        self.back.cells[idx].bg = Color::Rgb(color);
      }
    }
  }

  fn invalidate_overlays(&mut self, bounds: Rect) {
    if bounds.width == 0 || bounds.height == 0 {
      return;
    }
    let right = u32::from(bounds.x) + u32::from(bounds.width);
    let bottom = u32::from(bounds.y) + u32::from(bounds.height);
    self.ansi_overlays.retain(|overlay| {
      u32::from(overlay.y) < u32::from(bounds.y)
        || u32::from(overlay.y) >= bottom
        || u32::from(overlay.x) >= right
        || u32::from(overlay.x) + u32::from(overlay.width)
          <= u32::from(bounds.x)
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
          symbol: " ".into(),
          width: 1,
          fg: Color::Reset,
          bg: Color::Reset,
        };
        len
      ],
    }
  }

  fn clear(&mut self) {
    for cell in self.cells.iter_mut() {
      cell.clear_all();
    }
  }

  fn write(&mut self, idx: usize, symbol: &str, width: usize) {
    let background = self.cells[idx].bg;
    for occupied in idx..idx + width {
      let mut start = occupied;
      while self.cells[start].width == 0 && start > 0 {
        start -= 1;
      }
      let end = start + self.cells[start].width as usize;
      for cell in &mut self.cells[start..end] {
        cell.clear_symbol();
      }
    }
    self.cells[idx].symbol.clear();
    self.cells[idx].symbol.push_str(symbol);
    self.cells[idx].width = width as u16;
    for cell in &mut self.cells[idx + 1..idx + width] {
      cell.symbol.clear();
      cell.width = 0;
      cell.bg = background;
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
    let mut offset = 0;
    for symbol in text.graphemes(true) {
      let width = symbol.width();
      let idx = start + offset;
      if idx + width > self.cells.len()
        || x as usize + offset + width > w as usize
      {
        break;
      }
      if width == 0 {
        continue;
      }
      self.write(idx, symbol, width);
      for cell in &mut self.cells[idx..idx + width] {
        cell.fg = fg;
        cell.bg = bg;
      }
      offset += width;
    }
  }
}
#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
struct Cell {
  symbol: String,
  width: u16,
  fg: Color,
  bg: Color,
}
impl Cell {
  fn clear_all(&mut self) {
    self.clear_symbol();
    self.bg = Color::Reset;
  }

  fn clear_symbol(&mut self) {
    self.symbol.clear();
    self.symbol.push(' ');
    self.width = 1;
    self.fg = Color::Reset;
  }
}
