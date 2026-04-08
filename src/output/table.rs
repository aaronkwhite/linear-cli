use comfy_table::{Attribute, Cell, ContentArrangement, Table, presets::UTF8_FULL_CONDENSED};

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
