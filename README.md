# Lazy

> Lazy evaluation in Rust.

## Example

```rust
fn expensive() -> uint { println!("I am only evaluated once!"); 7u }

fn main() {
    let a = lazy!(expensive());

    // Thunks act like smart pointers!
    assert_eq!(*a, 7u); // "I am only evaluated once." is printed here

    let b = [*a, *a]; // Nothing is printed.
    assert_eq!(b, [7u, 7u]);
}
```

## API

> `lazy!($expr)`

Expands to `Thunk::new(proc() { $expr })`

> `Thunk::new(proc() -> T)`

Takes a proc, creates a delayed computation.

> `Thunk::force()`

Forces the evaluation of the thunk so subsequent accesses are cheap. Values are
stored unboxed.

> `Thunk::unwrap()`

Consumes and forces the evaluation of the thunk and returns the contained
value.

> `Thunk::deref()` / `Thunk::deref_mut()`

Gets the value out of the thunk by evaluating the proc or grabbing it
from the cache. Allows you to call methods on the thunk as if it was
an instance of the contained valued through auto-deref.

There is also an equivalent API for `SyncThunk`, which is `Sync + Send` and
usable for safe, concurrent laziness, except that they are created using
`sync_lazy!` or by doing `use Thunk = lazy::SyncThunk` and using `lazy!`.

