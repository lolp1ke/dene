// SPDX-License-Identifier: Apache-2.0

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(Default)]
pub struct IndexPath {
  pub section: usize,
  pub row: usize,
  pub col: usize,
}
impl IndexPath {
  #[must_use]
  pub fn row(mut self, row: usize) -> Self {
    self.row = row;
    self
  }
  #[must_use]
  pub fn col(mut self, col: usize) -> Self {
    self.col = col;
    self
  }
}
