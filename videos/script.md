# Refactoring: Rust and the Rustlings `try-from-into` exercise  🦀 <!-- omit in toc -->

> This is the script for the video version of this write-up.

> Open with a title card with the title above, and maybe the refactored
> version of the array solution as a background. Probably make using
> Affinity Designer.

I found that going through the Rustlings `try-from-into` exercise

> Switch to a screen showing that exercise in VS Code.

helped me better understand the Rust `TryFrom`
trait, and provided a nice little refactoring opportunity. In this
video I'll first work through the exercise in a "simplistic" fashion,
and then we'll do some refactoring to clean up solution by removing a
lot of duplicate code.

We'll focus here on the classic
red-green-refactor cycle.

> Switch view to a diagram of red-green-refactor, maybe https://medium.com/@melvinzehl/red-green-refactor-dd1d0abd3e16.

You start with tests that fail
(provided in this case by Rustlings), which is the _red_ state.
You then do the simplest possible things to get the tests to pass,
bringing us to the _green_ state. After you're green, then you
_refactor_, working to improve the code while using the tests
as a backstop to help ensure you don't break the code while
you're trying to improve it.

## What's the problem?

> Have a "What's the problem" transition screen, maybe with the
> problem statement in a faded text in the background.

The Rustlings `try-from-into` exercise has us construct an instance
of a `Color` struct

> Display the Rust definition of the `Color` struct:
>
> ```rust
>
> struct Color {
>     red: u8,
>     green: u8,
>     blue: u8,
> }
> ```

containing three `u8` values, i.e., unsigned 8-bit integers, representing
the three RGB values of that color. Our goal is to construct a `Color`
struct from three `i16` values, i.e., signed 16-bit integers, and we have
to do it three different ways.

> Display the three input types: tuple, array, and slice. Might have
> them on the left with arrows to the `Color` struct on the right.

To complete the exercise, we need to write three `try_from()`
functions that each construct a `Color` for a different way
of collecting three `i16` values.

> Add the three signatures to the relevant place in the diagram:
>
> * try_from(rgb: (i16, i16, i16)) -> Color
> * try_from(rgb: [i16; 3]) -> Color
> * try_from(rgb: &[i16]) -> Color

Before we solve the whole problem, however, we need to talk for a second
about the sub-problem of converting from `i16` to `u8`.

## Conversion: Its many risks and challenges

> Add a sub-title slide, perhaps with an exploding rocket faint in the
> background.
> <https://www.researchgate.net/profile/James-Armstrong-7/publication/336109913/figure/fig3/AS:929578147139586@1598640109962/Ariane-5-explosion-retrieved-from.jpg>

The key issue is that the integer values we're given
are _signed_ 16-bit values (`i16` in Rust), but the values
in the `Color` structure are _unsigned_ 8-bit values
(`u8` in Rust).

> Image showing 16 bits being squashed down into 8 bits. Also include
> the range of legal values: -2^15..(2^15-1) and 0..(2^8-1).

Even ignoring the signed/unsigned differences, it's clear that
we have a problem squishing 16 bits into 8.

A similar problem was a key part of what led to the explosive failure
of the maiden launch of the Ariane 5 rocket

> Image of rocket exploding <https://www.researchgate.net/profile/James-Armstrong-7/publication/336109913/figure/fig3/AS:929578147139586@1598640109962/Ariane-5-explosion-retrieved-from.jpg>

a loss which cost
more than US$370 million. There were several data
conversions from 64-bit floating point numbers to 16-bit
integer values, and (according to Wikipedia)
"the programmers had protected only
four out of seven critical variables against overflow".

This led to to a freak-out in the guidance system, the
rocket started to break up, and self-destruct was initiated.
Luckily there were no people on board, but it still caused
the loss of an expensive scientific satellite, and scattered
debris across French Guiana.

Oops. Not a great resume builder.

Returning to our problem, we could just try letting Rust do the
squishing for us, with something like this

> ```rust
>
> fn try_from(r: i16, g: i16, b: i16) -> Color {
>     Color { red: r, green: g, blue: b }
> }
> ```

hoping that Rust will somehow magically convert the `i16` values to
`u8`s. This might appear to "work" in languages like C, but will lead
to strange and confusing results (like rocket explosions) when the `i16`
values don't naturally fit in a `u8` slot.

Rust, however, won't even let you do it, and we get this sort of error
from the compiler:

> ```text
> error[E0308]: mismatched types
>   --> src/main.rs:36:18
>    |
> 36 |     Color { red: r, green: g, blue: b }
>    |                  ^ expected `u8`, found `i16`
> ```

