# R

_An experimental implementation of R, with embellishments_

Check out the [live demo](https://dgkf.github.io/R/)

## What can it do?

```sh
cargo run
```
```r
# R version 0.3.1 -- "Art Smock"

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

### Syntax

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

# destructuring assignment
(a, b) <- (1, 2)
```

There are plenty of more substantial [changes being considered](https://github.com/dgkf/R/issues?q=is%3Aissue+is%3Aopen+label%3Ameta-proposal). 
If you enjoy mulling over the direction of syntax and features, feel
free to join the conversation.

### Experiments

All experiments are feature-gated and enabled by running (or building) with 

```sh
cargo run -- --experiments "<experiment>"
```

Please try them out and share your thoughts in the corresponding issues!

#### Ellipsis packing and unpacking

> [!NOTE]  
> `--experiments rest-args` (discussed in [#48](https://github.com/dgkf/R/issues/48), [#49](https://github.com/dgkf/R/issues/49))

Current work is focused on `..args` named ellipsis arguments and `..args`
unpacking in function calls. However, due to the experimental nature of this
syntax it is currently behind a feature gate.

```r
f <- function(..args) {
  args
}

f(1, 2, 3)  # collect ellipsis args into a named variable
# (1, 2, 3)
```

```r
args <- (a = 1, b = 2, c = 3)
f <- function(a, b, c) {
  a + b + c
}

f(..args)  # unpack lists into arguments
# [1] 6

more_args <- (c = 10)
f(..args, ..more_args)  # duplicate names okay, last instance takes priority
# [1] 13
```

#### Tail Recursion

> [!NOTE]  
> `--experiments tail-calls` (discussed in [#60](https://github.com/dgkf/R/issues/60)) 

Tail recursion allows for arbitrarily recursive call stacks - or, more 
accurately, it discards frames from the call stack in this special case
allowing for recursion without overflowing of the call stack.

```r
f <- function(n) if (n > 0) f(n - 1) else "done"
f(10000)
# [1] "done"
```

The details of how this is achieves requires the tail call's arguments to be
executed _eagerly_ instead of R's typical _lazy_ argument evaluation. This 
change can result in some unexpected behaviors that need discussion before
the feature can be fully introduced.

### Performance

You might be thinking `rust` is fast, and therefore this project must be
fast. Well, unfortunately you'd be wrong. That's probably more of a 
reflection on me than `rust`. To get the basic skeleton in place, 
my focus has been on getting things working, not on getting them working
_well_. For now, expect this interpreter to be about ***1000x*** slower
than R. 

I'm feeling good about the general structure of the internals, but there
have been plenty of quick proofs of concept that involve excess copies, 
extra loops, panics and probably less-than-ideal data structures.
If you're an optimization fiend and you want to help narrow the gap with 
R, your help would be very much appreciated!

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
