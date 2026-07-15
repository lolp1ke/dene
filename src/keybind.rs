// SPDX-License-Identifier: Apache-2.0

use std::{rc::Rc, sync::Arc};

use anyhow::Context;
use smallvec::SmallVec;

use crate::Action;

#[derive(Debug)]
#[derive(derive_more::Deref, derive_more::DerefMut)]
#[derive(Default)]
pub struct Keybinds(pub(crate) Vec<Keybind>);
impl Keybinds {
  pub(crate) fn match_input(
    &self,
    input: &[&Keystroke],
  ) -> (SmallVec<[&Keybind; 1]>, bool) {
    let mut exact = SmallVec::<[&Keybind; 1]>::new();
    let mut has_pending = false;

    for binding in self.0.iter() {
      if !binding.match_keystrokes(input) {
        continue;
      };

      if input.len() == binding.keystrokes.len() {
        exact.push(binding);
      } else {
        has_pending = true;
      };
    }

    (exact, has_pending)
  }
}

#[derive(Debug)]
pub struct Keybind {
  pub(crate) action: Box<dyn Action>,
  pub(crate) keystrokes: SmallVec<[Keystroke; 2]>,
  pub(crate) key_context: Option<Rc<KeybindContextPredicate>>,
}
impl Keybind {
  fn match_keystrokes(&self, input: &[&Keystroke]) -> bool {
    if input.len() > self.keystrokes.len() {
      return false;
    };

    for (target, input) in self.keystrokes.iter().zip(input.iter()) {
      if target.key != input.key || target.modifiers != input.modifiers {
        return false;
      };
    }
    true
  }
}

#[derive(Debug)]
pub enum KeybindContextPredicate {
  Ident(Arc<str>),
  Eq(Arc<str>, Arc<str>),
  Neq(Arc<str>, Arc<str>),
  Not(Box<Self>),
  And(Box<Self>, Box<Self>),
  Or(Box<Self>, Box<Self>),
}
impl KeybindContextPredicate {
  fn parse(src: &str) -> anyhow::Result<Self> {
    let src = remove_whitespace(src);
    let (this, rest) = Self::parse_expr(src, 0)?;
    if !rest.is_empty() {
      anyhow::bail!("unexpected end: {}", rest);
    };
    Ok(this)
  }

  fn parse_expr(
    mut src: &str,
    min_precendence: u32,
  ) -> anyhow::Result<(Self, &str)> {
    type Op = fn(
      KeybindContextPredicate,
      KeybindContextPredicate,
    ) -> anyhow::Result<KeybindContextPredicate>;
    let (mut lhs, rest) = Self::parse_primary(src)?;
    src = rest;

    'parse: loop {
      for (operator, precendence, constructor) in [
        ("||", PRECENDENCE_OR, Self::new_or as Op),
        ("&&", PRECENDENCE_AND, Self::new_and),
        ("==", PRECENDENCE_EQ, Self::new_eq),
        ("!=", PRECENDENCE_NEQ, Self::new_neq),
      ] {
        if src.starts_with(operator) {
          src = remove_whitespace(&src[operator.len()..]);
          let (rhs, rest) = Self::parse_expr(src, precendence)?;
          lhs = (constructor)(lhs, rhs)?;
          src = rest;
          continue 'parse;
        };
      }

      break 'parse;
    }

