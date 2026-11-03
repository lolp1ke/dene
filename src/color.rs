// SPDX-License-Identifier: Apache-2.0

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub enum Color {
  Reset,
  Rgb(Rgb),
  Ansi(u8),
}
impl From<Color> for crossterm::style::Color {
  fn from(value: Color) -> Self {
    use crossterm::style;
    match value {
      Color::Reset => style::Color::Reset,
      Color::Rgb(Rgb { r, g, b }) => style::Color::Rgb {
        r: (r * 255.0) as u8,
        g: (g * 255.0) as u8,
        b: (b * 255.0) as u8,
      },
      Color::Ansi(ansi) => style::Color::AnsiValue(ansi),
    }
  }
}

#[inline]
pub fn rgb(hex: u32) -> Rgb {
  let [_, r, g, b] = hex.to_be_bytes().map(|b| (b as f32) / 255.0);
  Rgb { r, g, b }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub struct Rgb {
  pub r: f32,
  pub g: f32,
  pub b: f32,
}
