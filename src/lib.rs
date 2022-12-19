use std::{
  borrow::Cow,
  fmt::{Debug, Formatter},
};

use colored::Colorize;
use quick_xml::name::QName;
use serde::Deserialize;

pub mod theme;

/// `dbg` without formatting multi-line
#[macro_export]
macro_rules! dbgl {
  ($val:expr $(,)?) => {
    match $val {
      tmp => {
        eprintln!(
          "[{}:{}] {} = {:?}",
          file!(),
          line!(),
          stringify!($val),
          &tmp
        );
        tmp
      }
    }
  };
}

#[derive(Deserialize, Copy, Clone)]
pub struct Color<'a>(&'a str);

impl<'a> From<&'a str> for Color<'a> {
  fn from(value: &'a str) -> Self {
    Color(value)
  }
}

impl<'a> Debug for Color<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match hex(self.0.trim_start_matches('#')) {
      Some([r, g, b, _]) => {
        if colored::control::SHOULD_COLORIZE.should_colorize() {
          write!(f, "{}", self.0.truecolor(r, g, b))
        } else {
          write!(f, "{}", self.0)
        }
      }
      None => write!(f, "{}", self.0),
    }
  }
}

fn hex(str: &str) -> Option<[u8; 4]> {
  let [r, g, b, a] = u32::from_str_radix(str, 16).ok()?.to_le_bytes();

  match str.len() {
    1 | 2 => Some([r, r, r, 255]),
    3 | 4 => Some([r, g, g, 255]),
    5 | 6 => Some([r, g, b, 255]),
    7 | 8 => Some([r, g, b, a]),
    _ => None,
  }
}

pub(crate) trait QuickXmlAsStr<'a, 'b: 'a> {
  /// # Safety
  ///
  /// quick_xml for whatever reason uses `&[u8]` instead of `&str`
  /// so this converts them back to `&str`
  /// but the problem is lifetimes and borrowing
  /// so there is literally no way to guarantee safety (like [std::str::from_utf8])
  unsafe fn as_str(&'a self) -> &'b str;
}

impl<'a, 'b: 'a> QuickXmlAsStr<'a, 'b> for Cow<'a, [u8]> {
  #[allow(clippy::transmute_bytes_to_str)]
  unsafe fn as_str(&'a self) -> &'b str {
    std::mem::transmute(self.as_ref())
  }
}

impl<'a, 'b: 'a> QuickXmlAsStr<'a, 'b> for QName<'a> {
  #[allow(clippy::transmute_bytes_to_str)]
  unsafe fn as_str(&'a self) -> &'b str {
    std::mem::transmute(self.as_ref())
  }
}
