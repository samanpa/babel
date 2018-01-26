use xir;
use typing::types::Type;
use monoir;
use {Result,Vector,Error};

pub struct Uncurry {}

impl Default for Uncurry{
    fn default() -> Self {
        Self::new()
    }
}

impl ::Pass for Uncurry {
    type Input  = Vec<xir::Module>;
    type Output = Vec<monoir::Module>;

    fn run(mut self, module_vec: Self::Input) -> Result<Self::Output> {
        let res = Vector::map(&module_vec, |modl| self.uncurry_module(modl))?;
        Ok(res)
    }
}

impl Uncurry {
    pub fn new() -> Self {
        Uncurry{}
    }

    fn func(&self, var: &xir::TermVar, expr: &xir::Expr)
            -> Result<monoir::Func>
    {
        panic!("FSAF {:?}", var);
    }
    
    fn uncurry_module(&self, module: &xir::Module)
                      -> Result<monoir::Module>
    {
        let modname = module.name().clone();

        for decl in module.decls() {
            match *decl {
                xir::Decl::Extern(_, _) => (),
                xir::Decl::Let(ref id, ref expr) => {
                    println!("{:?} = \n{:?}\n", id.name(), expr);
                }
            }
        }

        for decl in module.decls() {
            match *decl {
                xir::Decl::Extern(_, _) => (),
                xir::Decl::Let(ref id, ref expr) => {
                    self.func(id, expr);
                }
            }
        }

        //let decls   = Vec::new();
        Ok(monoir::Module::new(modname))
    }
}

/*
fn mk_func(param: &Vec<Type>, ret: Type) -> Type {
    use self::Type::*;
    let mk_fn = |ret, param: &Type| App(Box::new(mk_tycon("->"))
                                        , vec![param.clone(), ret]);
    
    let itr = param.into_iter().rev();
    match itr.len() {
        0 => mk_fn(ret, &mk_tycon("unit")),
            _ => itr.fold(ret, mk_fn),
    }
}

    
fn monoir_fnty(ty: &xir::Type, mut nparams: n) -> Result<monoir::Type> {
    let params = Vec::with_capacity(n);
    let set_i  = |i, ty| unsafe{params.unchecked_set(i, monoir_type(ty)) }
    let mut itr = ty;
    while n > 0 {
        match *itr => {
            Type::App(ref 
        
    }
fn monoir_type(ty: &xir::Type) -> Result<monoir::Type> {
    match *ty {
        Type::Con( ref tyname) => {
            match *tyname => {
                "i32"  => monoir::Type::i32,
                "bool" => monoir::Type::Bool,
                "unit" => monoir::Type::Unit,
                _      => panic!("Not supported"),
            },
        }
        Type::App(_, _) => {
            panic!("App Not supported"),
        }
        Type::Var(_) => {
            panic!("Var not suppored"),
        }
    }                    
}
    
*/
