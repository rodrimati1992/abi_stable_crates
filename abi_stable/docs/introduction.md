
# Passing opaque values around with `VirtualWrapper<_>`

One can pass non-StableAbi types around by using type erasure,
using the `VirtualWrapper<_>` wrapper type.

The type generally looks like `VirtualWrapper<Pointer<OpaqueType<Interface>>>`,where:

- Pointer is some `pointer_trait::StableDeref` pointer type.

- OpaqueType is a zero-sized marker type.

- Interface is an `InterfaceType`,which describes what traits are 
    required when constructing the `VirtualWrapper<_>` and which ones it implements.

`trait InterfaceType` allows describing which traits are required 
when constructing a `VirtualWrapper<_>`,and which ones it implements.

`VirtualWrapper<_>` can be used as a trait object for a selected ammount of traits:

### Construction

To construct a `VirtualWrapper<_>` one can use these associated functions:
    
- from_value:
    Can be constructed from the value directly.
    Requires a value that has an associated `InterfaceType`.
    
- from_ptr:
    Can be constructed from a pointer of a value.
    Requires a value that has an associated `InterfaceType`.
    
- from_any_value:
    Can be constructed from the value directly.Requires a `'static` value.
    
- from_any_ptr
    Can be constructed from a pointer of a value.Requires a `'static` value.

### Trait object

`VirtualWrapper<_>` can be used as a trait object for a selected ammount of traits:

- Clone 

- Display 

- Debug 

- Default: Can be called as an inherent method.

- Eq 

- PartialEq 

- Ord 

- PartialOrd 

- Hash 

- serde::Deserialize:
    first deserializes from a string,and then calls the objects' Deserialize impl.

- serde::Serialize:
    first calls the objects' Deserialize impl,then serializes that as a string.

### Deconstruction

`VirtualWrapper<_>` can then be unwrapped into a concrete type using these 
(fallible) conversion methods:

- `into_unerased`:
    Unwraps into a pointer to `T`.
    Where the `VirtualWrapper<_>`'s interface must equal `<T as ImplType>::Interface`

- `as_unerased`:
    Unwraps into a `&T`.
    Where the `VirtualWrapper<_>`'s interface must equal `<T as ImplType>::Interface`

- `as_unerased_mut`:
    Unwraps into a `&mut T`.
    Where the `VirtualWrapper<_>`'s interface must equal `<T as ImplType>::Interface`

- `into_mut_unerased`:Unwraps into a pointer to `T`.Requires `T:'static`.

- `as_mut_unerased`:Unwraps into a `&T`.Requires `T:'static`.

- `as_mut_unerased_mut`:Unwraps into a `&mut T`.Requires `T:'static`.

