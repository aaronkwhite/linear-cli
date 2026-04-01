use console::Style;
use std::env;

pub fn is_color_enabled() -> bool {
    env::var("NO_COLOR").is_err() && console::Term::stdout().is_term()
}

pub fn bold(text: &str) -> String {
    if is_color_enabled() {
        Style::new().bold().apply_to(text).to_string()
    } else {
        text.to_string()
    }
}

pub fn dim(text: &str) -> String {
    if is_color_enabled() {
        Style::new().dim().apply_to(text).to_string()
    } else {
        text.to_string()
    }
}

pub fn green(text: &str) -> String {
    if is_color_enabled() {
        Style::new().green().apply_to(text).to_string()
    } else {
        text.to_string()
    }
}

pub fn yellow(text: &str) -> String {
    if is_color_enabled() {
        Style::new().yellow().apply_to(text).to_string()
    } else {
        text.to_string()
    }
}

pub fn red(text: &str) -> String {
    if is_color_enabled() {
        Style::new().red().apply_to(text).to_string()
    } else {
        text.to_string()
    }
}

pub fn cyan(text: &str) -> String {
    if is_color_enabled() {
        Style::new().cyan().apply_to(text).to_string()
    } else {
        text.to_string()
    }
}

pub fn format_priority(priority: i32) -> String {
    match priority {
        1 => red("!!!"),
        2 => yellow("!!"),
        3 => cyan("!"),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_priority_urgent() {
        let result = format_priority(1);
        assert!(result.contains("!!!") || result == "!!!");
    }

    #[test]
    fn test_format_priority_high() {
        let result = format_priority(2);
        assert!(result.contains("!!") || result == "!!");
    }

    #[test]
    fn test_format_priority_medium() {
        let result = format_priority(3);
        assert!(result.contains("!") || result == "!");
    }

    #[test]
    fn test_format_priority_low() {
        assert_eq!(format_priority(4), "");
    }

    #[test]
    fn test_format_priority_none() {
        assert_eq!(format_priority(0), "");
    }

    #[test]
    fn test_no_color_returns_plain_text() {
        let result = bold("hello");
        assert!(result.contains("hello"));
    }
}
