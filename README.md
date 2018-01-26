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
 - [ ] C based ABI
 - [ ] C interoperability
 - [ ] constrained parametric polymorphism (type classes)
 - [ ] affine types
 - [ ] references
 - [ ] pattern matching
 - [ ] functors
 - [ ] metaprgramming


### Where I diverge from literature
 - Type constructors and expr applications have arity this is to allow
   uncurrying to work.