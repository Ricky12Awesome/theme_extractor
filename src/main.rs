use theme_extractor::mappings::vscode_mappings;
use theme_extractor::theme::Theme;

fn main() -> anyhow::Result<()> {
  let source = include_str!("../theme/colorSchemes/RiderMelonDark.xml");
  let _theme = Theme::parse(source)?;
  let mappings = vscode_mappings();

  for name in mappings.attributes.keys() {
    println!("{name}");
  }

  Ok(())
}

