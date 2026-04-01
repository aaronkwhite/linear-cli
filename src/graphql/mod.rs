#[cynic::schema("linear")]
pub mod schema {}

/// Custom scalar types from Linear's GraphQL schema.
/// These must be declared so cynic knows how to handle them.
pub mod scalars {
    use super::schema;

    /// ISO 8601 datetime string
    #[derive(cynic::Scalar, Debug, Clone)]
    #[cynic(graphql_type = "DateTime")]
    pub struct DateTime(pub String);

    /// Timeless date string (YYYY-MM-DD)
    #[derive(cynic::Scalar, Debug, Clone)]
    #[cynic(graphql_type = "TimelessDate")]
    pub struct TimelessDate(pub String);

    /// Arbitrary JSON object
    #[derive(cynic::Scalar, Debug, Clone)]
    #[cynic(graphql_type = "JSONObject")]
    pub struct JSONObject(pub String);

    /// JSON value
    #[derive(cynic::Scalar, Debug, Clone)]
    #[cynic(graphql_type = "JSON")]
    pub struct JSON(pub String);

    /// Duration string
    #[derive(cynic::Scalar, Debug, Clone)]
    #[cynic(graphql_type = "Duration")]
    pub struct Duration(pub String);

    /// UUID string
    #[derive(cynic::Scalar, Debug, Clone)]
    #[cynic(graphql_type = "UUID")]
    pub struct UUID(pub String);

    /// DateTime or Duration value
    #[derive(cynic::Scalar, Debug, Clone)]
    #[cynic(graphql_type = "DateTimeOrDuration")]
    pub struct DateTimeOrDuration(pub String);

    /// TimelessDate or Duration value
    #[derive(cynic::Scalar, Debug, Clone)]
    #[cynic(graphql_type = "TimelessDateOrDuration")]
    pub struct TimelessDateOrDuration(pub String);
}

pub mod issues;
