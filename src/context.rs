use crate::r#type::Type;

type List<T> = Vec<T>;
#[derive(Debug, PartialEq, Clone)]
pub struct Ctxt(pub List<Type>);
