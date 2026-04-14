pub mod color;
pub mod detail;
pub mod interactive;
pub mod table;

use serde::Serialize;

pub fn print_json<T: Serialize>(value: &T) {
    let v = match serde_json::to_value(value) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error formatting JSON: {e}");
            return;
        }
    };
    // Strip GraphQL envelope: {"data": {...}} → {...}
    // Only strip when "data" is an object or array — never scalars.
    // INVARIANT: output structs constructed directly by commands must not use "data"
    // as a top-level field name, or they will be silently stripped.
    let output = v
        .get("data")
        .filter(|d| d.is_object() || d.is_array())
        .unwrap_or(&v);
    match serde_json::to_string(output) {
        Ok(json) => println!("{json}"),
        Err(e) => eprintln!("Error formatting JSON: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn format_json<T: Serialize>(value: &T) -> Option<String> {
        let v = serde_json::to_value(value).ok()?;
        let output = v
            .get("data")
            .filter(|d| d.is_object() || d.is_array())
            .unwrap_or(&v);
        serde_json::to_string(output).ok()
    }

    #[test]
    fn strips_graphql_data_envelope() {
        let wrapped = json!({ "data": { "issues": [{"id": "1"}] } });
        let result = format_json(&wrapped).unwrap();
        assert_eq!(result, r#"{"issues":[{"id":"1"}]}"#);
    }

    #[test]
    fn passes_through_non_envelope_json() {
        let plain = json!({ "id": "ENG-1", "title": "Fix bug" });
        let result = format_json(&plain).unwrap();
        assert_eq!(result, r#"{"id":"ENG-1","title":"Fix bug"}"#);
    }

    #[test]
    fn output_is_compact_not_pretty() {
        let value = json!({ "data": { "a": 1, "b": 2 } });
        let result = format_json(&value).unwrap();
        assert!(!result.contains('\n'), "output should not contain newlines");
        assert!(!result.contains("  "), "output should not contain indentation");
    }

    #[test]
    fn handles_array_at_data_root() {
        let wrapped = json!({ "data": [1, 2, 3] });
        let result = format_json(&wrapped).unwrap();
        assert_eq!(result, "[1,2,3]");
    }

    #[test]
    fn does_not_strip_scalar_data_field() {
        // A command output struct with a coincidental "data" key must not be mangled.
        let value = json!({ "data": null, "configured": false });
        let result = format_json(&value).unwrap();
        assert_eq!(result, r#"{"configured":false,"data":null}"#);
    }
}
