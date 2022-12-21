use std::path::PathBuf;

use clap::Parser;
use parse_display::FromStr;
use theme_extractor::colored_json::{ColoredValue, Colors};

use theme_extractor::theme::JBColorScheme;

#[derive(Default, Debug, Clone, FromStr)]
enum Mapper {
  #[from_str(regex = "(?i)(vs)?code")]
  VSCode,

  #[from_str(regex = "(?i)(jb|jetbrains)")]
  #[default]
  JetBrains,

  #[display("{0}")]
  Custom(PathBuf),
}

#[derive(Default, Debug, Parser)]
#[clap(author, version, long_about)]
struct Args {
  #[clap(short, long)]
  out: Option<PathBuf>,
  #[clap(short, long, default_value = "jb")]
  mapper: Mapper,
  sources: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
  let args = Args::try_parse()?;
  let mut theme = JBColorScheme::default();

  for file in args.sources {
    theme.read_file(file)?;
  }

  match args.mapper {
    Mapper::VSCode => {}
    Mapper::JetBrains => {
      let colors = Colors::from(&theme);
      let json = serde_json::json! {{
        "colors": &theme.colors,
        "attributes": theme.get_attributes(),
      }};
      let value = ColoredValue::new(&colors, &json);

      println!("{value:#}");
    }
    Mapper::Custom(_) => {}
  }

  Ok(())
}
