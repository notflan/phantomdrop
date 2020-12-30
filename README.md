# `PhantomDrop`

Small library for `defer`ing the running of function until the end of a block.

# Usage
It is similar to `marker` types, useful for adding destructors into structs that you don't want to implement `Drop` on themselves, and also for deferring function calls.

## Extra destructors
To add a destructor into a struct without needing to implement `Drop` on the struct itself (which can have some of its own issues, such as losing the ability for partial moves), we can use `PhantomDrop` as a field in the structure.
Note that when doing this, since closure types are opaque, the field may need to be sized.

## Deferring
Similar to the `defer` mechanism in Go, we can use this to defer the calling of functions
 ```rust
fn do_something()
{
  let _guard = phantomdrop::defer(|| println!("Hello!"));
  // do some work
} // "Hello!" will now be printed when the function returns or unwinds (unless unwinds are disabled).
 ```

# Holding data

The guard can also hold a value
 ```rust
fn do_something(print: String)
{
 let _guard = PhantomDrop::new(print, |string| println!("Dropped: {}", string));
 // do some work
} // `print` will now be printed here.
 ```

## Capturing

We can also capture a value, by reference, mutable reference, or moving. 
Both holding a value within the guard and a capturing closure can be used at the same time.
 ```rust
fn do_something(print: String)
{
 let _guard = phantomdrop::defer(move || println!("Dropped: {}", print)); // Moves `print` into itself.
 // do some work
} // `print` will now be printed here.

fn do_something_by_reference(print: String)
{
 let _guard = phantomdrop::defer(|| println!("Dropped: {}", print)); // Holds an immutable reference to `print`.
 let trimmed = print.trim(); // Can still be used
} // `print` will now be printed here.

fn do_something_by_mutable_reference(print: &mut String)
{
 let _guard = phantomdrop::defer(|| *print = String::from("Dropped")); // Holds a mutable reference to `print`.
} // `print` will now be set to "Dropped" here.
 ```

# License
GPL'd with <3