This is telling us that the Rust compiler "knows" that it can't convert
an `i16` value to `u8` with potentially losing data, and that it's going to
force us to make the hard choices about how to handle this.

Rust does give us a way of "blindly" converting and hoping for the best
using the `as` operator.

As we can see here in the Rust Playground, this code without any explicit
conversion fails

> ```rust
> fn main() {
>     let i : i16 = 300;
>     let u : u8 = i;
>     println!("Converting {i} as i16 to u8 resulted in {u}.")
> }
> ```

but if we add `as u8` to the second line

> ```rust
> fn main() {
>     let i : i16 = 300;
>     let u : u8 = i as u8;
>     println!("Converting {i} as i16 to u8 resulted in {u}.")
> }
> ```

then the code compiles and runs, but we are also obviously throwing away
data in the process as 300 as an `i16` converts to 44 as a `u8`!

## Enter the `TryFrom` trait and `Result`

> Switch to transition slide with this title and perhaps the `TryFrom` trait
> faint in the background.
> <https://doc.rust-lang.org/std/convert/trait.TryFrom.html>

A safer option is Rust's `TryFrom` trait. There are numerous implementations
of this trait, each of which provides a `try_from()` function which can be
used to _attempt_ conversions, but gracefully handle issues if/when
they arise.

A `try_from()` call looks like

> `A::try_from(v)`,

where `A` is the _type_ we're trying to convert _to_, and
`v` is the _value_ we're trying to convert _from_. In our
simple example we might having something like:

> Go to VSCode now.

> ```rust
> fn main() {
>     let i : i16 = 300;
>     let u : u8 = i;
>     println!("Converting {i} as i16 to u8 resulted in {u}.")
> }
> ```

> Go from `let u : u8 = i` to `try_from(i)` to `<u8>::try_from(i)`.
> Show that this works, but returns `Err(TryFromIntError(()))`
> from the `Result` type. Also show that if we change it to 30
> we get an `Ok(30)`.
>
> We then use a `match` clause to get the converted value.
>
> ```rust
>     let i: i16 = 300;
>     let u_result = u8::try_from(i);
>     match u_result {
>         Ok(u) => println!("{i} as i16 to {u} as u8"),
>         Err(e) => println!("Got an <{e}> error!"),
>     }
> ```

### `match`ing three `Result`s

> Title slide with the finished code for the simplified version
> as the background.

:warning: This is where I left off

Given this, a plausible start to our simplified version would
be something like:

```rust
fn try_from(r: i16, g: i16, b: i16) -> Result<Color, IntoColorError> {
    let red = u8::try_from(r);
    let green = u8::try_from(g);
    let blue = u8::try_from(b);
    // ...
}
```

This converts each of the input values from `i16` to `u8`, giving
us three `Result` values, one for each color component.

We've had to change the return type of `try_from()` to reflect
the possibility that one or more of the `u8::try_from()` calls
could return an error. Here we're using the `IntoColorError` type
provided in the starter code, which has an `IntConversion` variant
that we can use to indicate when we weren't able to convert from
a `i16` to a `u8`.

```rust
enum IntoColorError {
    // Integer conversion error
    IntConversion,
    // ...
}
```

The problem, then, is how to proceed after these three `try_from()`
calls. There are 8 possible combinations of `Ok()` and `Err()`
outcomes from these three calls (two possibilities for each of the
three calls). If they're all `Ok()` then we want to return an `Ok()`
as well, but if _any_ are `Err()` then we want to return an `Err()`.

Rather than list out all 8 cases in a `match` block, we
can put all three results in a tuple so we can treat them as
a single expression, and then organize things into just
two cases. In the first, we'll check to see if all three
`Result`s are `Ok()`. We'll then use the
default option `_` to match any other combination of `Ok()`s
and `Err()`s.

```rust
fn try_from(r: i16, g: i16, b: i16) -> Result<Color, IntoColorError> {
    let red = u8::try_from(r);
    let green = u8::try_from(g);
    let blue = u8::try_from(b);
    
    match (red, green, blue) {
        (Ok(red), Ok(green), Ok(blue)) => Ok(Color { red, green, blue }),
        _ => Err(IntoColorError::IntConversion),
    }
}
```

And this works!

It doesn't actually solve any of the posed problems, but it
does successfully solve a closely related problem, so we're
heading in a useful direction.

> Another option that would avoid the use of `match` would be to
> use `map_err()` and the `?` operator, but that doesn't work with
> the use of `map` we'll get to later, so we'll stick to using
> `match` for now.
