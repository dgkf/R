# R

_An experimental implementation of R_

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

First and foremost, to learn. 

I've always been interested in language design. I know `R` well and think it's a
super expressive language, so it felt like a good target to shoot for. Like any
long-time user of a language, I also have dreamt of how the language could be
improved. This project also offered a small testing ground for a few of those.

## Long-term Goals

As to not mislead anyone, I want to be upfront in saying that this project is
well beyond what I can achieve alone. 

For this project to mature, it is going to need a community of contributors
with diverse expertise. I welcome anyone interested to help out, and I'm happy
to find an intersection of interests as we hash out what the language aims to
deliver.

That said, my personal ambitions for any spiritual successor to R would be:

- Built with `R` code as a first-class input. Even if the language evolves past
`R`, I'd like for it to be able to leverage `R`'s package ecosystem.
- Reimagine many recent `R` language features without the confines of backwards
compatibility.
- Take Jan Vitek's analysis of R's performance to heart and bake in constructs
for isolating non-standard evaluation (though admittedly performance is a
distant goal at the moment).
- Leverage things that `rust` excels at, like its strong iterator support,
async/multithread execution and its error model.

## License

I welcome other contributors, but also have not thoughtfully selected a long-
term license yet. For now there's a CLA in place so that the  license can
be altered later on. I don't intend to keep it around forever.  If you have
suggestions or considerations for selecting an appropriate  license, your
feedback would be much appreciated.

My current preference is toward a copyleft license like GPL as opposed to a
permissive license like MIT, as I believe that languages are a best-case
candidate for such licenses and it fits well with the ethos of the R community
as being scientific-community first. If you disagree strongly with that
decision, now is your time to let me know.
