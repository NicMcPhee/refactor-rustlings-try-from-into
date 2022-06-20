# Refactoring: Rust and the Rustlings `try-from-into` exercise  🦀 <!-- omit in toc -->

In [episode 6](https://www.youtube.com/watch?v=c63p3TDRwtQ) of
my _Unhindered by Coding_ livestream
([Twitch](https://twitch.tv/NicMcPhee), [YouTube archive](https://www.youtube.com/channel/UC5tGIQti2UYfCSI9aUeSZFQ)),
I flailed pretty hard on the `try-from-into` exercise, spending
over half an hour and still not getting the thing to work.

Coming back to it on my own later, I think the big problem was
that I lost my discipline and was hacking more than thinking,
hoping that I could stumble my way onto a solution. After some
sleep and reflection, I was able to work through it in a
fairly straightforward fashion. In doing so, I realized that
there were some nice refactoring opportunities in this little
bit of code, and I thought I'd document that experience.

- [What's the problem?](#whats-the-problem)
- [Conversion: Its many risks and challenges](#conversion-its-many-risks-and-challenges)
- [How does Rust handle this kind of conversion?](#how-does-rust-handle-this-kind-of-conversion)
  - [`try_from()` and `Result`](#try_from-and-result)
  - [`match`ing three `Result`s](#matching-three-results)
- [The actual Rustlings exercise](#the-actual-rustlings-exercise)
  - [The `TryFrom` trait](#the-tryfrom-trait)
  - [Handling a triple as input](#handling-a-triple-as-input)

## What's the problem?

[The Rustlings `try-from-into` exericse](https://github.com/rust-lang/rustlings/blob/main/exercises/conversions/try_from_into.rs)
is essentially three different versions of constructing an
RGB color struct from three integer values for red, green, and
blue. The `Color` struct is a collection of three (named) `u8`
(unsigned 8-bit) values:

```rust
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}
```

To complete the exercise, we need to write three functions
that each construct a `Color` given three `i16` (signed 16-bit)
values, such as:

```rust
    fn try_from(red: i16, green: i16, blue: i16) -> Color {
        // ... 
    }
```

> We don't actually return a `Color`, because we can't
> always (safely) convert `i16`s to `u8`s, but more on that
> shortly.

The three functions differ in how they receive the three `i16`
values. One takes them as a *tuple*, one takes them as an *array*,
and one takes them as a *slice* (more on that later). To simplify
the initial discussion, though, I'll focus on a version similar
to the stub above, where the three color components are passed in
as separate arguments. Once we've finished beating the basics of
`i16` to `u8` conversion to death, we'll switch to the versions
required by the actual exercise.

## Conversion: Its many risks and challenges

The key issue is that the integer values we're given
are _signed_ 16 bit values (`i16` in Rust), but the values
in the `Color` structure are _unsigned_ 8 bit values
(`u8` in Rust).

A simple "solution", for example, would be something
like:

```rust
fn try_from(r: i16, g: i16, b: i16) -> Color {
    Color { red: r, green: g, blue: b }
}
```

While something like that might work in many languages
(looking at you, C), or might generate runtime exceptions
in others, it won't even compile in Rust. We get three
errors (one for each color component) of the form:

```text
error[E0308]: mismatched types
  --> src/main.rs:36:18
   |
36 |     Color { red: r, green: g, blue: b }
   |                  ^ expected `u8`, found `i16`
```

We thus have to do something to convert each color component
from a signed
16-bit values to an unsigned 8-bit values. This, however, risks
losing information or changing its interpretation.
If the initial value is between 0 and 255
we're good, but if it's negative or greater than 255, we'll
end up throwing away or reinterpreting information in a way
that is almost
certainly not what we want. For example, converting -1 (as
`i16`) to `u8` will result in 255, and converting 300 (as
`i16`) to `u8` will result in 44.

> A similar problem was a key part of what led to the failure
> of the maiden launch of the Ariane 5 rocket, which cost
> more than US$370 million. There were several data
> conversions from 64-bit floating point numbers to 16-bit
> integer values, and "the programmers had protected only
> four out of seven critical variables against overflow".
> ([Wikipedia](https://en.wikipedia.org/wiki/Ariane_flight_V88))
>
> This led to to a freak-out in the guidance system and the
> rocket started to break up, and self-destruct was initiated.
> Luckily there were no people on board, but it still caused
> the loss of an expensive scientific satellite, and scattered
> debris across French Guiana.
>
> Oops. Not a great resume builder.

## How does Rust handle this kind of conversion?

A language like C would happily convert from an signed 16-bit
value to an unsigned 8-bit value. Rust will also do it with the
`as` construct:

```rust
let i : i16 = 300;
let u : u8 = i as u8;
```

Unlike C, however, Rust requires that you _explicitly ask for the
conversion_ with `as u8`, presumably recognizing the risk in
doing so. This approach also makes it clear to future readers
that a potentially risky conversion is happening.

### `try_from()` and `Result`

Even cooler, though, is Rust's `try_from()` which can be used
to _attempt_ conversions, but gracefully handle issues if/when
they arise. The `try_from()` function returns a Rust `Result`
type, which is an enumeration with two variants: `Ok` and `Err`.
We use `Ok` to wrap (hold) the value of a successful conversion,
or use `Err` to wrap (hold) an error value indicating what
when wrong when the conversion failed.

In the `i16` to `u8` conversion problem, for example, we could
do something like:

```rust
    let i: i16 = 300;
    let u_result: Result<u8, TryFromIntError> = u8::try_from(i);
    match u_result {
        Ok(u) => println!("{} as i16 to {} as u8", i, u),
        Err(e) => println!("Got an <{}> error!", e),
    }
```

which prints:

```text
Got an <out of range integral type conversion attempted> error!
```

Here the `u8::try_from(i)` says we want to _try_ to convert `i`
(which has type `i16`) to `u8`. This returns a

```rust
Result<u8, TryFromIntError>
```

type that either wraps a `u8` in `Ok` if the conversion succeeded,
or wraps an `TryFromIntError` in `Err` if there was a problem
doing the conversion. In our example above, we use `match` to see
which of those cases occurred, and in this instance there was an
error because `300` doesn't fit in a `u8`.

### `match`ing three `Result`s

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

Rather than list out all 8 cases in a `match` block, we can, we
can check for all `OK()` (putting all three results in a tuple
so we can treat them as a single expression), and then use the
"default" option `_` to match any other combination of `Ok()`s
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

## The actual Rustlings exercise

Now that we've worked that out, let's move on to the actual
Rustlings exercises. They use the `Color` and `IntoColorError`
structs as defined above. Instead of independent an independent
function like I wrote above, we'll need to implement the
`TryFrom` trait for `Color` for each of the desired input
forms:

- A tuple of three color components
- An array of three color components
- A slice containing the color components

If we do this, for example, then we can make calls like

```rust
    let color_result = Color::try_from((183, 65, 14));
```

to convert from a tuple of color components to a
`Result<Color, IntoColorError>`, and from that we can extract
`Color` structs when the conversions are all successful.

### The `TryFrom` trait

If we look at the stub for the first of the three implementations
required in the exercise we can see how the `TryFrom` is being
used.:

```rust
impl TryFrom<(i16, i16, i16)> for Color {
    type Error = IntoColorError;
    fn try_from(tuple: (i16, i16, i16)) -> Result<Color, IntoColorError> {
        // ...
    }
}
```



### Handling a triple as input
