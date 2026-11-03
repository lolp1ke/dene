// SPDX-License-Identifier: Apache-2.0

#[inline]
pub fn rgb(hex: u32) -> Rgb {
  let [_, r, g, b] = hex.to_le_bytes().map(|b| (b as f32) / 255.0);
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
