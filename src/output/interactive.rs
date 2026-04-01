use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, MultiSelect};

pub fn is_interactive() -> bool {
    Term::stdout().is_term()
}

pub fn fuzzy_select(prompt: &str, items: &[String]) -> anyhow::Result<usize> {
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()?;
    Ok(selection)
}

pub fn multi_select(prompt: &str, items: &[String]) -> anyhow::Result<Vec<usize>> {
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .interact()?;
    Ok(selections)
}

pub fn confirm(prompt: &str) -> anyhow::Result<bool> {
    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(false)
        .interact()?;
    Ok(confirmed)
}
