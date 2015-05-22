# A pool of reusable values

A Rust library providing a pool structure for managing reusable values.
All values in the pool are initialized when the pool is created. Values
can be checked out from the pool at any time. When the checked out value
goes out of scope, the value is returned to the pool and made available
for checkout at a later time.

[![Build Status](https://travis-ci.org/carllerche/pool.svg?branch=master)](https://travis-ci.org/carllerche/pool)

- [API documentation](http://carllerche.github.io/pool/pool/)

- [Crates.io](https://crates.io/crates/pool)

## Usage

To use `pool`, first add this to your `Cargo.toml`:

```toml
[dependencies]
pool = "0.1.3"
```

Then, add this to your crate root:

```rust
extern crate pool;
```

## Features

* Simple
* Lock-free: values can be returned to the pool across threads
* Stores typed values and / or slabs of memory
