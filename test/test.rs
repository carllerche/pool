extern crate pool;

use pool::Pool;

#[test]
pub fn test_checkout_checkin() {
    let mut pool: Pool<i32> = Pool::with_capacity(10, 0, || 0);

    let mut val = pool.checkout_raw().unwrap();
    assert_eq!(*val, 0);

    // Update the value & return to the pool
    *val = 1;
    drop(val);

    let val = pool.checkout_raw().unwrap();
    assert_eq!(*val, 1);
}

#[test]
pub fn test_multiple_checkouts() {
    let mut pool: Pool<i32> = Pool::with_capacity(10, 0, || 0);

    // Use this to hold on to the checkouts
    let mut vec = vec![];

    for _ in 0..10 {
        let mut i = pool.checkout_raw().unwrap();
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
        vec.push(pool.checkout_raw().unwrap());
    }

    assert!(pool.checkout_raw().is_none());
    drop(vec);
    assert!(pool.checkout_raw().is_some());
}

#[test]
pub fn test_resetting_pool() {
    let mut pool: Pool<Vec<i32>> = Pool::with_capacity(1, 0, || Vec::new());
    {
        let mut val = pool.checkout().unwrap();
        val.push(5);
        val.push(6);
    }
    {
        let val = pool.checkout().unwrap();
        assert!(val.len() == 0);
    }
}

// TODO: Add concurrency stress tests
