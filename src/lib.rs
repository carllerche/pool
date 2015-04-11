//! # A store of pre-initialized values.
//!
//! Values can be checked out when needed, operated on, and will automatically
//! be returned to the pool when they go out of scope. It can be used when
//! handling values that are expensive to create. Based on the [object pool
//! pattern](http://en.wikipedia.org/wiki/Object_pool_pattern).
//!
//! Example:
//!
//! ```
//! use pool::Pool;
//! use std::thread;
//!
//! let mut pool = Pool::with_capacity(20, 0, || Vec::with_capacity(16_384));
//!
//! let mut vec = pool.checkout_raw().unwrap();
//!
//! // Do some work with the value, this can happen in another thread
//! thread::scoped(move || {
//!     for i in 0..10_000 {
//!         vec.push(i);
//!     }
//!
//!     assert_eq!(10_000, vec.len());
//! });
//!
//! // The vec will have been returned to the pool by now
//! let vec = pool.checkout_raw().unwrap();
//!
//! // The pool operates LIFO, so this vec will be the same value that was used
//! // in the thread above. The value will also be left as it was when it was
//! // returned to the pool, this may or may not be desirable depending on the
//! // use case.
//! assert_eq!(10_000, vec.len());
//!
//! ```
//!
//! ## Extra byte storage
//!
//! Each value in the pool can be padded with an arbitrary number of bytes that
//! can be accessed as a slice. This is useful if implementing something like a
//! pool of buffers. The metadata could be stored as the `Pool` value and the
//! byte array can be stored in the padding.
//!
//! ## Threading
//!
//! Checking out values from the pool requires a mutable reference to the pool
//! so cannot happen concurrently across threads, but returning values to the
//! pool is thread safe and lock free, so if the value being pooled is `Sync`
//! then `Checkout<T>` is `Sync` as well.
//!
//! The easiest way to have a single pool shared across many threads would be
//! to wrap `Pool` in a mutex.
use std::{mem, ops, ptr, usize};
use std::sync::Arc;
use std::sync::atomic::{self, AtomicUsize, Ordering};
pub use reset::Reset;

mod reset;

/// A pool of reusable values
pub struct Pool<T> {
    inner: Arc<PoolInner<T>>,
}

impl<T> Pool<T> {
    /// Creates a new pool that can contain up to `capacity` entries as well as
    /// `extra` extra bytes. Initializes each entry with the given function.
    pub fn with_capacity<F>(count: usize, mut extra: usize, init: F) -> Pool<T>
            where F: Fn() -> T {

        let mut inner = PoolInner::with_capacity(count, extra);

        // Get the actual number of extra bytes
        extra = inner.entry_size - mem::size_of::<Entry<T>>();

        // Initialize the entries
        for i in 0..count {
            *inner.entry_mut(i) = Entry {
                data: init(),
                next: i + 1,
                extra: extra,
            }
        }

        Pool { inner: Arc::new(inner) }
    }

    /// Checkout a value from the pool. Returns `None` if the pool is currently
    /// at capacity.
    ///
    /// The value returned from the pool has not been reset and contains the
    /// state that it previously had when it was last released.
    pub fn checkout_raw(&mut self) -> Option<Checkout<T>> {
        self.inner_mut().checkout()
            .map(|ptr| {
                Checkout {
                    entry: ptr,
                    pool: self.inner.clone(),
                }
            })
    }

    fn inner_mut(&self) -> &mut PoolInner<T> {
        unsafe { mem::transmute(&*self.inner) }
    }
}

impl <T: Reset> Pool<T> {
    /// Checkout a value from the pool.  Returns `None` if the pool is currently
    /// at capacity.
    ///
    /// The value returned has been reset to its empty state.
    pub fn checkout(&mut self) -> Option<Checkout<T>> {
        match self.checkout_raw() {
            Some(mut obj) => {
                obj.reset();
                Some(obj)
            }
            None => None
        }
    }
}

unsafe impl<T: Send> Send for Pool<T> { }

/// A handle to a checked out value. When dropped out of scope, the value will
/// be returned to the pool.
pub struct Checkout<T> {
    entry: *mut Entry<T>,
    pool: Arc<PoolInner<T>>,
}

impl<T> Checkout<T> {
    /// Read access to the raw bytes
    pub fn extra(&self) -> &[u8] {
        self.entry().extra()
    }

    /// Write access to the extra bytes
    pub fn extra_mut(&self) -> &mut [u8] {
        self.entry_mut().extra_mut()
    }

    fn entry(&self) -> &Entry<T> {
        unsafe { mem::transmute(self.entry) }
    }

    fn entry_mut(&self) -> &mut Entry<T> {
        unsafe { mem::transmute(self.entry) }
    }
}

impl<T> ops::Deref for Checkout<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.entry().data
    }
}

impl<T> ops::DerefMut for Checkout<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.entry_mut().data
    }
}

impl<T> Drop for Checkout<T> {
    fn drop(&mut self) {
        self.pool.checkin(self.entry);
    }
}

