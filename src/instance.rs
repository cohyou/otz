use crate::{context::Context, equation::Equation, schema::Schema};

#[derive(Default, Clone)]
pub struct Instance {
    pub schema: Schema,
    pub elems: Context,
    pub data: Vec<Equation>,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.data.iter()
            .map(|d| {
                format!("{:?} = {:?}", d.left.clone(), d.right.clone())
            }).collect::<Vec<_>>();

        f.debug_struct("Instance")
            .field("schema", &self.schema)
            .field("elems", &self.elems)
            .field("data", &data)
            .finish()
    }
}
