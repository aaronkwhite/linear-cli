pub mod cache;

use crate::error::LinearError;
use cache::Cache;
use reqwest::Client;
use std::env;
use std::path::PathBuf;
use std::time::Duration;

const API_URL: &str = "https://api.linear.app/graphql";
const MAX_RETRIES: u32 = 3;
const BACKOFF_BASE_MS: u64 = 1000;

pub struct LinearClient {
    http: Client,
    api_key: String,
    debug: bool,
    cache: Cache,
}

impl LinearClient {
    pub fn new(api_key: Option<String>, debug: bool) -> Result<Self, LinearError> {
        let api_key = match api_key {
            Some(key) => key,
            None => Self::resolve_api_key()?,
        };

        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(LinearError::Request)?;

        Ok(Self {
            http,
            api_key,
            debug,
            cache: Cache::new(),
        })
    }

    fn resolve_api_key() -> Result<String, LinearError> {
        // 1. Environment variable
        if let Ok(key) = env::var("LINEAR_API_KEY") {
            if !key.is_empty() {
                return Ok(key);
            }
        }

        // 2. .env and .env.local files
        for filename in &[".env", ".env.local"] {
            for dir in Self::search_dirs() {
                let path = dir.join(filename);
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    for line in contents.lines() {
                        let line = line.trim();
                        if let Some(val) = line.strip_prefix("LINEAR_API_KEY=") {
                            let val = val.trim().trim_matches('"').trim_matches('\'');
                            if !val.is_empty() {
                                return Ok(val.to_string());
                            }
                        }
                    }
                }
            }
        }

