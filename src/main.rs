use theme_extractor::theme::Theme;

fn main() -> anyhow::Result<()> {
  let source = include_str!("../theme/colorSchemes/RiderMelonDark.xml");
  let t = Theme::parse(source)?;

  dbg!(t);

  Ok(())
}