unsafe impl<T: Send> Send for Checkout<T> { }
unsafe impl<T: Sync> Sync for Checkout<T> { }

struct PoolInner<T> {
    #[allow(dead_code)]
    memory: Box<[u8]>,  // Ownership of raw memory
    next: AtomicUsize,  // Offset to next available value
    ptr: *mut Entry<T>, // Pointer to first entry
    count: usize,       // Total number of entries
    entry_size: usize,  // Byte size of each entry
}

// Max size of the pool
const MAX: usize = usize::MAX >> 1;

impl<T> PoolInner<T> {
    fn with_capacity(count: usize, mut extra: usize) -> PoolInner<T> {
        // The required alignment for the entry. The start of the entry must
        // align with this number
        let align = mem::align_of::<Entry<T>>();

        // Check that the capacity is not too large
        assert!(count < MAX, "requested pool size too big");
        assert!(align > 0, "something weird is up with the requested alignment");

        let mask = align - 1;

        // If the requested extra memory does not match with the align,
        // increase it so that it does.
        if extra & mask != 0 {
            extra = (extra + align) & !mask;
        }

        // Calculate the size of each entry. Since the extra bytes are
        // immediately after the entry, just add the sizes
        let entry_size = mem::size_of::<Entry<T>>() + extra;

        // This should always be true, but let's check it anyway
        assert!(entry_size & mask == 0, "entry size is not aligned");

        // Ensure that the total memory needed is possible. It must be
        // representable by an `isize` value in order for pointer offset to
        // work.
        assert!(entry_size.checked_mul(count).is_some(), "requested pool capacity too big");
        assert!(entry_size * count < MAX, "requested pool capacity too big");

        let size = count * entry_size;

        // Allocate the memory
        let (memory, ptr) = alloc(size, align);

        // Zero out the memory for safety
        unsafe {
            ptr::write_bytes(ptr, 0, size);
        }

        PoolInner {
            memory: memory,
            next: AtomicUsize::new(0),
            ptr: ptr as *mut Entry<T>,
            count: count,
            entry_size: entry_size,
        }
    }

    fn checkout(&mut self) -> Option<*mut Entry<T>> {
        let mut idx = self.next.load(Ordering::Acquire);

        loop {
            debug_assert!(idx <= self.count, "invalid index: {}", idx);

            if idx == self.count {
                // The pool is depleted
                return None;
            }

            let nxt = self.entry_mut(idx).next;

            debug_assert!(nxt <= self.count, "invalid next index: {}", idx);

            let res = self.next.compare_and_swap(idx, nxt, Ordering::Relaxed);

            if res == idx {
                break;
            }

            // Re-acquire the memory before trying again
            atomic::fence(Ordering::Acquire);
            idx = res;
        }

        Some(self.entry_mut(idx) as *mut Entry<T>)
    }

    fn checkin(&self, ptr: *mut Entry<T>) {
        let mut idx;
        let mut entry: &mut Entry<T>;

        unsafe {
            // Figure out the index
            idx = ((ptr as usize) - (self.ptr as usize)) / self.entry_size;
            entry = mem::transmute(ptr);
        }

        debug_assert!(idx < self.count, "invalid index; idx={}", idx);

        let mut nxt = self.next.load(Ordering::Relaxed);

        loop {
            // Update the entry's next pointer
            entry.next = nxt;

            let actual = self.next.compare_and_swap(nxt, idx, Ordering::Release);

            if actual == nxt {
                break;
            }

            nxt = actual;
        }
    }

    fn entry(&self, idx: usize) -> &Entry<T> {
        unsafe {
            debug_assert!(idx < self.count, "invalid index");
            let ptr = self.ptr.offset(idx as isize);
            mem::transmute(ptr)
        }
    }

    fn entry_mut(&mut self, idx: usize) -> &mut Entry<T> {
        unsafe { mem::transmute(self.entry(idx)) }
    }
}

struct Entry<T> {
    data: T,       // Keep first
    next: usize,   // Index of next available entry
    extra: usize,  // Number of extra bytes available
}

impl<T> Entry<T> {
    fn extra(&self) -> &[u8] {
        use std::slice;

        unsafe {
            let ptr: *const u8 = mem::transmute(self);
            let ptr = ptr.offset(mem::size_of::<Entry<T>>() as isize);

            slice::from_raw_parts(ptr, self.extra)
        }
    }

    fn extra_mut(&mut self) -> &mut [u8] {
        unsafe { mem::transmute(self.extra()) }
    }
}

/// Allocate memory
fn alloc(mut size: usize, align: usize) -> (Box<[u8]>, *mut u8) {
    size += align;

    unsafe {
        // Allocate the memory
        let mut vec = Vec::with_capacity(size);
        vec.set_len(size);

        // Juggle values around
        let mut mem = vec.into_boxed_slice();
        let ptr = (*mem).as_mut_ptr();

        // Align the pointer
        let p = ptr as usize;
        let m = align - 1;

        if p & m != 0 {
            let p = (p + align) & !m;
            return (mem, p as *mut u8);
        }

        (mem, ptr)
    }
}
