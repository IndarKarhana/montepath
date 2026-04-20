use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationSpec {
    pub name: String,
    pub version: String,
    pub parameters: Vec<ParameterSpec>,
    pub axes: BTreeMap<String, AxisSpec>,
    pub random_variables: Vec<RandomVarSpec>,
    pub state_variables: Vec<StateVarSpec>,
    pub steps: Vec<StepSpec>,
    pub observations: Vec<ObservationSpec>,
    pub reductions: Vec<ReductionSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParameterSpec {
    pub name: String,
    pub dtype: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AxisSpec {
    pub name: String,
    pub kind: AxisKind,
    pub size: Option<usize>,
    pub parallel: bool,
    pub ordered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AxisKind {
    Static,
    Runtime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RandomVarSpec {
    pub name: String,
    pub distribution: String,
    pub dtype: String,
    pub axes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateVarSpec {
    pub name: String,
    pub dtype: String,
    pub init: Expr,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepSpec {
    pub name: String,
    pub axis: String,
    pub updates: Vec<StateUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateUpdate {
    pub target: String,
    pub expr: Expr,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObservationSpec {
    pub name: String,
    pub expr: Expr,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReductionSpec {
    pub name: String,
    pub op: String,
    pub source: String,
    pub axes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Expr {
    Literal { value: f64 },
    ParameterRef { value: String },
    StateRef { value: String },
    RandomRef { value: String },
    UnaryOp { op: String, arg: Box<Expr> },
    BinaryOp { op: String, lhs: Box<Expr>, rhs: Box<Expr> },
    Call { fn_name: String, args: Vec<Expr> },
}
