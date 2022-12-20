use theme_extractor::{mappings::vscode_mappings, theme::JBColorScheme};

fn main() -> anyhow::Result<()> {
  let source = include_str!("../theme/colorSchemes/RiderMelonDark.xml");
  let theme = JBColorScheme::parse(source)?;
  let mappings = vscode_mappings();

  for (name, attr) in &theme.attributes {
    if name.starts_with("DEFAULT") {
      println!("{name} {:?}", attr.foreground.or(attr.effect_color));
    }
  }
  let [kw, ff, fd, pa, pr, br, s, cl, n, cm] = theme.get_attributes_fg_unwrap(
    &mappings,
    [
      "keyword",
      "function",
      "function.declaration",
      "parenthesis",
      "parameter",
      "brackets",
      "semicolon",
      "class",
      "number",
      "comment",
    ],
  );

  println!(
    r#"
{cm}// testing colors on an example
{kw}fn {ff}function{pa}({pr}value: {cl}Value{pa}) {br}{{
  {fd}declaration{pa}({n}69{pa}){s};
{br}}}
  "#
  );

  Ok(())
}