        Err(LinearError::NoApiKey)
    }

    fn search_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Ok(cwd) = env::current_dir() {
            dirs.push(cwd);
        }
        if let Ok(exe) = env::current_exe() {
            if let Some(parent) = exe.parent() {
                dirs.push(parent.to_path_buf());
            }
        }
        dirs
    }

    /// Execute a raw GraphQL query string, returning the raw JSON Value.
    /// Used by all commands for now — typed cynic queries can be added later.
    pub async fn query_raw(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, LinearError> {
        if self.debug {
            eprintln!("--- GraphQL Query ---\n{query}");
            if let Some(ref vars) = variables {
                eprintln!(
                    "--- Variables ---\n{}",
                    serde_json::to_string_pretty(vars).unwrap_or_default()
                );
            }
        }

        let mut body = serde_json::json!({"query": query});
        if let Some(vars) = &variables {
            body["variables"] = vars.clone();
        }

        let mut last_err = None;

        for attempt in 0..=MAX_RETRIES {
            let response = self
                .http
                .post(API_URL)
                .header("Content-Type", "application/json")
                .header("Authorization", &self.api_key)
                .json(&body)
                .send()
                .await
                .map_err(LinearError::Request)?;

            let status = response.status();

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS && attempt < MAX_RETRIES {
                let wait = Duration::from_millis(BACKOFF_BASE_MS * 2u64.pow(attempt));
                tokio::time::sleep(wait).await;
                continue;
            }

            let response_text = response.text().await.map_err(LinearError::Request)?;

            if self.debug {
                eprintln!("--- Response ---\n{response_text}");
            }

            if !status.is_success() {
                // Try to extract GraphQL error message
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response_text) {
                    if let Some(errors) = parsed.get("errors").and_then(|e| e.as_array()) {
                        let msgs: Vec<&str> = errors
                            .iter()
                            .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                            .collect();
                        if !msgs.is_empty() {
                            return Err(LinearError::GraphQL(msgs.join("; ")));
                        }
                    }
                }
                last_err = Some(LinearError::Http {
                    status: status.as_u16(),
                    body: response_text,
                });
                continue;
            }

            let gql_response: serde_json::Value =
                serde_json::from_str(&response_text).map_err(LinearError::Json)?;

            if let Some(errors) = gql_response.get("errors").and_then(|e| e.as_array()) {
                if !errors.is_empty() {
                    let msgs: Vec<&str> = errors
                        .iter()
                        .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                        .collect();
                    return Err(LinearError::GraphQL(msgs.join("; ")));
                }
            }

            return Ok(gql_response);
        }

        Err(last_err.unwrap_or(LinearError::GraphQL("Max retries exceeded".into())))
    }

    // --- Cache methods ---

    pub async fn get_teams(&self) -> Result<Vec<cache::CachedTeam>, LinearError> {
        if let Some(teams) = self.cache.teams.get() {
            return Ok(teams.clone());
        }
        let result = self.query_raw(
            "query { teams { nodes { id name key } } }",
            None,
        ).await?;
        let nodes = result
            .pointer("/data/teams/nodes")
            .ok_or_else(|| LinearError::GraphQL("No teams data".into()))?;
        let teams: Vec<cache::CachedTeam> = serde_json::from_value(nodes.clone())?;
        let _ = self.cache.teams.set(teams.clone());
        Ok(teams)
    }

    pub async fn get_team_id(&self, key_or_name: &str) -> Result<String, LinearError> {
        let teams = self.get_teams().await?;
        Ok(cache::find_team(&teams, key_or_name)?.id.clone())
    }

    pub async fn get_users(&self) -> Result<Vec<cache::CachedUser>, LinearError> {
        if let Some(users) = self.cache.users.get() {
            return Ok(users.clone());
        }
        let result = self.query_raw(
            "query { users(first: 250) { nodes { id name displayName email active } } }",
            None,
        ).await?;
        let nodes = result
            .pointer("/data/users/nodes")
            .ok_or_else(|| LinearError::GraphQL("No users data".into()))?;
        let users: Vec<cache::CachedUser> = serde_json::from_value(nodes.clone())?;
        let _ = self.cache.users.set(users.clone());
        Ok(users)
    }

    pub async fn get_user_id(&self, name: &str) -> Result<String, LinearError> {
        let users = self.get_users().await?;
        Ok(cache::find_user(&users, name)?.id.clone())
    }

    pub async fn get_project_id(&self, name: &str) -> Result<String, LinearError> {
        let projects = if let Some(p) = self.cache.projects.get() {
            p.clone()
        } else {
            let result = self.query_raw(
                "query { projects(first: 250) { nodes { id name } } }",
                None,
            ).await?;
            let nodes = result
                .pointer("/data/projects/nodes")
                .ok_or_else(|| LinearError::GraphQL("No projects data".into()))?;
            let projects: Vec<cache::CachedProject> = serde_json::from_value(nodes.clone())?;
            let _ = self.cache.projects.set(projects.clone());
            projects
        };
        Ok(cache::find_project(&projects, name)?.id.clone())
    }

    pub async fn get_state_id(&self, team_key: &str, state_name: &str) -> Result<String, LinearError> {
        if !self.cache.states.contains_key(team_key) {
            let team_id = self.get_team_id(team_key).await?;
            let result = self.query_raw(
                r#"query($teamId: ID!) { workflowStates(filter: { team: { id: { eq: $teamId } } }) { nodes { id name type } } }"#,
                Some(serde_json::json!({"teamId": team_id})),
            ).await?;
            let nodes = result
                .pointer("/data/workflowStates/nodes")
                .ok_or_else(|| LinearError::GraphQL("No states data".into()))?;
            let states: Vec<cache::CachedState> = serde_json::from_value(nodes.clone())?;
            self.cache.states.insert(team_key.to_string(), states);
        }
        let entry = self.cache.states.get(team_key).unwrap();
        Ok(cache::find_state(entry.value(), state_name)?.id.clone())
    }

    pub async fn get_label_ids(&self, names: &[&str], team_key: Option<&str>) -> Result<Vec<String>, LinearError> {
        let cache_key = team_key.unwrap_or("__workspace__");
        if !self.cache.labels.contains_key(cache_key) {
            let (query, variables) = if let Some(tk) = team_key {
                let team_id = self.get_team_id(tk).await?;
                (
                    r#"query($teamId: ID!) { issueLabels(filter: { team: { id: { eq: $teamId } } }) { nodes { id name } } }"#,
                    Some(serde_json::json!({"teamId": team_id})),
                )
            } else {
                ("query { issueLabels { nodes { id name } } }", None)
            };
            let result = self.query_raw(query, variables).await?;
            let nodes = result
                .pointer("/data/issueLabels/nodes")
                .ok_or_else(|| LinearError::GraphQL("No labels data".into()))?;
            let labels: Vec<cache::CachedLabel> = serde_json::from_value(nodes.clone())?;
            self.cache.labels.insert(cache_key.to_string(), labels);
        }
        let entry = self.cache.labels.get(cache_key).unwrap();
        cache::find_labels(entry.value(), names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_api_key_from_env() {
        temp_env::with_var("LINEAR_API_KEY", Some("lin_api_test123"), || {
            let key = LinearClient::resolve_api_key().unwrap();
            assert_eq!(key, "lin_api_test123");
        });
    }

    #[test]
    fn test_resolve_api_key_empty_env() {
        temp_env::with_var("LINEAR_API_KEY", Some(""), || {
            let result = LinearClient::resolve_api_key();
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_resolve_api_key_missing() {
        temp_env::with_var_unset("LINEAR_API_KEY", || {
            let result = LinearClient::resolve_api_key();
            assert!(result.is_err());
            let err = result.unwrap_err().to_string();
            assert!(err.contains("LINEAR_API_KEY not found"));
        });
    }

    #[test]
    fn test_new_with_explicit_key() {
        let client = LinearClient::new(Some("lin_api_explicit".into()), false);
        assert!(client.is_ok());
    }
}
