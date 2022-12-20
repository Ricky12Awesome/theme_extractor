use indexmap::IndexMap;
use quick_xml::{
  events::{attributes::Attribute, Event},
  Reader,
};

use crate::{mappings::Mappings, Color, QuickXmlAsStr};

#[derive(Debug, Clone, Default)]
pub struct JBColorScheme<'a> {
  pub colors: IndexMap<&'a str, Color<'a>>,
  pub attributes: IndexMap<&'a str, JBAttribute<'a>>,
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
pub struct JBAttribute<'a> {
  pub foreground: Option<Color<'a>>,
  pub background: Option<Color<'a>>,
  pub effect_color: Option<Color<'a>>,
  pub error_stripe_color: Option<Color<'a>>,
  pub effect_type: EffectType,
  pub font_type: FontType,
}

impl<'a> JBAttribute<'a> {
  fn set(&mut self, key: &str, value: &'a str) {
    match key {
      "BACKGROUND" => self.background = Some(Color::from(value)),
      "FOREGROUND" => self.foreground = Some(Color::from(value)),
      "FONT_TYPE" => {
        self.font_type = value
          .parse::<u32>()
          .map(FontType::n)
          .ok()
          .flatten()
          .unwrap_or_default();
      }
      "EFFECT_TYPE" => {
        self.effect_type = value
          .parse::<u32>()
          .map(EffectType::n)
          .ok()
          .flatten()
          .unwrap_or_default();
      }
      "EFFECT_COLOR" => self.effect_color = Some(Color::from(value)),
      "ERROR_STRIPE_COLOR" => self.error_stripe_color = Some(Color::from(value)),
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
          self.attribute = Some(JBAttribute::default());
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
  pub fn parse(str: &'a str) -> anyhow::Result<Self> {
    let mut theme = Self::default();

    for e in JBColorSchemeReader::from_str(str) {
      match e {
        JBColorSchemeType::Color(name, color) => {
          theme.colors.insert(name, color);
        }
        JBColorSchemeType::Attribute(option, value) => {
          theme.attributes.insert(option, value);
        }
      }
    }

    Ok(theme)
  }

  pub fn get_attribute(&'a self, mappings: &Mappings, name: &str) -> Option<&'a JBAttribute<'a>> {
    let (&name, _) = mappings
      .attributes
      .iter()
      .find(|(_, possible_names)| possible_names.contains(&name))?;

    self.attributes.get(name)
  }

  pub fn get_attribute_fg(&'a self, mappings: &Mappings, name: &str) -> Option<Color<'a>> {
    self.get_attribute(mappings, name)?.foreground
  }

  pub fn get_attributes_fg_unwrap<const N: usize>(
    &'a self,
    mappings: &Mappings,
    names: [&str; N],
  ) -> [Color<'a>; N] {
    names.map(|name| self.get_attribute_fg(mappings, name).expect(name))
  }
}
