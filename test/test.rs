extern crate pool;

use pool::Pool;

#[test]
pub fn test_checkout_checkin() {
    let mut pool: Pool<i32> = Pool::with_capacity(10, 0, || 0);

    let mut val = pool.checkout().unwrap();
    assert_eq!(*val, 0);

    // Update the value & return to the pool
    *val = 1;
    drop(val);

    let val = pool.checkout().unwrap();
    assert_eq!(*val, 1);
}

#[test]
pub fn test_multiple_checkouts() {
    let mut pool: Pool<i32> = Pool::with_capacity(10, 0, || 0);

    // Use this to hold on to the checkouts
    let mut vec = vec![];

    for _ in 0..10 {
        let mut i = pool.checkout().unwrap();
        assert_eq!(*i, 0);
        *i = 1;
        vec.push(i);
    }
}

#[test]
pub fn test_depleting_pool() {
    let mut pool: Pool<i32> = Pool::with_capacity(5, 0, || 0);

    let mut vec = vec![];

    for _ in 0..5 {
        vec.push(pool.checkout().unwrap());
    }

    assert!(pool.checkout().is_none());
    drop(vec);
    assert!(pool.checkout().is_some());
}

// TODO: Add concurrency stress tests
