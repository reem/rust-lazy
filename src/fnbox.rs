//! Boxed FnOnce trait objects

/// Trait to wrap to build `FnOnce() -> A` trait objects
///
/// This works by only allowing to call the trait objects, when they are boxed.
/// In this way they become object-safe.
///
/// It has been adapted from an unstable part of the standard library and
/// specialized to 0-arity functions for the use case relevant for this crate.
/// (NOTE: an implementation generic over arity would require an additional
/// unstable feature `unboxed_closures`.)
pub trait FnBox {
    /// Return type of the boxed function
    type Output;

    /// Call the boxed function
    fn call_box(self: Box<Self>, args: ()) -> Self::Output;
}

impl<A, F: FnOnce() -> A> FnBox for F
{
    type Output = A;

    fn call_box(self: Box<F>, _: ()) -> F::Output {
        (*self)()
    }
}
