use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StoryType {
    Feature,
    Bug,
    Refactor,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StoryStatus {
    Todo,
    InProgress,
    Done,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
    Grooming,
    Planning,
    Decomposition,
    Implementation,
    Testing,
    Review,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Design,
    Test,
    Code,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Pending,
    Qa,
    Running,
    Paused,
    Blocked,
    Done,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QaStage {
    Grooming,
    Planning,
    TaskQa,
    Implementation,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QaRoundStatus {
    Active,
    Superseded,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContainerMode {
    Project,
    Dev,
    Standalone,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrgRole {
    Owner,
    Admin,
    Member,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeCategory {
    Convention,
    Adr,
    ApiDoc,
    DesignSystem,
    Custom,
}
