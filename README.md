# Lazy

> Lazy evaluation in Rust.

## Example

```rust
fn expensive() -> i32 {
    println!("I am only evaluated once!"); 7
}

fn main() {
    let a = lazy!(expensive());

    // Thunks are just smart pointers!
    assert_eq!(*a, 7); // "I am only evaluated once." is printed here

    let b = [*a, *a]; // Nothing is printed.
    assert_eq!(b, [7, 7]);
}
```

## API

> `lazy!($expr)`

Expands to `Thunk::new(|:| { $expr })`

> `Thunk::new(|:| -> T)`

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
`sync_lazy!` or by doing `use lazy::SyncThunk as Thunk` and using `lazy!`.

