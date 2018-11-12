use ast;
use idtree;
use typing::{Type, TyCon, TyVar};
use {Vector, Result, Error};
use scoped_map::ScopedMap;
use std::rc::Rc;
use std::collections::HashMap;
use fresh_id;
use utils::{Graph, SCC};

struct TopLevelFunc(u32);

pub struct Rename {
    names: ScopedMap<String, idtree::Symbol>,
    //Store uniq names across all scopes to reduce memory.
    // FIXME: Is this even worth it?
    uniq_names: HashMap<String, Rc<String>>, 
    call_ref_graph: Graph<u32, idtree::Symbol>,
    top_level_funcs:  HashMap<u32, TopLevelFunc>
}

impl ::Pass for Rename {
    type Input  = Vec<ast::Module>; //A list of parsed files
    type Output = Vec<idtree::Module>;

    fn run(mut self, mod_vec: Self::Input) -> Result<Self::Output> {
        let result = Vector::map(&mod_vec, |module| self.conv_module(module));
        let sorted = SCC::run(&self.call_ref_graph);
        println!("{:#?}", sorted);
        Ok(result?)
    }
}

impl Default for Rename {
    fn default() -> Self {
        Self::new()
    }
}

impl Rename {
    pub fn new() -> Self {
        Rename{
            names: ScopedMap::new(),
            uniq_names: HashMap::new(),
            call_ref_graph: Graph::new(),
            top_level_funcs: HashMap::new(),
        }
    }
    
    fn conv_ty(&mut self, ty: &ast::Type) -> Result<Type> {
        use ast::Type::*;
        let ty = match *ty {
            Var(ref _v)           => self.new_tyvar(),
            Con(ref nm, ref kind) => {
                let tycon = match nm.as_str() {
                    "i32"  => TyCon::I32,
                    "bool" => TyCon::Bool,
                    "()"   => TyCon::Unit,
                    "->"   => TyCon::Func,
                    _      => TyCon::NewType(self.mk_tycon(nm))
                };
                Type::Con(tycon, kind.clone())
            }
            App(ref con, ref args) => {
                let con = self.conv_ty(con)?;
                let args = Vector::map( args, |arg| self.conv_ty(arg))?;
                Type::App(Box::new(con), args)
            }
        };
        Ok(ty)
    }

    fn add_uniq_name(&mut self, nm: &String) -> Rc<String>{
        self.uniq_names
            .entry(nm.clone())
            .or_insert_with(|| Rc::new(nm.clone()))
            .clone()
    }
        
    fn mk_tycon(&mut self, nm: &String) -> Rc<String> {
        self.add_uniq_name(nm)
    }

    fn add_top_level(&mut self, sym: &idtree::Symbol) -> TopLevelFunc {
        let vertex_key = self.call_ref_graph.add_vertex(sym.clone());
        TopLevelFunc(vertex_key)
    }
    
    fn add_sym(&mut self, nm: &String, ty: Type) -> Result<idtree::Symbol> {
        let var_name = self.add_uniq_name(nm);
        let sym      = idtree::Symbol::new(var_name, ty, fresh_id());
        if let Some(_) = self.names.insert(nm.clone(), sym.clone()) {
            //Allow duplicates at everywhere except the top level
            if self.names.scope() == 0 {
                let msg = Error::new(format!("Name {} already declared", nm));
                return Err(msg)
            }
        }
        Ok(sym)
    }
    
    fn conv_module(&mut self, module: &ast::Module) -> Result<idtree::Module> {
        let decls = Vector::map(module.decls(), |decl| {
            self.conv_decl(decl)
        });
        Ok(idtree::Module::new(module.name().clone(), decls?))
    }

    fn new_tyvar(&self) -> Type {
        let level = self.names.scope();
        Type::Var(TyVar::fresh(level))
    }

    fn conv_decl(&mut self, decl: &ast::Decl) -> Result<idtree::Decl> {
        use ast::Decl::*;
        let res = match *decl {
            Extern(ref name, ref ty) => {
                let ty     = self.conv_ty(ty)?;
                let funcid = self.add_sym(name, ty)?;
                self.add_top_level(&funcid);
                idtree::Decl::Extern(funcid)
            }
            Func(ref bind)           => {
                let ast::Bind(ref name, ref expr) = *bind;
                let ty   = self.new_tyvar();
                let sym  = self.add_sym(name, ty)?;
                self.add_top_level(&sym);
                let expr = self.conv(expr, &sym)?;
                let bind = idtree::Bind::new(sym, expr);
                idtree::Decl::Let(vec![bind])
            }
        };
        Ok(res)
    }

    fn conv(
        &mut self,
        expr: &ast::Expr,
        func: &idtree::Symbol
    ) -> Result<idtree::Expr> {
        use ast::Expr::*;
        let res = match *expr {
            UnitLit      => idtree::Expr::UnitLit,
            I32Lit(n)    => idtree::Expr::I32Lit(n),
            BoolLit(b)   => idtree::Expr::BoolLit(b),
            Lam(ref lam) => {
                self.names.begin_scope();
                let params = Vector::map(lam.params(), |p| {
                    let tv = self.new_tyvar();
                    self.add_sym(p, tv)
                })?;
                let body   = self.conv(lam.body(), func)?;
                self.names.end_scope();
                idtree::Expr::Lam(params, Box::new(body))
            }
            If(ref e)    => {
                let if_expr = idtree::If::new(
                    self.conv(e.cond(), func)?,
                    self.conv(e.texpr(), func)?,
                    self.conv(e.fexpr(), func)?
                );
                idtree::Expr::If(Box::new(if_expr))
            }
            App(ref callee, ref args) => {
                let callee = Box::new(self.conv(callee, func)?);
                let args   = Vector::map(args, |arg| self.conv(arg, func))?;
                idtree::Expr::App(callee, args)
            }
            Var(ref nm) => {
                let sym = match self.names.get(nm) {
                    Some(v) => v,
                    None => {
                        let msg = format!("Could not find variable {}", nm);
                        return Err(Error::new(msg))
                    }
                };
                let v1 = self.top_level_funcs.get(&func.id());
                let v2 = self.top_level_funcs.get(&sym.id());
                if let (Some(v1), Some(v2)) = (v1, v2) {
                    self.call_ref_graph.add_edge(v2.0, v1.0);
                }
                idtree::Expr::Var(sym.clone())
            }
            Let(ref bind, ref let_expr) => {
                let ast::Bind(ref name, ref bind_expr) = **bind;
                let ty    = self.new_tyvar();
                //Convert the bound expression before adding the bound symbol
                let bexpr = self.conv(bind_expr, &func)?;
                let sym   = self.add_sym(name, ty)?;
                let bind  = idtree::Bind::new(sym, bexpr);

                let expr  = self.conv(let_expr, func)?;
                let let_  = idtree::Let::new(bind, expr);
                idtree::Expr::Let(Box::new(let_))
            }
        };
        Ok(res)
    }
}
