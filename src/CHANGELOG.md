# 0.2.0 "In Bloom"

## Major Changes

* Primitives are now more consistently handled, allowing them to be reassigned
  like any other function object. Prior to these enhancements, primitives 
  would only be used for calls to specificly named symbols.

  ```r
  f <- paste
  f("a", c("b", "c"))
  # [1] "a b" "a c"
  ```

* A call stack with proper call frames is now added, paving the way for 
  improved error messages and the implementation of R's metaprogramming tools. 

  You can view the call stack by introducing a `callstack()` call:

  ```r
  f <- function(...) list(..., callstack())
  f("Hello, World!")
  # [[1]]
  # [1] "Hello, World!"
  # 
  # [[2]]
  # [[2]][[1]]
  # f("Hello, World!")
  # 
  # [[2]][[2]]
  # list(..., callstack())
  # 
  # [[2]][[3]]
  # callstack()
  ```

* Even more primitives now implemented! This release brings `paste()` and 
  `callstack()` (akin to R's `sys.calls()`)

## Behind the Scenes

* Primitives are now _all_ implemented as `dyn Primitive` objects, implementing
  a `Callable` trait. They still don't have a proper standard library namespace, 
  and are discovered only if not found in the environment (or its parents), 
  but this paves the way for treating primitives more like user-defined 
  functions.

## Thanks to our new contributors!

@armenic (#16)

# 0.1.0 "Why Not?"

Initial release.
