use std::{collections::HashMap, path::Path};

use indexmap::IndexMap;
use quick_xml::{
  events::{attributes::Attribute, Event},
  Reader,
};
use serde::{Deserialize, Serialize};

use crate::{Color, QuickXmlAsStr};

#[derive(Debug, Clone, Default)]
pub struct JBColorScheme<'a> {
  _sources: Vec<String>,
  pub colors: IndexMap<&'a str, Color<'a>>,
  pub attributes: IndexMap<&'a str, JBAttribute<'a>>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, enumn::N)]
#[repr(u32)]
pub enum FontType {
  Bold = 1,
  Italic = 2,
  BoldItalic = 3,

  #[default]
  None = 0,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, enumn::N)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JBAttributeData<'a> {
  #[serde(borrow)]
  pub foreground: Option<Color<'a>>,
  #[serde(borrow)]
  pub background: Option<Color<'a>>,
  #[serde(borrow)]
  pub effect_color: Option<Color<'a>>,
  #[serde(borrow)]
  pub error_stripe_color: Option<Color<'a>>,
  pub effect_type: EffectType,
  pub font_type: FontType,
}

#[derive(Debug, Clone, Default)]
pub enum JBAttribute<'a> {
  BaseAttribute(&'a str),
  Data(JBAttributeData<'a>),

  #[default]
  None,
}

impl<'a> JBAttribute<'a> {
  fn inner_data_mut_ref(&mut self) -> &mut JBAttributeData<'a> {
    match self {
      Self::Data(data) => data,
      _ => unreachable!(),
    }
  }

  fn set(&mut self, key: &str, value: &'a str) {
    let data = self.inner_data_mut_ref();

    match key {
      "BACKGROUND" => data.background = Some(Color::from(value)),
      "FOREGROUND" => data.foreground = Some(Color::from(value)),
      "FONT_TYPE" => {
        data.font_type = value
          .parse::<u32>()
          .map(FontType::n)
          .ok()
          .flatten()
          .unwrap_or_default();
      }
      "EFFECT_TYPE" => {
        data.effect_type = value
          .parse::<u32>()
          .map(EffectType::n)
          .ok()
          .flatten()
          .unwrap_or_default();
      }
      "EFFECT_COLOR" => data.effect_color = Some(Color::from(value)),
      "ERROR_STRIPE_COLOR" => data.error_stripe_color = Some(Color::from(value)),
      _ => {}
    }
  }
}

pub struct JBColorSchemeReader<'a> {
  _src: &'a str,
  in_colors: bool,
  in_attributes: bool,
  attribute: Option<JBAttribute<'a>>,
  option: Option<&'a str>,
  reader: Reader<&'a [u8]>,
}

pub enum JBColorSchemeType<'a> {
  Color(&'a str, Color<'a>),
  Attribute(&'a str, JBAttribute<'a>),
}

impl<'a> JBColorSchemeReader<'a> {
  /// because of lifetime, can't use trait
  #[allow(clippy::should_implement_trait)]
  pub fn from_str(_src: &'a str) -> Self {
    Self {
      _src,
      in_colors: false,
      in_attributes: false,
      option: None,
      attribute: None,
      reader: Reader::from_str(_src),
    }
  }

  fn handle_event(&mut self, event: Event) -> Option<Option<JBColorSchemeType<'a>>> {
    match event {
      Event::Start(e) => match e.name().as_ref() {
        b"colors" => self.in_colors = true,
        b"attributes" => self.in_attributes = true,
        b"option" if self.in_attributes => unsafe {
          let Attribute { value: name, .. } = e.try_get_attribute(b"name").ok()??;

          self.option = Some(name.as_str());
          self.attribute = Some(JBAttribute::Data(JBAttributeData::default()));
        },
        _ => {}
      },
      Event::End(e) => match e.name().as_ref() {
        b"colors" => self.in_colors = false,
        b"attributes" => self.in_attributes = false,
        b"option" if self.in_attributes => {
          if let (Some(name), Some(attribute)) = (self.option.take(), self.attribute.take()) {
            return Some(Some(JBColorSchemeType::Attribute(name, attribute)));
          }
        }
        _ => {}
      },
      Event::Empty(e) => match e.name().as_ref() {
        b"option" if self.in_colors => unsafe {
          let Attribute { value: name, .. } = e.try_get_attribute(b"name").ok()??;
          let Attribute { value: color, .. } = e.try_get_attribute(b"value").ok()??;

          let name = name.as_str();
          let color = color.as_str();

          return Some(Some(JBColorSchemeType::Color(name, Color::from(color))));
        },
        b"option" if self.in_attributes && self.option.is_none() => unsafe {
          let Attribute { value: name, .. } = e.try_get_attribute(b"name").ok()??;
          let Attribute { value: base, .. } = e.try_get_attribute(b"baseAttributes").ok()??;

          let name = name.as_str();
          let base = base.as_str();

          return Some(Some(JBColorSchemeType::Attribute(
            name,
            JBAttribute::BaseAttribute(base),
          )));
        },
        b"option" if self.in_attributes => unsafe {
          if let Some(attribute) = &mut self.attribute {
            let Attribute { value: key, .. } = e.try_get_attribute(b"name").ok()??;
            let Attribute { value, .. } = e.try_get_attribute(b"value").ok()??;

            let key = key.as_str();
            let value = value.as_str();

            attribute.set(key, value);
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

impl<'a> Iterator for JBColorSchemeReader<'a> {
  type Item = JBColorSchemeType<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let event = self.reader.read_event().ok()?;

      if let Some(value) = self.handle_event(event)? {
        return Some(value);
      }
    }
  }
}

impl<'a> JBColorScheme<'a> {
  pub fn read_file(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
    let string = std::fs::read_to_string(path)?;

    self.read_string(string)
  }

  pub fn read_string(&mut self, string: String) -> anyhow::Result<()> {
    self._sources.push(string);

    let str = self._sources[self._sources.len() - 1].as_str();

    unsafe {
      // Complains not living along enough and borrowing data while mutating
      // will never die since _sources is just a holder to keep it alive
      // and it's elements will never be removed
      self.read_str(std::mem::transmute(str))
    }
  }

  pub fn read_str(&mut self, str: &'a str) -> anyhow::Result<()> {
    for e in JBColorSchemeReader::from_str(str) {
      match e {
        JBColorSchemeType::Color(name, color) => {
          self.colors.insert(name, color);
        }
        JBColorSchemeType::Attribute(option, value) => {
          self.attributes.insert(option, value);
        }
      }
    }

    Ok(())
  }

  pub fn parse(str: &'a str) -> anyhow::Result<Self> {
    let mut theme = Self::default();

    theme.read_str(str)?;

    Ok(theme)
  }

  pub fn get_attributes(&self) -> HashMap<&'a str, &'a JBAttributeData> {
    self
      .attributes
      .iter()
      .filter_map(|(&k, v)| Some((k, self.get_attribute_data(v)?)))
      .collect()
  }

  fn get_attribute_data(&self, attr: &'a JBAttribute) -> Option<&'a JBAttributeData> {
    match attr {
      JBAttribute::BaseAttribute(name) => self.get_attribute(name),
      JBAttribute::Data(data) => Some(data),
      JBAttribute::None => None,
    }
  }

  pub fn get_attribute(&self, attr: &str) -> Option<&'a JBAttributeData> {
    self.get_attribute_data(self.attributes.get(attr)?)
  }
}
