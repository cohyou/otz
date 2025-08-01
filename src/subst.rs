use std::{collections::HashMap, rc::Rc};

use crate::{id::VarId, term::{Term, TermInner}};

impl Term {
    pub fn substitute(&self, substs: HashMap<VarId, Rc<TermInner>>) -> Term {
        let mut result = self.inner.clone();
        for (var, term) in substs {
            result = self.substitute_inner(var, term);
        }
        Term { context: self.context.clone(), inner: result }
    }

    fn substitute_inner(&self, var: VarId, term: Rc<TermInner>) -> Rc<TermInner> {
        match term.as_ref() {
            TermInner::Var(_) => term,
            TermInner::Fun(oper_id, args) => {
                Rc::new(TermInner::Fun(oper_id.clone(), args.iter()
                .map(|arg| {
                    self.substitute_inner(var.clone(), arg.clone())
                }).collect()))
            }
            _ => term,
        }
    }
}

#[test]
fn test_substitute() {
    use crate::id::TypeId;
    use crate::r#type::Type;
    use crate::context::Context;
    use crate::term::Term;
    use std::rc::Rc;

    let mut context = HashMap::new();
    context.insert(VarId(0), Type::Unary(TypeId(0)));
    let context = Context(context);
    let inner = TermInner::Var(VarId(0));
    let term = Term { context, inner: Rc::new(inner) };

    let mut substs = HashMap::new();
    let v = Rc::new(TermInner::Int(100));
    substs.insert(VarId(0), v);
    
    let result = term.substitute(substs);
    dbg!(result);
}