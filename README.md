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

## What's the problem?

[The Rustlings `try-from-into` exericse](https://github.com/rust-lang/rustlings/blob/main/exercises/conversions/try_from_into.rs)
is essentially three different versions of constructing an
RGB color struct from three integer values for red, green, and
blue. The key issue is that the integer values are _signed_
16 bit values (`i16` in Rust), but the values in the `Color`
structure are _unsigned_ 8 bit values (`u8` in Rust).

:bangbang: I think I should explain more of the actual exercise
here before wandering off into conversions.

We thus have to convert each color component from a signed
16-bit values to an unsigned 8-bit values, but this risks
losing information. If the initial value is between 0 and 255
we're good, but if it's negative or greater than 255, we'll
end up throwing away information in a way that is almost
certainly not what we want. For example, converting -1 (as
`i16`) to `u8` will result in 255, and converting 300 (as
`i16`) to `u8` will result in 44.

> A related problem was part of what led to the failure
> of the maiden launch of the Ariane 5 rocket, which cost
> more than US$370 million. There were several data
> conversions from 64-bit floating point numbers to 16-bit
> integer values, and "the programmers had protected only
> four out of seven critical variables against overflow".
>
> This led to to a freak-out in the guidance system and the
> rocket started to break up, so self-destruct was initiated.
> Luckily there were no people on board, but it still led to
> the loss of an expensive scientific satellite, and scattered
> debris across French Guiana.
> ([Wikipedia](https://en.wikipedia.org/wiki/Ariane_flight_V88))
>
> Oops.

## How does Rust handle this kind of conversion?

A language like C would happily convert from an signed 16-bit
value to an unsigned 8-bit value. Rust will also do it with the
`as` construct:

```rust
let i : i16 = 300;
let u : u8 = i as u8;
```

Unlike in C, in Rust you have to _explicitly ask for the
conversion_ with `as u8`, presumably recognizing the risk in
doing so. This approach also makes it clear to future readers
that a potentially risky conversion is happening.

Even cooler, though, is Rust's `try_from()` which can be used
to _attempt_ conversions, but gracefully handle issues if/when
they arise. The `try_from()` function returns a Rust `Result`
type, which is an enumeration that can either be `Ok` wrapping
the value of the successful conversion, or `Err` wrapping
an error value indicating what when wrong.

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

The `match` syntax used above is in some ways the most
straightforward way of dealing with a `Result` type, but can
lead to awkward nesting. There are other constructs like
`if let` and `?` that can simplify handling errors, and we'll
see some of that below.
