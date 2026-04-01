use thiserror::Error;

#[derive(Error, Debug)]
pub enum LinearError {
    #[error("GraphQL error: {0}")]
    GraphQL(String),

    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },

    #[error(
        "LINEAR_API_KEY not found.\n\
         Set it via environment variable or .env file.\n\
         Get your key from: Linear Settings > API > Personal API keys"
    )]
    NoApiKey,

    #[error("{entity} not found: {name}")]
    NotFound { entity: &'static str, name: String },

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
