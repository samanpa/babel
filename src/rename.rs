use ast::*;
use {Result,Error};
use scoped_map::ScopedMap;
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Clone,Debug)]
pub struct Var {
    name: Rc<String>,
    ty:   Type,
    id:   u32,
}

pub struct Rename {
    count: u32,
    names: ScopedMap<String, Var>,
    //Store uniq names across all scopes to reduce memory.
    // FIXME: Is this even worth it?
    uniq_names: HashMap<String, Rc<String>>, 
}

impl ::Pass for Rename {
    type Input  = Vec<TopLevel<String>>; //A list of parsed files
    type Output = Vec<TopLevel<Var>>;

    fn run(&mut self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        let mut result = Vec::new();
        for toplevel in &toplevel_vec {
            result.push(self.rename_toplevel(toplevel)?);
        }
        Ok(result)
    }
}

impl Rename {
    pub fn new() -> Self {
        Rename{count: 0,
               names: ScopedMap::new(),
               uniq_names: HashMap::new()
        }
    }

    fn add_var(&mut self, name: String, ty: Type) -> Result<Var> {
        let interned_name = self.uniq_names
            .entry(name.clone())
            .or_insert(Rc::new(name.clone()))
            .clone();
        let var = Var{ name: interned_name, ty, id: self.count };
        if let Some(ref nm) = self.names.insert(name.clone(), var.clone()) {
            return Err(Error::new(format!("Name {} already declared", name)));
        }
        self.count = self.count + 1;
        Ok(var)
    }

    fn rename_toplevel(&mut self, toplevel: &TopLevel<String>)
                       -> Result<TopLevel<Var>> {
        let mut decls = Vec::new();
        for decl in toplevel.decls() {
            decls.push(self.rename_topdecl(decl)?);
        }
        Ok(TopLevel::new(decls))
    }
    
    fn rename_topdecl(&mut self
                      , decl: &TopDecl<String>) -> Result<TopDecl<Var>> {
        use ast::TopDecl::*;
        let res = match *decl {
            Extern{ref name, ref ty} => {
                let var = self.add_var(name.clone(), ty.clone())?;
                Extern{name: var, ty: ty.clone() }
            }
            Lam(ref lam)  => Lam(Box::new(self.rename_lam(&mut &**lam)?)),
            Use{ref name} => Use{name: name.clone()}
        };
        Ok(res)
    }

    fn rename_param(&mut self, param: &Param<String>) -> Result<Param<Var>> {
        let var = self.add_var(param.name().clone(), param.ty().clone())?;
        Ok(Param::new(var, param.ty().clone()))
    }

    fn rename_lam(&mut self, lam: &Lam<String>) -> Result<Lam<Var>> {
        let func_ty = Type::BaseType(BaseType::Unit); //Fixme
        let func_nm = self.add_var(lam.name().clone(), func_ty)?;
        self.names.begin_scope();
        let mut params = Vec::new();
        for param in lam.params() {
            params.push(self.rename_param(param)?)
        }
        let body = self.rename(lam.body())?;
        let lam = Lam::new(func_nm, params, lam.return_ty().clone(), body);
        self.names.end_scope();
        Ok(lam)
    }
    
    fn rename(&mut self, expr: &Expr<String>) -> Result<Expr<Var>> {
        use ast::Expr::*;
        let res = match *expr {
            UnitLit      => UnitLit,
            I32Lit(n)    => I32Lit(n),
            BoolLit(b)   => BoolLit(b),
            If(ref e)    => {
                use ast::*;
                let if_expr = If::new(self.rename(e.cond())?,
                                           self.rename(e.texpr())?,
                                           self.rename(e.fexpr())?);
                If(Box::new(if_expr))
            }
            App{ref callee, ref args} => {
                let callee = Box::new(self.rename(callee)?);
                let mut args_renamed  = Vec::new();
                for arg in args {
                    args_renamed.push(self.rename(arg)?);
                }
                App{callee, args: args_renamed}                
            }
            Var(ref nm) => {
                match self.names.get(nm) {
                    Some(v) => Var(v.clone()),
                    None => {
                        let msg = format!("Could not find variable {}", nm);
                        return Err(Error::new(msg))
                    }
                }
            }
            ref expr    => {
                println!("RENAME: NOTHANDLED\n{:?} not handled", expr);
                unimplemented!()
            },
        };
        Ok(res)
    }
}
