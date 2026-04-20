use std::collections::BTreeMap;

use crate::{
    AxisKind, AxisSpec, ObservationSpec, ParameterSpec, RandomVarSpec, ReductionSpec,
    SimulationSpec, StateVarSpec, StepSpec,
};

#[derive(Debug, Clone)]
pub struct SimulationSpecBuilder {
    spec: SimulationSpec,
}

impl SimulationSpecBuilder {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            spec: SimulationSpec {
                schema_version: crate::SUPPORTED_SCHEMA_VERSION.to_string(),
                name: name.into(),
                version: version.into(),
                parameters: Vec::new(),
                axes: BTreeMap::new(),
                random_variables: Vec::new(),
                state_variables: Vec::new(),
                steps: Vec::new(),
                observations: Vec::new(),
                reductions: Vec::new(),
            },
        }
    }

    pub fn schema_version(mut self, schema_version: impl Into<String>) -> Self {
        self.spec.schema_version = schema_version.into();
        self
    }

    pub fn parameter(mut self, name: impl Into<String>, dtype: impl Into<String>) -> Self {
        self.spec.parameters.push(ParameterSpec {
            name: name.into(),
            dtype: dtype.into(),
        });
        self
    }

    pub fn axis(
        mut self,
        name: impl Into<String>,
        kind: AxisKind,
        size: Option<usize>,
        parallel: bool,
        ordered: bool,
    ) -> Self {
        let name = name.into();
        self.spec.axes.insert(
            name.clone(),
            AxisSpec {
                name,
                kind,
                size,
                parallel,
                ordered,
            },
        );
        self
    }

    pub fn random_variable(mut self, random_variable: RandomVarSpec) -> Self {
        self.spec.random_variables.push(random_variable);
        self
    }

    pub fn state_variable(mut self, state_variable: StateVarSpec) -> Self {
        self.spec.state_variables.push(state_variable);
        self
    }

    pub fn step(mut self, step: StepSpec) -> Self {
        self.spec.steps.push(step);
        self
    }

    pub fn observation(mut self, observation: ObservationSpec) -> Self {
        self.spec.observations.push(observation);
        self
    }

    pub fn reduction(mut self, reduction: ReductionSpec) -> Self {
        self.spec.reductions.push(reduction);
        self
    }

    pub fn build(self) -> SimulationSpec {
        self.spec
    }
}
