use crate::client::LinearClient;
use clap::Args;

#[derive(Args, Debug)]
pub struct MeArgs {}

pub async fn execute(_args: &MeArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug, None)?;

    let result = client
        .query_raw(
            r#"query {
            viewer {
                id name displayName email active admin
                organization { id name urlKey }
                teamMemberships { nodes { team { key name } } }
            }
        }"#,
            None,
        )
        .await?;

    if json {
        crate::output::print_json(&result);
    } else {
        let viewer = &result["data"]["viewer"];
        let name = viewer
            .get("displayName")
            .or(viewer.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let email = viewer.get("email").and_then(|v| v.as_str()).unwrap_or("-");
        let org = viewer
            .pointer("/organization/name")
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let admin = viewer
            .get("admin")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        crate::output::detail::print_detail("Name", name, 0);
        crate::output::detail::print_detail("Email", email, 0);
        crate::output::detail::print_detail("Organization", org, 0);
        crate::output::detail::print_detail("Role", if admin { "Admin" } else { "Member" }, 0);

        if let Some(teams) = viewer
            .pointer("/teamMemberships/nodes")
            .and_then(|v| v.as_array())
        {
            let team_list: Vec<&str> = teams
                .iter()
                .filter_map(|t| t.pointer("/team/key").and_then(|v| v.as_str()))
                .collect();
            if !team_list.is_empty() {
                crate::output::detail::print_detail("Teams", &team_list.join(", "), 0);
            }
        }
    }
    Ok(())
}
