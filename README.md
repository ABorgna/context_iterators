context_iterators
=========

[![build_status][]](https://github.com/aborgna/context_iterators/actions)
[![crates][]](https://crates.io/crates/context_iterators)
[![msrv][]](https://github.com/aborgna/context_iterators)

Iterators adaptors with associated read-only data.

Useful for naming the types of wrapped iterators by using function pointers or
non-capturing closures.

```rust
use context_iterators::*;

type Closure = fn(usize, &usize) -> usize;
type MappedIterator = MapCtx<WithCtx<std::ops::Range<usize>, usize>, Closure>;

let iter: MappedIterator = (0..10)
    .with_context(42)
    .map_with_context(|item: usize, context: &usize| item + *context);
```

The `MappedIterator` type can be used in contexts where a concrete type is
needed, for example as an associated type of a trait.

```rust
trait Iterable {
    type Iter: Iterator<Item = usize>;
}

struct MyIterable;

impl Iterable for MyIterable {
   type Iter = MappedIterator;
}
```

Please read the [API documentation here][].

## Recent Changes

See [CHANGELOG][] for a list of changes. The minimum supported rust version will
only change on major releases.

## License

This project is dual-licensed under the Apache License, Version 2.0
http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
http://opensource.org/licenses/MIT, at your option. This file may not be copied,
modified, or distributed except according to those terms.

  [API documentation here]: https://docs.rs/context_iterators/
  [build_status]: https://github.com/aborgna/context_iterators/workflows/Continuous%20integration/badge.svg?branch=main
  [crates]: https://img.shields.io/crates/v/context_iterators
  [LICENSE]: LICENCE
  [msrv]: https://img.shields.io/badge/rust-1.70.0%2B-blue.svg?maxAge=3600
  [RELEASES]: RELEASES.rst