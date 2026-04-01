use super::schema;

#[derive(cynic::QueryVariables, Debug)]
pub struct IssueByIdVariables {
    pub id: String,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
#[cynic(graphql_type = "Query", variables = "IssueByIdVariables")]
pub struct IssueById {
    #[arguments(id: $id)]
    pub issue: Issue,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct Issue {
    pub id: cynic::Id,
    pub identifier: String,
    pub title: String,
    /// Priority: 0 = No priority, 1 = Urgent, 2 = High, 3 = Medium, 4 = Low
    pub priority: f64,
}
