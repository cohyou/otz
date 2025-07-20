use crate::r#type::Type;

type List<T> = Vec<T>;
#[derive(PartialEq, Clone)]
pub struct Ctxt(pub List<Type>);

impl std::fmt::Debug for Ctxt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, ty) in self.0.iter().enumerate() {
            if i > 0 {
                let _ = write!(f, ", ");
            }
            let _ = write!(f, "Var{:?}: {:?}", i, ty);
        }
        write!(f, "")
    }
}
