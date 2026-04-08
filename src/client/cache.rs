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
    #[allow(dead_code)]
    pub email: Option<String>,
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

pub fn find_team<'a>(
    teams: &'a [CachedTeam],
    key_or_name: &str,
) -> Result<&'a CachedTeam, LinearError> {
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
    // Also normalize dots/underscores to spaces for fuzzy matching (e.g., "sam.volin" -> "sam volin")
    let needle_normalized = needle.replace(['.', '_'], " ");

    // Helper: check if any of a user's fields match the needle
    let matches = |u: &CachedUser| -> bool {
        let fields: Vec<String> = [
            u.display_name.as_deref(),
            u.name.as_deref(),
            u.email.as_deref(),
        ]
        .iter()
        .filter_map(|f| f.map(|s| s.to_lowercase()))
        .collect();

        for field in &fields {
            if field == &needle || field.contains(&needle) {
                return true;
            }
            // Also try normalized needle (dots/underscores as spaces)
            if needle_normalized != needle {
                let field_normalized = field.replace(['.', '_'], " ");
                if field_normalized == needle_normalized
                    || field_normalized.contains(&needle_normalized)
                {
                    return true;
                }
            }
        }
        false
    };

    users
        .iter()
        .find(|u| matches(u))
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

pub fn find_labels(labels: &[CachedLabel], names: &[&str]) -> Result<Vec<String>, LinearError> {
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
            CachedTeam {
                id: "t1".into(),
                name: "Engineering".into(),
                key: "ENG".into(),
            },
            CachedTeam {
                id: "t2".into(),
                name: "Design".into(),
                key: "DES".into(),
            },
        ]
    }

    fn test_users() -> Vec<CachedUser> {
        vec![
            CachedUser {
                id: "u1".into(),
                name: Some("Alice Smith".into()),
                display_name: Some("Alice".into()),
                email: Some("alice@example.com".into()),
                active: true,
            },
            CachedUser {
                id: "u2".into(),
                name: Some("Bob Jones".into()),
                display_name: None,
                email: Some("bob.jones@example.com".into()),
                active: true,
            },
            CachedUser {
                id: "u3".into(),
                name: Some("Sam Volin".into()),
                display_name: Some("Sam Volin".into()),
                email: Some("sam.volin@example.com".into()),
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
    fn test_find_user_by_email() {
        let users = test_users();
        let user = find_user(&users, "alice@example.com").unwrap();
        assert_eq!(user.id, "u1");
    }

    #[test]
    fn test_find_user_by_email_partial() {
        let users = test_users();
        let user = find_user(&users, "sam.volin").unwrap();
        assert_eq!(user.id, "u3");
    }

    #[test]
    fn test_find_user_dot_normalized() {
        let users = test_users();
        // "sam.volin" should match "Sam Volin" via dot-to-space normalization
        let user = find_user(&users, "sam.volin").unwrap();
        assert_eq!(user.id, "u3");
    }

    #[test]
    fn test_find_user_case_insensitive() {
        let users = test_users();
        let user = find_user(&users, "SAM VOLIN").unwrap();
        assert_eq!(user.id, "u3");
    }

    #[test]
    fn test_find_labels_all_found() {
        let labels = vec![
            CachedLabel {
                id: "l1".into(),
                name: "bug".into(),
            },
            CachedLabel {
                id: "l2".into(),
                name: "feature".into(),
            },
        ];
        let ids = find_labels(&labels, &["bug", "feature"]).unwrap();
        assert_eq!(ids, vec!["l1", "l2"]);
    }

    #[test]
    fn test_find_labels_one_missing() {
        let labels = vec![CachedLabel {
            id: "l1".into(),
            name: "bug".into(),
        }];
        let result = find_labels(&labels, &["bug", "nope"]);
        assert!(result.is_err());
    }
}
