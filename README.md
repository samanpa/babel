Babel is an exercise to understand expressive type systems. The goal is to
build an expressive, call by value programming language which does not have a
runtime system.

### Design goals
 - Keep the language/compiler simple
 - Keep semantics clear
 - Be as UNORIGINAL as possible

### Features
 - [x] static typing
 - [x] parametric polymorphism
 - [x] type inference
 - [x] value restriction
 - [x] monomorphization
 - [x] C based ABI
 - [x] C interoperability
 - [ ] Records
 - [ ] constrained parametric polymorphism (type classes)
 - [ ] affine types
 - [ ] references
 - [ ] pattern matching
 - [ ] functors
 - [ ] metaprgramming


### Unsupported
 - Mutually recursive functions that are not at the top level. 
   In theory we can do dependency analysis o detect mutual recursion and then 
   group the mutually recursive functions. Creating the groups can reorder
   side effects and so is not safe to do in an impure language.

### Where I diverge from literature
 - Type constructors and expr applications have arity this is to allow
   uncurrying to work.