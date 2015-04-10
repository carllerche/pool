# A pool of reusable values

A Rust library providing a pool structure for managing reusable values.
All values in the pool are initialized when the pool is created. Values
can be checked out from the pool at any time. When the checked out value
goes out of scope, the value is returned to the pool and made available
for checkout at a later time.

[![Build Status](https://travis-ci.org/carllerche/pool.svg?branch=master)](https://travis-ci.org/carllerche/pool)
