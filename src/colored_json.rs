use crate::theme::JBColorScheme;
use colored::{Color, Colorize};
use serde_json::Value;
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub struct Colors {
  pub string: Color,
  pub number: Color,
  pub null: Color,
  pub bool: Color,
  pub key: Color,
  pub brackets: Color,
  pub braces: Color,
  pub comma: Color,
  pub colon: Color,
}

impl From<crate::Color<'_>> for Color {
  fn from(c: crate::Color<'_>) -> Self {
    let [r, g, b, _] = c.as_hex().unwrap_or([255; 4]);

    Self::TrueColor { r, g, b }
  }
}

impl From<&JBColorScheme<'_>> for Colors {
  fn from(value: &JBColorScheme<'_>) -> Self {
    let def = Color::BrightWhite;

    Self {
      string: value
        .get_attribute("DEFAULT_STRING")
        .and_then(|attr| attr.foreground)
        .map(Color::from)
        .unwrap_or(def),

      number: value
        .get_attribute("DEFAULT_NUMBER")
        .and_then(|attr| attr.foreground)
        .map(Color::from)
        .unwrap_or(def),

      null: value
        .get_attribute("DEFAULT_KEYWORD")
        .and_then(|attr| attr.foreground)
        .map(Color::from)
        .unwrap_or(def),

      bool: value
        .get_attribute("DEFAULT_KEYWORD")
        .and_then(|attr| attr.foreground)
        .map(Color::from)
        .unwrap_or(def),

      key: value
        .get_attribute("DEFAULT_INSTANCE_FIELD")
        .and_then(|attr| attr.foreground)
        .map(Color::from)
        .unwrap_or(def),

      brackets: value
        .get_attribute("DEFAULT_BRACKETS")
        .and_then(|attr| attr.foreground)
        .map(Color::from)
        .unwrap_or(def),

      braces: value
        .get_attribute("DEFAULT_BRACES")
        .and_then(|attr| attr.foreground)
        .map(Color::from)
        .unwrap_or(def),

      comma: value
        .get_attribute("DEFAULT_COMMA")
        .and_then(|attr| attr.foreground)
        .map(Color::from)
        .unwrap_or(def),

      colon: value
        .get_attribute("DEFAULT_COMMA")
        .and_then(|attr| attr.foreground)
        .map(Color::from)
        .unwrap_or(def),
    }
  }
}

#[derive(Debug)]
pub struct ColoredValue<'a> {
  colors: &'a Colors,
  value: &'a Value,
  indent: usize,
}

impl<'a> ColoredValue<'a> {
  pub fn new(colors: &'a Colors, value: &'a Value) -> Self {
    Self {
      colors,
      value,
      indent: 0,
    }
  }

  pub fn nest(&self, value: &'a Value) -> Self {
    Self {
      colors: self.colors,
      value,
      indent: self.indent + 1,
    }
  }
}

impl<'a> Display for ColoredValue<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let pretty = f.alternate();

    match self.value {
      Value::Null => write!(f, "{}", "null".color(self.colors.null)),
      Value::Bool(b) => write!(f, "{}", b.to_string().color(self.colors.bool)),
      Value::Number(n) => write!(f, "{}", n.to_string().color(self.colors.number)),
      Value::String(s) => {
        write!(f, "{}", "\"".color(self.colors.string))?;
        write!(f, "{}", s.color(self.colors.string))?;
        write!(f, "{}", "\"".color(self.colors.string))
      }
      Value::Array(arr) if arr.is_empty() && pretty => {
        write!(f, "{}", "[".color(self.colors.braces))?;
        write!(f, "{}", "]".color(self.colors.braces))
      },
      Value::Array(arr) if !arr.is_empty() && pretty => {
        writeln!(f, "{}", "[".color(self.colors.braces))?;

        for i in 0..arr.len() {
          let value = &arr[i];
          let value = self.nest(value);

          for _ in 0..value.indent * 2 {
            write!(f, " ")?;
          }

          Display::fmt(&value, f)?;

          if i != arr.len() - 1 {
            writeln!(f, "{}", ",".color(self.colors.comma))?;
          }
        }

        writeln!(f)?;

        for _ in 0..self.indent * 2 {
          write!(f, " ")?;
        }

        write!(f, "{}", "]".color(self.colors.braces))
      }
      Value::Array(arr) => {
        write!(f, "{}", "[".color(self.colors.braces))?;

        for i in 0..arr.len() {
          let value = &arr[i];
          let value = ColoredValue::new(self.colors, value);

          Display::fmt(&value, f)?;

          if i != arr.len().saturating_sub(1) {
            write!(f, "{}", ", ".color(self.colors.comma))?;
          }
        }

        write!(f, "{}", "]".color(self.colors.braces))
      }
      Value::Object(map) if map.is_empty() && pretty => {
        write!(f, "{}", "{".color(self.colors.braces))?;
        write!(f, "{}", "}".color(self.colors.braces))
      },
      Value::Object(map) if pretty => {
        writeln!(f, "{}", "{".color(self.colors.braces))?;

        let arr = map.iter().collect::<Vec<_>>();

        for i in 0..arr.len() {
          let (key, value) = &arr[i];
          let value = self.nest(value);

          for _ in 0..value.indent * 2 {
            write!(f, " ")?;
          }

          write!(f, "{}", "\"".color(self.colors.key))?;
          write!(f, "{}", key.color(self.colors.key))?;
          write!(f, "{}", "\"".color(self.colors.key))?;
          write!(f, "{}", ": ".color(self.colors.colon))?;

          Display::fmt(&value, f)?;

          if i != arr.len().saturating_sub(1) {
            writeln!(f, "{}", ", ".color(self.colors.comma))?;
          }
        }

        writeln!(f)?;

        for _ in 0..self.indent * 2 {
          write!(f, " ")?;
        }

        write!(f, "{}", "}".color(self.colors.brackets))
      }
      Value::Object(map) => {
        write!(f, "{}", "{".color(self.colors.braces))?;

        let arr = map.iter().collect::<Vec<_>>();

        for i in 0..arr.len() {
          let (key, value) = &arr[i];
          let value = ColoredValue::new(self.colors, value);

          write!(f, "{}", "\"".color(self.colors.key))?;
          write!(f, "{}", key.color(self.colors.key))?;
          write!(f, "{}", "\"".color(self.colors.key))?;
          write!(f, "{}", ": ".color(self.colors.colon))?;

          Display::fmt(&value, f)?;

          if i != arr.len().saturating_sub(1) {
            write!(f, "{}", ", ".color(self.colors.comma))?;
          }
        }

        write!(f, "{}", "}".color(self.colors.brackets))
      }
    }
  }
}
