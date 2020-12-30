//! Small library for `defer`ing the running of function until the end of a block.

use core::mem::MaybeUninit;
use core::ops::Drop;

/// When dropped, the included function is ran with the argument held by the structure.
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
    /// Box the closure in this instance.
    #[inline] pub fn boxed(self) -> PhantomDrop<T, Box<dyn FnOnce(T)>>
    {	
	let (v, f) = unsafe { self.into_raw_parts() };
	PhantomDrop::new(v, Box::new(f))
    }

    /// Replace the function to be ran on drop with a no-op in place.
    #[inline] pub fn neutralise_boxed(&mut self)
    {
	unsafe { self.value_mut().1 = Box::new(drop) };
    }
}
impl<T> PhantomDrop<T, fn (T)>
{
    /// Replace the function to be ran on drop with a no-op in place.
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