    Ok((lhs, src))
  }

  fn parse_primary(mut src: &str) -> anyhow::Result<(Self, &str)> {
    let next = src.chars().next().context("unexpected end")?;

    match next {
      '!' => {
        src = &src[1..];
        let (context, rest) = Self::parse_expr(src, PRECENDENCE_NOT)?;
        Ok((Self::Not(Box::new(context)), rest))
      }
      ch if is_ident_start_char(ch) => {
        let len = src.find(|ch| !is_ident_char(ch)).unwrap_or(src.len());
        let (ident, rest) = src.split_at(len);
        src = remove_whitespace(rest);
        Ok((Self::Ident(ident.into()), src))
      }
      _ => anyhow::bail!("unexpected char: {}", next),
    }
  }

  fn new_or(self, other: Self) -> anyhow::Result<Self> {
    Ok(Self::Or(Box::new(self), Box::new(other)))
  }
  fn new_and(self, other: Self) -> anyhow::Result<Self> {
    Ok(Self::And(Box::new(self), Box::new(other)))
  }
  fn new_eq(self, other: Self) -> anyhow::Result<Self> {
    if let (Self::Ident(lhs), Self::Ident(rhs)) = (&self, &other) {
      Ok(Self::Eq(lhs.clone(), rhs.clone()))
    } else {
      anyhow::bail!("Idents are expected; found: {:?} = {:?}", self, other)
    }
  }
  fn new_neq(self, other: Self) -> anyhow::Result<Self> {
    if let (Self::Ident(lhs), Self::Ident(rhs)) = (&self, &other) {
      Ok(Self::Neq(lhs.clone(), rhs.clone()))
    } else {
      anyhow::bail!("Idents are expected; found: {:?} = {:?}", self, other)
    }
  }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Keystroke {
  modifiers: Modifiers,
  key: Arc<str>,
  key_char: Option<Arc<str>>,
}
impl Keystroke {
  pub fn parse(source: &str) -> anyhow::Result<Self> {
    let mut modifiers = Modifiers::empty();
    let mut key = None;
    let mut key_char = None;

    let mut components = source.split('-').peekable();

    while let Some(component) = components.next() {
      if component.eq_ignore_ascii_case("ctrl") {
        modifiers |= Modifiers::CONTROL;
        continue;
      }
      if component.eq_ignore_ascii_case("alt") {
        modifiers |= Modifiers::ALT;
        continue;
      }
      if component.eq_ignore_ascii_case("shift") {
        modifiers |= Modifiers::SHIFT;
        continue;
      };
      if component.eq_ignore_ascii_case("meta")
        || component.eq_ignore_ascii_case("cmd")
        || component.eq_ignore_ascii_case("super")
        || component.eq_ignore_ascii_case("win")
      {
        modifiers |= Modifiers::META;
        continue;
      };

      let mut key_str = component.to_string();

      if let Some(next) = components.peek() {
        if next.is_empty() && source.ends_with('-') {
          key = Some(String::from("-"));
          break;
        } else if next.len() > 1 && next.starts_with('>') {
          key = Some(key_str.clone());
          components.next();
        } else {
          anyhow::bail!("Invalid keystroke: {}", source);
        }
        continue;
      }

      if component.len() == 1 && component.as_bytes()[0].is_ascii_uppercase() {
        modifiers |= Modifiers::SHIFT;
        key_str.make_ascii_lowercase();
      } else {
        key_str.make_ascii_lowercase();
      };

      key = Some(key_str.clone());
      if modifiers.contains(Modifiers::SHIFT) {
        key_char = Some(key_str.to_uppercase().to_string());
      } else {
        key_char = Some(key_str.to_lowercase().to_string());
      };
    }

    let key = key
      .ok_or_else(|| anyhow::anyhow!("Invalid keystroke: {}", source))?
      .into();
    let key_char = key_char.and_then(|key_char| {
      if key_char.len() != 1 {
        None
      } else {
        Some(key_char.into())
      }
    });

    Ok(Self {
      modifiers,
      key,
      key_char,
    })
  }
}

bitflags::bitflags! {
  #[derive(Debug)]
  #[derive(Clone, Copy)]
  #[derive(PartialEq)]
  pub struct Modifiers: u32 {
    const NONE = 1 << 0;
    const SHIFT = 1 << 1;
    const CONTROL = 1 << 2;
    const ALT = 1 << 3;
    const META = 1 << 4;
  }
}

const PRECENDENCE_OR: u32 = 2;
const PRECENDENCE_AND: u32 = 3;
const PRECENDENCE_EQ: u32 = 4;
const PRECENDENCE_NEQ: u32 = 4;
const PRECENDENCE_NOT: u32 = 5;

#[inline]
fn remove_whitespace(src: &str) -> &str {
  let start = src
    .find(|ch: char| !ch.is_whitespace())
    .unwrap_or(src.len());
  &src[start..]
}

#[inline]
fn is_ident_start_char(ch: char) -> bool {
  !ch.is_numeric() && is_ident_char(ch)
}
#[inline]
fn is_ident_char(ch: char) -> bool {
  ch.is_alphanumeric() || ch == '_'
}
