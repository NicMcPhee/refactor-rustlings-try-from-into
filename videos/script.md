# Refactoring: Rust and the Rustlings `try-from-into` exercise  🦀 <!-- omit in toc -->

- [What's the problem?](#whats-the-problem)
- [Conversion: Its many risks and challenges](#conversion-its-many-risks-and-challenges)
- [Enter the `TryFrom` trait and `Result`](#enter-the-tryfrom-trait-and-result)
  - [`match`ing three `Result`s](#matching-three-results)
- [The actual Rustlings exercise](#the-actual-rustlings-exercise)
  - [The `TryFrom` trait](#the-tryfrom-trait)
  - [Handling a tuple as input](#handling-a-tuple-as-input)
  - [Handling a slice as input](#handling-a-slice-as-input)
- [Let's refactor this puppy](#lets-refactor-this-puppy)
  - [Handle tuples using arrays](#handle-tuples-using-arrays)
  - [Handle slices using arrays](#handle-slices-using-arrays)
  - [Mapping over the array elements](#mapping-over-the-array-elements)
- [Wrap-up](#wrap-up)

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

> Go to VSCode now? Maybe not – maybe stay in Playground until
> we get to the actual exercise code.

> ```rust
> fn main() {
>     let i : i16 = 300;
>     let u : u8 = i;
>     println!("Converting {i} as i16 to u8 resulted in {u}.")
> }
> ```
>
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

## The actual Rustlings exercise

> Title slide with `IntoColorError` as the background

Now that we've worked out the basics of converting `i16` to `u8`,
let's move on to the actual Rustlings exercises.

They use the `Color` struct as defined above, and an extended version of the
`IntoColorError` type:

> ```rust
> struct Color {
>     red: u8,
>     green: u8,
>     blue: u8,
> }
> 
> enum IntoColorError {
>     IntConversion, // Integer conversion error
>     BadLen,        // Slice argument with the incorrect length
> }
> ```

We'll use the `IntConversion` variant when we have a problem converting from
`i16` to `u8`. We'll see why we need the `BadLen` variant a little later.

The Rustlings exercise asks us to implement the `TryFrom` trait for `Color`
for each of three desired input forms:

> - A tuple of three color components
> - An array of three color components
> - A slice containing the color components

As an example, this will allow us to make calls like

> Add this to the previous slide
> ```rust
> let color_result: Result<Color, IntoColorError> = Color::try_from([183, 65, 14]);
> ```
> Add arrow that indicates that we're converting from the array to
> the `Result` type.

to convert from an array of color components to a
`Result<Color, IntoColorError>`, and from that we can extract
`Color` structs when the conversions are all successful.

### The `TryFrom` trait

> Title slide with the `TryFrom` stub in the background

The exercise gives us stubs for all three instances of the `TryFrom`
trait that we need to implement. Here, for example, is the stub for
converting from a tuple:

> ```rust
> impl TryFrom<(i16, i16, i16)> for Color {
>     type Error = IntoColorError;
>     fn try_from(tuple: (i16, i16, i16)) -> Result<Color, IntoColorError> {
>         // ...
>     }
> }
> ```

This implementation of the `TryFrom` trait tells Rust how to convert _from_
a tuple of three `i16` values into a `Color` struct. This will allow us
to make calls like

> Add to previous slide
> ```rust
> Color::try_from((183, 65, 14))
> ```

to construct a `Color`, returning a `Result` type to capture the possibility
of an error when performing the conversion.

### Handling a tuple as input

> Title slide that has the unfinished `TryFrom` stub as its background.

So now that we understand the problem, let's implement the first case
where we have a tuple as our input.

> Switch to VSCode screen cast mode here. Start with the stub, with the
> argument highlighted.

> Have `bacon` running in a terminal along the bottom.

Down in the bottom terminal we're using the Rust `bacon` tool to
watch the code and continually rerun the tests whenever we make
changes. At the moment all 14 of the tests are failing because
we haven't implemented any parts of the exercise, so we're
in the "red" phase of the Red-Green-Refactor process.

> Have the typing happen while the following
> voiceover is occurring.

This first case, where we have a tuple as our input, is almost exactly
the same as the implementation of the simplified
function from earlier, namely we convert each color component to a
`u8` and use a `match` clause to check for errors:

```rust
impl TryFrom<(i16, i16, i16)> for Color {
    type Error = IntoColorError;
    fn try_from(tuple: (i16, i16, i16)) -> Result<Color, IntoColorError> {
        let red_result = u8::try_from(tuple.0);
        let green_result = u8::try_from(tuple.1);
        let blue_result = u8::try_from(tuple.2);
        
        match (red_result, green_result, blue_result) {
            (Ok(red), Ok(green), Ok(blue)) => Ok(Color { red, green, blue }),
            _ => Err(IntoColorError::IntConversion),
        }
    }
}
```

This works fine and passes the relevant tests, but I'm not a fan of the
`tuple.0` notation since it's awfully easy to write `tuple.2` instead of
`tuple.1` and not have anyone notice the mistake, either as you're making
it, in a code review, or debugging after the fact.

> Start typing this change concurrent with the following paragraph of
> voiceover.

```rust
impl TryFrom<(i16, i16, i16)> for Color {
    type Error = IntoColorError;
    fn try_from((red, green, blue): (i16, i16, i16)) -> Result<Color, IntoColorError> { 
        let red_result = u8::try_from(red);
        let green_result = u8::try_from(green);
        let blue_result = u8::try_from(blue);
        
        match (red_result, green_result, blue_result) {
            (Ok(red), Ok(green), Ok(blue)) => Ok(Color { red, green, blue }),
            _ => Err(IntoColorError::IntConversion),
        }
    }
}

We can avoid the need for explicitly extracting the tuple components by taking
advantage of Rust's pattern matching in the function argument. This allows
us to give the components more useful names, which we can then use in the
conversion statements.

Not a huge change, but definitely an improvement in readability
and maintainability in my opinion.

### Handling an array as input

> Title slide with the array stub as the background

Now let's move on to handling an array as our input.

> Back to VSCode, scrolled down to the appropriate stub with the argument
> highlighted. Start typing right away while the voiceover continues.

```rust
impl TryFrom<[i16; 3]> for Color {
    type Error = IntoColorError;
    fn try_from([red, green, blue]: [i16; 3]) -> Result<Color, IntoColorError> {
        let red_result = u8::try_from(red);
        let green_result = u8::try_from(green);
        let blue_result = u8::try_from(blue);
        
        match (red_result, green_result, blue_result) {
            (Ok(red), Ok(green), Ok(blue)) => Ok(Color { red, green, blue }),
            _ => Err(IntoColorError::IntConversion),
        }
    }
}
```

The array implementation is almost identical to the tuple
implementation, with the _only_ different being in the function
argument pattern matching used to extract the arguments `red`,
`green`, and `blue`.

We'll actually copy/paste the function body
from the tuple function rather than type all this out again.

The remarkable similarity  between these two implementations _strongly_
suggests a refactoring opportunity, but we'll come to that after we've
implemented the third part of the exercise.

### Handling a slice as input

> Title slide with the slice stub as the background

The third part of the exercise is a bit trickier because the input is an
array slice.

> Back to VSCode, scrolled down to the appropriate stub, highlighting
> the argument.

The issue here that we don't know the length of an array slice. The tuple
and the array inputs in the previous versions were both were guaranteed
to have exactly three elements, but the vector slice in this case could
have any length, from 0 to arbitrarily many elements. So we can't just pattern
match against `red`, `green`, and `blue` like we did before, and will in
fact have to extract the relevant elements from the slice "by hand" using
something like:

> Move to slide with this code
> ```rust
> let red = slice[0];
> let green = slice[1];
> let blue = slice[2];
> ```

There's a problem here, though, which is the slice might not contain
exactly three elements. If it contains fewer than three, then our code
will panic at run time with an error like:

> Add this below the three lines in the previous slide and *read* the error.
> ```text
> thread … panicked at 'index out of bounds: the len is 1 but the index is 2'
> ```

> Slight pause so they can read the error message.

> Then go back to a slide with the `IntoColorError` type:
> ```rust
> enum IntoColorError {
>     IntConversion, // Integer conversion error
>     BadLen,        // Slice argument with the incorrect length
> }
> ```

This is why our `IntoColorError` type has the `BadLen` variant, so we can
return that error in cases where the slice doesn't have the right length
(i.e., 3).

Depending on how we write it, our code might not actually fail
if our input slice has too many elements. You could imagine a specification
where it's fine if the input slice has too many elements; we just use the
first three to construct the `Color` and ignore the rest.

The tests provided with the exercise, however, suggest that we are
expected to return a `BadLen` error in that circumstance as well.

> Go back to VS Code and add the length check during this voiceover.

```rust
    if slice.len() != 3 {
        return Err(IntoColorError::BadLen);
    }
```

So we need to add a length check before we start extracting
and converting values, returning an `Err(IntoColorError::BadLen)` if the
the length isn't three.

Now that we're guaranteed that the slice has the desired length,
we can use the ideas from the previous solutions to finish this
component of the exericse.

> Type in the `let red = slice[0]` lines, and paste in the body below
> that.

```rust
impl TryFrom<&[i16]> for Color {
    type Error = IntoColorError;
    fn try_from(slice: &[i16]) -> Result<Color, IntoColorError> {
        if slice.len() != 3 {
            return Err(IntoColorError::BadLen);
        }

        let red = slice[0];
        let green = slice[1];
        let blue = slice[2];

        let red_result = u8::try_from(red);
        let green_result = u8::try_from(green);
        let blue_result = u8::try_from(blue);
        
        match (red_result, green_result, blue_result) {
            (Ok(red), Ok(green), Ok(blue)) => Ok(Color { red, green, blue }),
            _ => Err(IntoColorError::IntConversion),
        }
    }
}
```

After extracting the relevant elements from the slice, we'll again paste in
the body from the previous solutions, causing the
refactoring opportunity warning bell to go "Ding , Ding, Ding!"
_very_ loudly. 

> Run the tests in VS Code, showing that we've solved the problem.

If we now run the tests, we'll see that have technically solved the problem
as all the tests pass. So we've hit the "Green" part of the "red-green-refactor"
cycle, which is cool!

Along the way, though, we've copy/pasted a substantial block of code, so
there are definitely opportunities for refactoring. So let's do it!

## Let's refactor this puppy

> Title slide with the copied chunk as the background

We know from how we implemented things, there are three copies of the same
substantial block of code.

> Switch to VS Code, scrolled to the array solution, with the body
> highlighted.

The slice version has a lot of slice-specific code, but both the tuple and
the array versions almost exactly capture the shared logic, differing only
in the pattern matching on the function's argument. So either of them could
nicely form the basis of our refactoring.

Looking ahead a little, I'm going to define the first and third versions
(tuple and slice) in terms of the second (array) version. The main reason
for that is that we can map across arrays, but we can't map across tuples.
Tuples can contain elements of different types, which makes mapping impossible
(at least in a strongly typed language like Rust) since we wouldn't know
how to type the function we're mapping across the tuple.

### Handle tuples using arrays

> Title slide with the tuple refactoring as the background

It's pretty easy to rewrite the tuple version so that it uses the array
version.

> Switch to VS Code, and do the refactoring while the voiceover happens.

```rust
impl TryFrom<(i16, i16, i16)> for Color {
    type Error = IntoColorError;
    fn try_from((red, green, blue): (i16, i16, i16)) -> Result<Color, IntoColorError> { 
        Color::try_from([red, green, blue])
    }
}
```

All we need to do is rewrite the input tuple to a array of
three `i16`s and then call the array version.

> Re-run the tests and confirm that they're still green.

And voilà! We've replaced seven lines of code with just a single line and
the tests still pass!

### Handle slices using arrays

> Title slide with the final slice refactoring as the background

The slice version will require a little more work because we'll still
need to check the length and extract the components.

> Switch to VS Code and do the first refactoring along with the voiceover.

```rust
impl TryFrom<&[i16]> for Color {
    type Error = IntoColorError;
    fn try_from(slice: &[i16]) -> Result<Color, IntoColorError> {
        if slice.len() != 3 {
            return Err(IntoColorError::BadLen);
        }

        let red = slice[0];
        let green = slice[1];
        let blue = slice[2];

        Color::try_from([red, green, blue])
    }
}
```

A simple way to refactor the slice version is to keep the length test,
extract the `red`, `green`, and `blue` components, and then call the
array version

> Re-run the tests to confirm that we're still green.

This is an improvement, cutting the number of lines roughly in half.

An alternative, though, would be to use `try_from` in yet another form
that converts slices to arrays:

> Switch to slide with this code:

```rust
    let a = <[i16; 3]>::try_from(slice).unwrap();
```

Here the `<[i16; 3]>::try_from(slice)` call attempts to convert
`slice` into an array of three `i16`s.

This can fail if the given
vector doesn't have the right number of elements, which is why the
`try_from()` call returns a `Result` type. If we do the length check
first, then we'll _know_ that the length of the vector `slice` is 3, so we
can just call `.unwrap()` to extract the value in the `Ok()` variant.
That `unwrap()` call will panic if the slice has the wrong length, but
we're safe because of the length check.

> Switch back to VS Code and type as the voiceover continues.

```rust
impl TryFrom<&[i16]> for Color {
    type Error = IntoColorError;
    fn try_from(slice: &[i16]) -> Result<Color, IntoColorError> {
        if slice.len() != 3 {
            return Err(IntoColorError::BadLen);
        }
        let a = <[i16; 3]>::try_from(slice).unwrap();
        Color::try_from(a)
    }
}
```

We can then use this conversion to further simplify the slice version.

> Confirm that this all passes.

> Switch to slide with this code block:
> ```rust
>         if slice.len() != 3 {
>             return Err(IntoColorError::BadLen);
>         }
>         let a = <[i16; 3]>::try_from(slice).unwrap();
> ```

Since the `Color::try_from()` we're defining already returns a `Result`, we
don't _have_ to avoid returning an error. We also don't even have to
explicitly _check_ for the error. The Rust `?` construct allows us
to extract the `Ok()` value from a `Result` type, while immediately
returning an error if the `Result` is an `Err()` variant instead of an
`Ok()`. So we could _almost_ replace this block with

> Put this below the previous block, highlighting the ? at the end.
> ```rust
>         let a = <[i16; 3]>::try_from(slice)?;
> ```

Here if the `<[16; 3]>::try_from(slice)` call returns an `Ok()` value,
that value will be extracted by the `?` operator and assigned to `a`
and we can happily move on. If `<[16; 3]>::try_from(slice)` returns
an `Err()` variant, however, then the `?` operator will immediately
return that `Err()` and none of the subsequent code will be run.

> Switch back to VS Code and make that change.

This _almost_ works, but as you can see we end up with a compiler error

> Hover over the error so we can see what's happening. Or switch to
> a terminal and run `cargo build` and show the error there.

```text
the trait `From<TryFromSliceError>` is not implemented for `IntoColorError`
```

> Highlight the relevant parts of the code as we go through the voiceover.

What this is telling us is that the `<[i16; 3]>::try_from(slice)` call
returns a `TryFromSliceError` if there's a problem, but our function is
declared as returning `Result<Color, IntoColorError>`. Rust doesn't know
how to convert a `TryFromSliceError` into a `IntoColorError`, hence the
error.

We have a couple of options here. We _could_ actually implement the
`TryFrom` trait for converting a `TryFromSliceError` into a
`IntoColorError`.

> Make this change while the voiceover happens.
> ```rust
>   let a = <[i16; 3]>::try_from(slice).map_err(|_| IntoColorError::BadLen)?;
> ```

A simpler option in this case, though, is to use the `map_err`
function to convert the `TryFromSliceError` into a `IntoColorError`.

Now if the `<[i16; 3]>::try_from(slice)` call returns an `Ok()` variant,
that will be left alone by the `map_err()` call, and the value will
be extracted by the `?` operator.

If the `<[i16; 3]>::try_from(slice)` call returns an `Err()` variant,
however, the error will be mapped by `map_err()` to a
`IntoColorError`, and the `?` operator will immediately return that error.
Note that the use of `_` as the argument to the `map_err()` closure
says we don't care _what_ error type was returned by
`<[i16; 3]>::try_from(slice)`; we'll convert _any_ error type that
it returns into `IntoColorError::BadLen`.

> Re-run the tests.

And now we have refactored this part of the exercise down to two lines of code
and things still work!

After the refactoring our three definitions are collectively 28 lines of code,
versus 49 lines originally, or a reduction of over 40%!

### Mapping over the array elements

> Title slide with the `color_elements.map()` line as the background.

This is pretty nice, but I still find the repetition in this part
of the array implementation annoying:

> Go to VS Code, with this code highlighted. Keep making changes
> as the voiceover continues.
> 
> ```rust
>     let red_result = u8::try_from(red);
>     let green_result = u8::try_from(green);
>     let blue_result = u8::try_from(blue);
> ```

We can avoid this by using `map` to apply `u8::try_from()` to each of the
color elements instead of having to make three separate calls.

```rust
    fn try_from(color_elements: [i16; 3]) -> Result<Color, IntoColorError> {
        let result = color_elements.map(|v| u8::try_from(v));

        match result {
            [Ok(red), Ok(green), Ok(blue)] => Ok(Color { red, green, blue }),
            _ => Err(IntoColorError::IntConversion),
        }
    }
```

This actually runs fine and passes the tests…

> Run the tests and confirm that they pass.

…even though any `Err()` values in `result` will have the "wrong" type,
because they won't be `IntoColorError` values.

In our `match` statement we never explicitly indicate what types we're
expecting in the catch-all `_` class, so as long as that clause returns
the correct error type (`IntoColorError::IntConversion`) then everything
be fine.

If/when the `try_map` method on arrays
moves into the release version of Rust, we could use it to
simplify this further and avoid the `match` clause altogether.

## Wrap-up

> Title slide with big -> small image in background

Our refactored version still works (we're still green), but it's about
half as long as the initial working version!

> Have an image here with the original version on the left and the
> refactored version on the right. Both will have really tiny font
> sizes so they won't be readable, but we'll be able to see the
> difference.

Here our refactoring work has taken us from 49 lines of code down
to 26 lines, which is pretty nifty. Note how valuable it was having
solid tests to provide a safety net while we were doing the refactoring,
so we knew we were in good shape at every stage of the process.

