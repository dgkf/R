# R

_An experimental implementation of R, with embellishments_

Check out the [live demo](https://dgkf.github.io/R/)

## What can it do?

```sh
cargo run
```
```r
# R version 0.3.0 -- "Days of Abandon"

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

## What's different?

This project is not just a rewrite of R, but a playground for features and
reinterpretations. It is not meant to reimplement a compatible R layer, but 
to rethink some of R's assumptions. 

To start, there are a few superficial changes:

```r
# 'fn' keyword
f <- fn(a, b, c) {
  a + b + c
}

# vector syntax
v <- [1, 2, 3, 4]

# list syntax
l <- (a = 1, b = 2, c = 3)

# lowercase keywords
kws <- (na, null, inf, true, false)
```

## Why

This project is primarily a personal exploration into language design. 

At the outset, many of the choices are researched one-by-one and are almost
certainly naive implementations. My goal is to learn and explore, and in 
that way the project is already a success in my eyes. Beyond advancing my own
understanding of language internals, I'd love to see the project garner enough
interest to become self-sustaining. 

If you see value in the project for anything beyond prototyping ideas, then
pushing the project toward something practical is contingent on your support.
Contributions, suggestions, feedback and testing are all appreciated.

### Values

Being primarily a one-person project, the values currently map closely to my
own. Somethings I want to aim for:

- A reasonably approachable language for R users (possibly with the ability to
  interpret R code).
- Improved R constructs for complex calls, including argument packing and
  unpacking, partial function calls, destructuring assignment
- Guardrails on non-standard-evaluation, allowing for user-facing 
  domain-specific-languages, while allowing a more rigid evaluation scheme
  internally. 
- Lean into the things that `rust` does well, such as threading, arguably 
  async evaluation, first-class data structures and algebraic error types.
- Learn from more general languages like `TypeScript` to better understand
  how static typing can be comfortably embedded in a high-level language.

## Contribution Guide

If you also want to learn some `rust` or want to explore language design with
me, I'm happy to have you along for the ride. There are plenty of ways to
contribute. In order of increasing complexity, this might include:

- Documenting internals
- Improving documentation throughout
- Helping to improve the demo page hosted on GitHub pages
- Implementing new language concepts
- Providing feedback on internals

Any and all contributions are appreciated, and you'll earn yourself a mention
in release notes!

## License

I welcome other contributors, but also have not thoughtfully selected a long-
term license yet. For now there's a CLA in place so that the license can
be altered later on. I don't intend to keep it around forever. If you have
suggestions or considerations for selecting an appropriate license, your
feedback would be much appreciated.

My current preference is toward a copyleft license like GPL as opposed to a
permissive license like MIT, as I believe that languages are a best-case
candidate for such licenses and it fits well with the ethos of the R community
as being scientific-community first. If you disagree strongly with that
decision, now is your time to let me know.
