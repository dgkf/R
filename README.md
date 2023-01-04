# R

An experimental implementation of R

## What can it do?

```sh
cargo run
```
```r
# R version 0.0.1 -- "Why Not?"

x <- function(a = 1, ...) { a + c(...) }
# function(a = 1, ...) {
#   a + c(...)
# }
# <environment 0x6000005b8e28>

y <- function(...) x(...)
# function(...) x(...)
# <environment 0x6000005b8e28>

y(4, 3, 2, 1)
# [1] 7 6 5 
```

This amounts to (most) of R's grammar parsing, basic primitives, scope
management and ellipsis argument passing.

## Why

First and foremost, to learn - at this point, it is far too early  to fixate on
other goals.

Although these are distant goals, if the project ever builds any sort of
momentum, I'd like for it to grow into a project that is the foundation of a
successor language for R -- providing tooling to either port pure R code or
provide a separate mode of operating to execute R code.

## License

I welcome other contributors, but also have not thoughtfully selected a long-
term license yet. For now there's a CLA in place so that the  license can
be altered later on. I don't intend to keep it around forever.  If you have
suggestions or considerations for selecting an appropriate  license, your
feedback would be much appreciated.
