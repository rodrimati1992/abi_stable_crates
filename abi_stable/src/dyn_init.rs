/*!

Module for dynamic copying between structs using a runtime representation of the layout.


*/



/**

# Safety

This can cause undefined behavior if all these conditions about the types are true:

- They are syntactically the same (they are written exactly identically in the source code).

- They have the same alignement.

- They have the same size.

# Answers to possible questions

Question:Why don't you use a ::std::any::TypeId to check that they are the same type?

Answer:Because while they may be logically the same type,
they may come from different compilations of the same library,
causing their TypeIds to be a different value.

* Question *:Why not use generics to implement this more safely?

* Answer *:Because I want to keep compile-times down,
using generics would not make it significantly faster anyway,
since this function is only called when libraries are initialized.


*/
fn dynamic_copy()
