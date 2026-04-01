use comfy_table::{presets::UTF8_FULL_CONDENSED, Attribute, Cell, Color, ContentArrangement, Table};

pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);

    if !headers.is_empty() {
        table.set_header(
            headers
                .iter()
                .map(|h| Cell::new(h).add_attribute(Attribute::Bold))
                .collect::<Vec<_>>(),
        );
    }

    for row in rows {
        table.add_row(row);
    }

    println!("{table}");
}

pub fn print_table_with_status(
    headers: &[&str],
    rows: &[Vec<String>],
    status_col: usize,
) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);

    if !headers.is_empty() {
        table.set_header(
            headers
                .iter()
                .map(|h| Cell::new(h).add_attribute(Attribute::Bold))
                .collect::<Vec<_>>(),
        );
    }

    for row in rows {
        let cells: Vec<Cell> = row
            .iter()
            .enumerate()
            .map(|(i, val)| {
                if i == status_col {
                    let color = match val.to_lowercase().as_str() {
                        s if s.contains("completed") || s.contains("done") => Color::Green,
                        s if s.contains("progress") || s.contains("started") => Color::Yellow,
                        s if s.contains("canceled") || s.contains("cancelled") => Color::Red,
                        _ => Color::Reset,
                    };
                    Cell::new(val).fg(color)
                } else {
                    Cell::new(val)
                }
            })
            .collect();
        table.add_row(cells);
    }

    println!("{table}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_table_no_panic() {
        print_table(
            &["ID", "Name"],
            &[
                vec!["1".into(), "Alice".into()],
                vec!["2".into(), "Bob".into()],
            ],
        );
    }

    #[test]
    fn test_print_table_empty() {
        print_table(&[], &[]);
    }
}
