use std::collections::HashMap;

use quick_xml::{
  events::{attributes::Attribute, Event},
  Reader,
};

use crate::{Color, UnsafeAsStr};

#[derive(Debug, Clone, Default)]
pub struct Theme<'a> {
  pub colors: HashMap<&'a str, Color<'a>>,
  pub attributes: HashMap<&'a str, ThemeAttribute<'a>>,
}

#[derive(Debug, Clone, Copy, Default, enumn::N)]
#[repr(u32)]
pub enum FontType {
  Bold = 1,
  Italic = 2,
  BoldItalic = 3,

  #[default]
  None = 0,
}

#[derive(Debug, Clone, Copy, Default, enumn::N)]
#[repr(u32)]
pub enum EffectType {
  Underscored = 0,
  BoldUnderscored = 1,
  Underwave = 2,
  Bordered = 3,
  Strike = 4,
  Dotted = 5,

  #[default]
  None,
}

#[derive(Debug, Clone, Default)]
pub struct ThemeAttribute<'a> {
  pub foreground: Option<Color<'a>>,
  pub background: Option<Color<'a>>,
  pub effect_color: Option<Color<'a>>,
  pub error_stripe_color: Option<Color<'a>>,
  pub effect_type: EffectType,
  pub font_type: FontType,
}

pub struct ThemeReader<'a> {
  _src: &'a str,
  in_colors: bool,
  in_attributes: bool,
  attribute: Option<ThemeAttribute<'a>>,
  option: Option<&'a str>,
  reader: Reader<&'a [u8]>,
}

impl<'a> ThemeReader<'a> {
  pub fn from(_src: &'a str) -> Self {
    Self {
      _src,
      in_colors: false,
      in_attributes: false,
      option: None,
      attribute: None,
      reader: Reader::from_str(_src),
    }
  }

  fn handle_event(&mut self, event: Event) -> Option<Option<ThemeEvent<'a>>> {
    match event {
      Event::Start(e) => match e.name().as_ref() {
        b"colors" => self.in_colors = true,
        b"attributes" => self.in_attributes = true,
        b"option" if self.in_attributes => unsafe {
          let Attribute { value: name, .. } = e.try_get_attribute(b"name").ok()??;

          self.option = Some(name.as_str_unchecked());
          self.attribute = Some(ThemeAttribute::default());
        },
        _ => {}
      },
      Event::End(e) => match e.name().as_ref() {
        b"colors" => self.in_colors = false,
        b"attributes" => self.in_attributes = false,
        b"option" if self.in_attributes => {
          if let (Some(name), Some(attribute)) = (self.option.take(), self.attribute.take()) {
            return Some(Some(ThemeEvent::Attribute(name, attribute)));
          }
        }
        _ => {}
      },
      Event::Empty(e) => match e.name().as_ref() {
        b"option" if self.in_colors => unsafe {
          let Attribute { value: name, .. } = e.try_get_attribute(b"name").ok()??;
          let Attribute { value: color, .. } = e.try_get_attribute(b"value").ok()??;

          let name = name.as_str_unchecked();
          let color = color.as_str_unchecked();

          return Some(Some(ThemeEvent::Color(name, Color::from(color))));
        },
        b"option" if self.in_attributes => unsafe {
          if let Some(attribute) = &mut self.attribute {
            let Attribute { value: key, .. } = e.try_get_attribute(b"name").ok()??;
            let Attribute { value, .. } = e.try_get_attribute(b"value").ok()??;

            let value = value.as_str_unchecked();

            match key.as_ref() {
              b"BACKGROUND" => attribute.background = Some(Color::from(value)),
              b"FOREGROUND" => attribute.foreground = Some(Color::from(value)),
              b"FONT_TYPE" => {
                attribute.font_type = value.parse::<u32>().map(FontType::n).ok()??
              }
              b"EFFECT_TYPE" => {
                attribute.effect_type = value.parse::<u32>().map(EffectType::n).ok()??
              }
              b"EFFECT_COLOR" => attribute.effect_color = Some(Color::from(value)),
              b"ERROR_STRIPE_COLOR" => attribute.error_stripe_color = Some(Color::from(value)),
              _ => {}
            }
          }
        },
        _ => {}
      },
      Event::Eof => return None,
      _ => {}
    };

    Some(None)
  }
}

impl<'a> Iterator for ThemeReader<'a> {
  type Item = ThemeEvent<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let event = self.reader.read_event().ok()?;

      if let Some(value) = self.handle_event(event)? {
        return Some(value);
      }
    }
  }
}

pub enum ThemeEvent<'a> {
  Color(&'a str, Color<'a>),
  Attribute(&'a str, ThemeAttribute<'a>),
}

impl<'a> Theme<'a> {
  pub fn parse(str: &'a str) -> anyhow::Result<Self> {
    let mut theme = Self::default();

    for e in ThemeReader::from(str) {
      match e {
        ThemeEvent::Color(name, color) => {
          theme.colors.insert(name, color);
        }
        ThemeEvent::Attribute(option, value) => {
          theme.attributes.insert(option, value);
        }
      }
    }

    Ok(theme)
  }
}