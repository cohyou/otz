use crate::{equation::Equation, instance::Instance, rule::Rule};

impl Instance {
    pub fn complete(&self) -> Vec<Rule> {
        // TODO: 本来はSchemaのconstraintsも必要
        self.data.iter().map(Equation::to_rule).collect()
    }
}