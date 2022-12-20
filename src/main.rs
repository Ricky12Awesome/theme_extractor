use theme_extractor::mappings::vscode_mappings;
use theme_extractor::theme::Theme;

fn main() -> anyhow::Result<()> {
  let source = include_str!("../theme/colorSchemes/RiderMelonDark.xml");
  let theme = Theme::parse(source)?;
  let mapper = vscode_mappings();

  for name in mapper.keys() {
    println!("{name}")
  }

  Ok(())
}

