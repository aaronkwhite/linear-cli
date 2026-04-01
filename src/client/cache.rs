use crate::error::LinearError;
use dashmap::DashMap;
use serde::Deserialize;
use tokio::sync::OnceCell;

#[derive(Debug, Deserialize, Clone)]
pub struct CachedTeam {
    pub id: String,
    pub name: String,
    pub key: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CachedUser {
    pub id: String,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub active: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CachedProject {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CachedState {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub state_type: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CachedLabel {
    pub id: String,
    pub name: String,
}

pub struct Cache {
    pub teams: OnceCell<Vec<CachedTeam>>,
    pub users: OnceCell<Vec<CachedUser>>,
    pub projects: OnceCell<Vec<CachedProject>>,
    pub states: DashMap<String, Vec<CachedState>>,
    pub labels: DashMap<String, Vec<CachedLabel>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            teams: OnceCell::new(),
            users: OnceCell::new(),
            projects: OnceCell::new(),
            states: DashMap::new(),
            labels: DashMap::new(),
        }
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

pub fn find_team<'a>(teams: &'a [CachedTeam], key_or_name: &str) -> Result<&'a CachedTeam, LinearError> {
    let needle = key_or_name.to_lowercase();
    teams
        .iter()
        .find(|t| t.key.to_lowercase() == needle || t.name.to_lowercase() == needle)
        .ok_or_else(|| LinearError::NotFound {
            entity: "Team",
            name: key_or_name.to_string(),
        })
}

pub fn find_user<'a>(users: &'a [CachedUser], name: &str) -> Result<&'a CachedUser, LinearError> {
    let needle = name.to_lowercase();
    users
        .iter()
        .find(|u| {
            let display = u
                .display_name
                .as_deref()
                .or(u.name.as_deref())
                .unwrap_or("")
                .to_lowercase();
            display == needle || display.contains(&needle)
        })
        .ok_or_else(|| LinearError::NotFound {
            entity: "User",
            name: name.to_string(),
        })
}

pub fn find_project<'a>(
    projects: &'a [CachedProject],
    name: &str,
) -> Result<&'a CachedProject, LinearError> {
    let needle = name.to_lowercase();
    projects
        .iter()
        .find(|p| p.name.to_lowercase().contains(&needle))
        .ok_or_else(|| LinearError::NotFound {
            entity: "Project",
            name: name.to_string(),
        })
}

pub fn find_state<'a>(
    states: &'a [CachedState],
    state_name: &str,
) -> Result<&'a CachedState, LinearError> {
    let needle = state_name.to_lowercase();
    states
        .iter()
        .find(|s| s.name.to_lowercase() == needle)
        .ok_or_else(|| LinearError::NotFound {
            entity: "State",
            name: state_name.to_string(),
        })
}

pub fn find_labels(
    labels: &[CachedLabel],
    names: &[&str],
) -> Result<Vec<String>, LinearError> {
    names
        .iter()
        .map(|name| {
            let needle = name.to_lowercase().trim().to_string();
            labels
                .iter()
                .find(|l| l.name.to_lowercase() == needle)
                .map(|l| l.id.clone())
                .ok_or_else(|| LinearError::NotFound {
                    entity: "Label",
                    name: name.to_string(),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_teams() -> Vec<CachedTeam> {
        vec![
            CachedTeam { id: "t1".into(), name: "Engineering".into(), key: "ENG".into() },
            CachedTeam { id: "t2".into(), name: "Design".into(), key: "DES".into() },
        ]
    }

    fn test_users() -> Vec<CachedUser> {
        vec![
            CachedUser {
                id: "u1".into(),
                name: Some("Alice Smith".into()),
                display_name: Some("Alice".into()),
                email: None,
                active: true,
            },
            CachedUser {
                id: "u2".into(),
                name: Some("Bob Jones".into()),
                display_name: None,
                email: None,
                active: true,
            },
        ]
    }

    #[test]
    fn test_find_team_by_key() {
        let teams = test_teams();
        let team = find_team(&teams, "ENG").unwrap();
        assert_eq!(team.id, "t1");
    }

    #[test]
    fn test_find_team_by_name_case_insensitive() {
        let teams = test_teams();
        let team = find_team(&teams, "engineering").unwrap();
        assert_eq!(team.id, "t1");
    }

    #[test]
    fn test_find_team_not_found() {
        let teams = test_teams();
        let result = find_team(&teams, "NOPE");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_user_by_display_name() {
        let users = test_users();
        let user = find_user(&users, "Alice").unwrap();
        assert_eq!(user.id, "u1");
    }

    #[test]
    fn test_find_user_partial_match() {
        let users = test_users();
        let user = find_user(&users, "ali").unwrap();
        assert_eq!(user.id, "u1");
    }

    #[test]
    fn test_find_user_falls_back_to_name() {
        let users = test_users();
        let user = find_user(&users, "Bob Jones").unwrap();
        assert_eq!(user.id, "u2");
    }

    #[test]
    fn test_find_labels_all_found() {
        let labels = vec![
            CachedLabel { id: "l1".into(), name: "bug".into() },
            CachedLabel { id: "l2".into(), name: "feature".into() },
        ];
        let ids = find_labels(&labels, &["bug", "feature"]).unwrap();
        assert_eq!(ids, vec!["l1", "l2"]);
    }

    #[test]
    fn test_find_labels_one_missing() {
        let labels = vec![CachedLabel { id: "l1".into(), name: "bug".into() }];
        let result = find_labels(&labels, &["bug", "nope"]);
        assert!(result.is_err());
    }
}
