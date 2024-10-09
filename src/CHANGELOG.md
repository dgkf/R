# dev

## Changes

* Named vectors were added and can e.g. be constructed via `[a = 1, b = 2]`
* The `is_null()` primitive was added
* Setting a list value to `null` actually sets it to `null` and does not remove it.

## Internals

* The `List` is now represented as a `Rep<Obj>`, unifying heterogenous and atomic vectors.
  This included a considerable refactor.
* Iterating over references of a `Rep<T>` was made much simpler and new methods were added
  and unused ones removed.

## Notable Bugs Addressed

* Concatenating `list`s now works as expected (#177).
* `names()` now works correctly when the vector is subset, e.g.
  `names(list(a = 1, b = 2)[1])` (#181).
* `[[.<-` assignment has been fixed for lists (#179)

# 0.4.0

## Changes

* Added German (`ger`) localization.

* Digits can now be separated by an underscore to improve readability.

* Added assertions on function parameters (#74)

* Added the `length()` primitive (#105)

* Assignment is now also possible using the `=` operator (#144 @sebffischer)

* Added new primtiive function, `substitute()` (#125 @dgkf)

## Internals

* Rename `Numeric` variant of `Vector` enum to `Double`

* Promises now retain their expression even after being evaluated (#125 @dgkf)

* The internal vector representation has been changed to allow for
  in-place modification of vector, e.g. when they materialize themselves
  from their lazy representation:
  What was previously `Rep` is now `RepType` and `Rep` can switch out
  one `RepType` for another. (#127 @sebffischer)

* The internal vector representation has been extended to allow for mutable
  views and lazy copies via the `VecData` type (#127 @sebffischer)

* Added the internal `call_mutable()` method to `Callable`, which for now
  is only implemented to retrieve vectors mutably from environments,
  which drives call-assignment (things like `x[1] <- 2`) (@sebffischer).

* Implemented `IntoIterator` for the Vector `Rep`presentation (@sebffischer).

* Increased the version of `pest` for new clippy warnings:
  https://github.com/pest-parser/pest/issues/1000 (@sebffischer)

* The `Clone` implementation for `Vector` now creates a lazy copy (@sebffischer)

* Use a faster hash function (#138, @sebffischer)


## Notable Bugs Addressed

* Fix accidental tail calls escaping binary operator evaluation (#115 @dgkf)

* Fix history file not being created if one doesn't already exist (@dgkf)

* Vectors are no longer mutated in-place after assignment (#104 @sebffischer)

# 0.3.3 "Beautiful You"

## Changes

* Adding localizations. To start, only English (`en`), Spanish (`es`) and
  Chinese (`es`). Can be    specified using the `--locale` flag when starting
  the REPL.

* Adding new primitive functions, `eval()`, `quote()` and `print()`

* Experiments moved from `rust` "features", which need to be compiled into
  the language runtime, to instead be flags that are set for the session.

# 0.3.2 "Eurydice"

## Changes

* Tracebacks are now available on error.

## Notable Bugs Addressed

* Control flow `if-else`, `repeat`, `while` and `for` have been fixed following
  some bugs introduced by the new `Tail` return type.

## Internals

* Many internal `panic!`s have been further moved to captured errors.

* `Closure`s of `Symbol`s (an approximation for `Promise`s) will now evaluate
  a bit more quickly by avoiding adding frames to the call stack.

* `Closure`s that propegate to a nested call will no longer introduce a new
  wrapping `Closure`, instead propegating the existing `Closure` value and
  avoiding the extra layer of indirection.

# 0.3.1 "Art Smock"

## Major Changes

* License was changed to be `GPL-3`. The contributor license agreement (`CLA`)
  used in the development repository will remain in place with a grace period
  for comment. With it comes a copyright and warranty disclaimer as recommended
  in the `GPL`, which should make long-time R users feel right at home.

* Added simplest-case destructuring assignment.

  ```r
  (x, y) <- list(a = 1, b = 2)
  x
  # [1] 1
  y
  # [1] 2
  ```

* `return` keyword introduced (this is unlike R's `return()` primitive, and
  this might change back to using a `return()` primitive in the future)

## Experiments

This release introduces "experiments", which are feature-gated behaviors. This
release introduces two to start:

* `tail-call-optimization`, when enabled, will handle tail calls without
  extending the call stack, but with the possibly unexpected behavior of
  eagerly evaluating arguments to the call.

* `rest-args`, when enabled, introduces the ability to name ellipsis arguments
  and to unpack lists into function calls.

## Notable Bugs Addressed

* `for` loop off-by-one error corrected.

## Internals

* Many `panic!`s where replaced with proper errors.

* Introduced the start of destructuring assignment.

* Added `Tail` and `Return` `Signal` variants, which are used to raise returns
  back to the calling frame (the calling function). `Return` is used to return
  values and `Tail` is used to return the tail expression for potential tail
  call optimization.

# 0.3.0 "Days of Abandon"

## Major Changes

* Vector indexed assignment (`x[1:3] <- 10`) is now supported! What's more,
  it avoids intermediate replications of the vector by keeping track of the
  indexing operation as a "view" of the vector.

  ```r
  x <- 1:10
  x[2:8][2:5][[2]] <- 1000
  x
  # [1]    1    2    3 1000    5    6    7    8    9   10
  ```

* Mutating assignment implemented for `List`s, including by named index.

  ```r
  x <- list(a = 1, b = 2, c = 3, d = 4, e = 5)
  x[2:3][[1]] <- 200
  x[1:4][c("d", "c")] <- 1000
  x
  # list(a = 1, b = 200, c = 1000, d = 1000, e = 5)
  ```

## Internals

* "altreps" are now supported internally, though currently only a "Subset"
  (used for indexed assignment) is implemented.

* `List`s were reworked to use a `HashMap` of named values, allowing for
  more immediate access to named values without repeated traversals of a
  vector of pairs.


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

## Internals

* Primitives are now _all_ implemented as `dyn Primitive` objects, implementing
  a `Callable` trait. They still don't have a proper standard library namespace,
  and are discovered only if not found in the environment (or its parents),
  but this paves the way for treating primitives more like user-defined
  functions.

## Thanks to our new contributors!

@armenic (#16)

# 0.1.0 "Why Not?"

Initial release.
