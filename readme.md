# cargo docgen - a Rust Documentation Test Helper

## Rationale

[Documentation tests](https://doc.rust-lang.org/book/first-edition/testing.html#documentation-tests)
are an essential part of releasing good Rust crates on [crates.io](https::/crates.io). To quote the
first edition of the Rust book:

> Nothing is better than documentation with examples.
> Nothing is worse than examples that don't actually work

Every item (module, function, method, etc) should have an example which both _compiles_ and
_runs as a test_.

However, if you mosey to that most excellent resource [docs.rs](https://docs.rs) and browse some of
the 11,000-odd crates, you will see that many don't even try to provide any documentation, which
is disappointimg and leaves you with the irritating necessity of _actually reading the source_.
Part of this is just human nature, or at least the nature of programmers who find it difficult
to switch from code to English, but much of it is that good documentation is _hard work_.
Not only is formatting doc tests tiresome, but running `cargo test` to run all the tests
can take a fair amount of time even for small projects.

The [Guidelines](https://rust-lang-nursery.github.io/api-guidelines/documentation.html) for
documentation are very comprehensive and fairly demanding. `cargo docgen` aims to make preparing
working tests and embedding them in your source easier.

## A Simple Example

Say you wish to publish your great work, the crate `life`. You wish to document the function
`life::answer`. Write a little code snippet like so in some subdirectory of the `life` project
(I personally create a `scratch` dir and put it in `.gitignore`)

```rust
// answer.rs
let a = life::answer();
assert_eq!(a, 42);
```

And run `cargo docgen`:

```
$ cargo docgen answer.rs
****** Copy and paste this output into your code ******

/// ```
/// let a = life::answer();
/// assert_eq!(a, 42);
/// ```
```

It will run this snippet using `cargo run --example` and comment the result appropriately.
You can type the doc test in a real editor, run it immediately, and
have something that can be pasted directly into your code.  (I don't know about other people, but
I like typing Rust in a code-aware editor, and I do not like waiting
to find out if I have inevitable mistakes.)

This comment is suitable for any code item which is not module-level. If I said `cargo docgen -m answer.rs`,
the result is formatted for a module-level example:

```rust
//! ```
//! let a = life::answer();
//! assert_eq!(a, 42);
//! ```
```

You can indent the result using `--indent`. I tend to say `-i4` because I like spaces, but `-i1t` will
indent by one tab, and so forth. (Mixing spaces and tabs is an Abomination.)

## Support for the Question Mark Operator

Consider this snippet which I wrote to test [lua-patterns](https://docs.rs/lua-patterns):

```rust
let mut pat = lua_patterns::LuaPattern::new_try("^%s*$").unwrap();
assert!(pat.matches("  "));
```
It's common to see `unwrap` in little examples, and it is both nasty and misleading, because
in well-written code, it hardly appears. In real life, we use the question mark operator for
error handling. As the Guidelines say: "Like it or not, example code is often copied verbatim
by users. Unwrapping an error should be a conscious decision that the user needs to make."

This is the purpose of the `--question` flag (`-q` for short.)

So you should write:

```rust
let mut pat = lua_patterns::LuaPattern::new_try("^%s*$")?;
assert!(pat.matches("  "));
```

And `cargo docgen -q -i4 new_try.rs` will generate the following code:

```rust
    /// ```
    /// # use std::error::Error;
    /// #
    /// # fn run() -> Result<(),Box<Error>> {
    /// let mut pat = lua_patterns::LuaPattern::new_try("^%s*$")?;
    /// assert!(pat.matches("  "));
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #    run().unwrap();
    /// # }
    /// ```
```

This is the recommended way to present code where
errors may occur, and it's a lot of boilerplate.  The doc test syntax allows for
lines to be hidden using `#`, so only the actual snippet lines will appear in the rendered
documentation.
(Here we're using the convenient fact that _any_ `Error` type will convert into a `Box<Error>`)

Compiling and running this snippet took 1.2s - `cargo test` for the whole project took 14.7s in clock time!
And it would take far longer, and be more painful, to enter the full commented code directly
into the library source.

## Examples which are Not Tests

A doc test (like any other Rust test) consists of a set of assertions. You may use
`println!` but the test runner will swallow this output. `cargo docgen` _will_ print out
the output, but will issue a warning.

Some examples should be compiled, but _not_ run. Here we process an example of obviously
bad test code from The Book, First Edition:

```rust
$ cat loop.rs
loop {
    println!("Hello, world");
}
$ cargo docgen -n loop.rs
****** Copy and paste this into your code ******

/// ```rust,no_run
/// loop {
///   println!("hello, world");
/// }
/// ```
```

## Testing and Formatting Markdown

The `--module-doc` (`-M`) flag lets you process a _whole Markdown file_ containing little doc test
snippets. Here is a silly example:

> This should be any text
> whatsoever which can be edited safely. Snippets are only
> run if they change:
>
> ```rust?
> use lua_patterns::*;
> let mut pat = LuaPattern::new_try("^%s*$")?;
> assert!(pat.matches("  "));
> assert!(! pat.matches(" x "));
> ```
> and the text continues.
>
> This shows how by default matches are 'unanchored':
>
> ```rust
> let mut pat = lua_patterns::LuaPattern::new("boo");
> assert!( pat.matches("boo") );
> assert!( pat.matches("  boo ") );
> ```
>
> And another:
> ```rust
> for i in 0..4 {
>     println!("gotcha! {}",i);
> }
> ```

This is almost the Github-flavoured Markdown that we know and love, with one little change.
If a doc test uses the question-operator, `cargo codegen` needs to know so it can
generate the necessary boilerplate. Since reliably detecting `?` in source is tricky
(it could be in a comment, or in a string) I've opted for an explicit approach, where
the usual guard "```rust" becomes "```rust?".

Running `cargo docgen -M doc.md` gives, after _running each snippet_:

```
//! This should be any text
//! whatsoever which can be edited safely. Snippets are only
//! run if they change:
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn run() -> Result<(),Box<Error>> {
//! use lua_patterns::*;
//! let mut pat = LuaPattern::new_try("^%s*$")?;
//! assert!(pat.matches("  "));
//! assert!(! pat.matches(" x "));
//! # Ok(())
//! # }
//! #
//! # fn main() {
//! #    run().unwrap();
//! # }
//! ```
//! and the text continues.
//!
//! This shows how by default matches are 'unanchored':
//!
//! ```
//! let mut pat = lua_patterns::LuaPattern::new("boo");
//! assert!( pat.matches("boo") );
//! assert!( pat.matches("  boo ") );
//! ```
//!
//! And another:
//! ```
//! for i in 0..4 {
//!     println!("gotcha! {}",i);
//! }
//! ```
//!
```

Furthermore, these code snippets are cached (look in 'doc.md.cache' afterwards)
and subsequent runs will _only_ re-run those doc tests which have in fact
changed.

Good Rust document tests are hard to _type_, and I hope this utility makes it easier
for other lazy people to write better, functional documentation for their crates.
To install, just use `cargo install cargo-docgen`.






