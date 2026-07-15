// SPDX-License-Identifier: Apache-2.0

use crate::App;

pub trait Global: 'static {}

pub trait ReadGlobal {
  fn global(app: &App) -> &Self;
}
impl<G> ReadGlobal for G
where
  G: Global,
{
  fn global(app: &App) -> &Self {
    todo!()
  }
}
