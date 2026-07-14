// SPDX-License-Identifier: Apache-2.0

slotmap::new_key_type! {
  pub struct FocusId;
}

#[derive(Debug)]
pub struct FocusHandle {
  id: FocusId,
  tab_index: isize,
}
