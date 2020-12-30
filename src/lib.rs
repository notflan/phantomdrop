//! Small library for `defer`ing the running of function until the end of a block.
//!
//! # Uasge
//! Similar to the `defer` mechanism in Go, we can use this to defer the calling of functions
//! ```
//! fn do_something()
//! {
//!   let _guard = phantomdrop::defer(|| println!("Hello!"));
//!   // do some work
//! } // "Hello!" will now be printed when the function returns or unwinds (unless unwinds are disabled).
//! ```
//!
//! The guard can also hold a value
//! ```
//! fn do_something(print: String)
//! {
//! # use phantomdrop::PhantomDrop;
//!  let _guard = PhantomDrop::new(print, |string| println!("Dropped: {}", string));
//!  // do some work
//! } // `print` will now be printed here.
//! ```
//!
//! Or capture a value, by reference, mutable reference, or moving.
//! ```
//! fn do_something(print: String)
//! {
//!  let _guard = phantomdrop::defer(move || println!("Dropped: {}", print)); // Moves `print` into itself.
//!  // do some work
//! } // `print` will now be printed here.
//!
//! fn do_something_by_reference(print: String)
//! {
//!  let _guard = phantomdrop::defer(|| println!("Dropped: {}", print)); // Holds an immutable reference to `print`.
//!  let trimmed = print.trim(); // Can still be used
//! } // `print` will now be printed here.
//!
//! fn do_something_by_mutable_reference(print: &mut String)
//! {
//!  let _guard = phantomdrop::defer(|| *print = String::from("Dropped")); // Holds a mutable reference to `print`.
//! } // `print` will now be set to "Dropped" here.
//! ```
use core::mem::MaybeUninit;
use core::ops::Drop;

/// When dropped, the included function is ran with the argument held by the structure.
///
/// # Notes
/// If both the function and the value are zero-sized (unique non-capturing closures are ZSTs), this wrapper will also be zero-sized.
#[derive(Debug)]
pub struct PhantomDrop<T, F: FnOnce(T)>(MaybeUninit<(T, F)>);

impl<T: Clone, F: Clone + FnOnce(T)> Clone for PhantomDrop<T,F>
{
    #[inline] fn clone(&self) -> Self
    {
	let re = unsafe { self.value_ref() };
	Self(MaybeUninit::new((re.0.clone(), re.1.clone())))
    }
}

impl<F> PhantomDrop<(),F>
where F: FnOnce(())
{
    /// Defer a function to run when this guard is dropped.
    #[inline] pub fn defer(fun: F) -> Self
    {
	PhantomDrop::new((), fun)
    }
}

/// Defer this function to run when the returned guard is dropped.
pub fn defer(fun: impl FnOnce()) -> PhantomDrop<(), impl FnOnce(())>
{
    PhantomDrop::defer(move |_| fun())
}

impl<T, F> PhantomDrop<T,F>
where F: FnOnce(T)
{
    #[inline(always)] unsafe fn value_mut(&mut self) -> &mut (T, F)
    {
	&mut (*self.0.as_mut_ptr())
    }
    #[inline(always)] unsafe fn value_ref(&self) -> &(T, F)
    {
	&(*self.0.as_ptr())
    }
    #[inline(always)] unsafe fn into_raw_parts(self) -> (T, F)
    {
	let (v, f) = self.0.as_ptr().read();
	core::mem::forget(self);
	(v, f)
    }
    
    /// Defer a function to run on this stored value when this guard is 
    #[inline] pub fn new(value: T, fun: F) -> Self
    {
	Self(MaybeUninit::new((value, fun)))
    }

    /// Consume the instance into its held type without running the drop closure.
    #[inline] pub fn into_inner(self) -> T
    {
	unsafe { self.into_raw_parts() }.0
    }

    /// Consume this instance without running the drop closure.
    ///
    /// # Notes
    /// This largely has the same behaviour of `core::mem::forget`, however this method is preferable for instances of `PhantomDrop`, as it properly calls destructors for both its value and its function if needed.
    #[inline] pub fn forget(self)
    {
	unsafe { self.into_raw_parts() };
    }

    /// Get a mutable reference to the held type.
    #[inline] pub fn as_mut(&mut self) -> &mut T
    {
	unsafe { &mut self.value_mut().0 }
    }
    /// Get a reference to the held type.
    #[inline] pub fn as_ref(&self) -> &T
    {
	unsafe { &self.value_ref().0 }
    }

    /// Replace the function to be ran on drop with a no-op.
    #[inline] pub fn neutralise(self) -> PhantomDrop<T, fn (T)>
    {
	PhantomDrop::new(self.into_inner(), drop)
    }

}

impl<T: 'static> PhantomDrop<T, Box<dyn FnOnce(T)>>
{
    /// Box the closure in this instance on to the heap.
    #[inline] pub fn boxed(self) -> PhantomDrop<T, Box<dyn FnOnce(T)>>
    {	
	let (v, f) = unsafe { self.into_raw_parts() };
	PhantomDrop::new(v, Box::new(f))
    }

    /// Replace the function to be ran on drop with a no-op in place on the heap.
    #[inline] pub fn neutralise_boxed(&mut self)
    {
	unsafe { self.value_mut().1 = Box::new(drop) };
    }
}
impl<T> PhantomDrop<T, fn (T)>
{
    /// Replace the function to be ran on drop with a no-op in place with no allocations.
    #[inline] pub fn neutralise_in_place(&mut self)
    {
	unsafe { self.value_mut().1 = drop };
    }
}


impl<T, F> Drop for PhantomDrop<T,F>
where F: FnOnce(T)
{
    #[inline] fn drop(&mut self)
    {
	let (v, f) = unsafe { self.0.as_ptr().read() };
	f(v);
    }
}

#[cfg(test)]
mod tests
{
    #[test]
    fn zero_sized()
    {
	let guard = super::defer(|| println!("Hello world!"));
	assert_eq!(core::mem::size_of_val(&guard), 0);
    }
    #[test]
    fn mut_reference_holding()
    {
	let mut hi = String::from("Hello?");
	let _guard = super::PhantomDrop::new(&mut hi, |string| {
	    *string = String::from("Hello!");
	    println!("{}", string);
	});
    }
    #[test]
    fn reference_holding()
    {
	let hi = String::from("Hello!");
	let _guard = super::PhantomDrop::new(&hi, |string| println!("{}", string));
    }
    #[test]
    fn value_holding()
    {
	let hi = String::from("Hello!");
	let _guard = super::PhantomDrop::new(hi, |string| println!("{}", string));
    }
    #[test]
    fn value_capturing()
    {
	let hi = String::from("Hello!");
	let _guard = super::defer(move || println!("{}", hi));
    }
    #[test]
    fn mut_reference_capturing()
    {
	let mut hi = String::from("Hello?");
	let _guard = super::defer(|| {
	    hi = String::from("Hello!");
	    println!("{}", hi)
	});
    }
    #[test]
    fn reference_capturing()
    {
	let hi = String::from("Hello!");
	let _guard = super::defer(|| println!("{}", hi));
    }
    #[test]
    fn deferring()
    {
	let _guard = super::defer(|| println!("Hello!"));
    }
}
