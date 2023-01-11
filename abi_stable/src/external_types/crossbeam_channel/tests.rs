use super::*;

use crossbeam_utils::thread::scope as scoped_thread;

#[test]
fn test_size_methods() {
    let cap = 5;
    {
        let (tx, rx) = bounded(cap);
        assert_eq!(tx.capacity(), Some(cap));
        assert_eq!(rx.capacity(), Some(cap));

        assert_eq!(tx.len(), 0);
        assert_eq!(rx.len(), 0);
        assert!(tx.is_empty());
        assert!(rx.is_empty());
        for i in 0..cap {
            tx.send(i).unwrap();
            assert_eq!(tx.len(), i + 1);
            assert_eq!(rx.len(), i + 1);
        }
        assert_eq!(tx.len(), cap);
        assert_eq!(rx.len(), cap);
        assert!(tx.is_full());
        assert!(rx.is_full());

        for i in 0..cap {
            assert_eq!(tx.len(), cap - i);
            assert_eq!(rx.len(), cap - i);
            assert_eq!(rx.recv().unwrap(), i);
        }

        assert_eq!(tx.len(), 0);
        assert_eq!(rx.len(), 0);
        assert!(tx.is_empty());
        assert!(rx.is_empty());
    }
    {
        let (tx, rx) = unbounded::<u32>();
        assert_eq!(tx.capacity(), None);
        assert_eq!(rx.capacity(), None);
    }
}

#[test]
#[cfg(not(all(miri, target_os = "windows")))]
fn send_recv() {
    scoped_thread(|scope| {
        let (tx, rx) = bounded::<u32>(0);

        let cap = 5;
        scope.spawn(move |_| {
            for i in 0..cap {
                tx.send(i).unwrap();
            }
        });

        for i in 0..cap {
            assert_eq!(rx.recv().unwrap(), i);
        }
    })
    .unwrap();
}

// miri gets stuck in this test
#[cfg(not(miri))]
#[test]
fn send_try_recv() {
    let (tx, rx) = bounded::<u32>(0);

    scoped_thread(|scope| {
        let cap = 5;
        let tx = tx.clone();
        let rx = rx.clone();

        assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

        scope.spawn(move |_| {
            for i in 0..cap {
                tx.send(i).unwrap();
            }
        });

        scope.spawn(move |_| {
            for i in 0..cap {
                loop {
                    match rx.try_recv() {
                        Ok(v) => {
                            assert_eq!(v, i);
                            break;
                        }
                        Err(TryRecvError::Empty) => continue,
                        Err(TryRecvError::Disconnected) => {
                            panic!("Channel was disconnected at iteration {}", i)
                        }
                    }
                }
            }
        });
    })
    .unwrap();

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

// miri gets stuck here
#[cfg(not(miri))]
#[test]
fn try_send_recv() {
    let (tx, rx) = bounded::<u32>(0);

    scoped_thread(|scope| {
        let cap = 5;
        let tx = tx.clone();
        let rx = rx.clone();

        assert_eq!(tx.try_send(0), Err(TrySendError::Full(0)));

        scope.spawn(move |_| {
            for i in 0..cap {
                loop {
                    match tx.try_send(i) {
                        Ok(()) => break,
                        Err(TrySendError::Full(x)) => {
                            assert_eq!(x, i);
                            continue;
                        }
                        Err(TrySendError::Disconnected { .. }) => {
                            panic!("Channel was disconnected at iteration {}", i)
                        }
                    }
                }
            }
        });

        scope.spawn(move |_| {
            for i in 0..cap {
                assert_eq!(rx.recv().unwrap(), i);
            }
        });
    })
    .unwrap();

    assert_eq!(tx.try_send(0), Err(TrySendError::Full(0)));
}

const MS: Duration = Duration::from_millis(1);

// unsupported operation: `clock_gettime` not available when isolation is enabled
#[test]
#[cfg(not(all(miri, target_os = "windows")))]
fn timeout_send_recv() {
    let (tx, rx) = bounded::<u32>(0);

    scoped_thread(|scope| {
        let cap = 5;

        let tx = tx.clone();
        let rx = rx.clone();

        assert_eq!(rx.recv_timeout(MS), Err(RecvTimeoutError::Timeout));

        scope.spawn(move |_| {
            for i in 0..cap {
                while tx.send_timeout(i, MS).is_err() {}
            }
        });

        scope.spawn(move |_| {
            for i in 0..cap {
                loop {
                    match rx.recv_timeout(Duration::from_millis(1)) {
                        Ok(v) => {
                            assert_eq!(v, i);
                            break;
                        }
                        Err(_) => continue,
                    }
                }
            }
        });
    })
    .unwrap();

    assert_eq!(rx.recv_timeout(MS), Err(RecvTimeoutError::Timeout));
}

#[test]
fn disconnected() {
    {
        let (tx, rx) = unbounded::<()>();
        let clone = tx.clone();
        assert_eq!(rx.try_recv().unwrap_err(), TryRecvError::Empty);
        drop(tx);
        assert_eq!(rx.try_recv().unwrap_err(), TryRecvError::Empty);
        drop(clone);
        assert_eq!(rx.try_recv().unwrap_err(), TryRecvError::Disconnected);
    }
    {
        let (tx, rx) = bounded::<()>(0);
        let clone = rx.clone();
        assert_eq!(tx.try_send(()).unwrap_err(), TrySendError::Full(()));
        drop(rx);
        assert_eq!(tx.try_send(()).unwrap_err(), TrySendError::Full(()));
        drop(clone);
        assert_eq!(tx.try_send(()).unwrap_err(), TrySendError::Disconnected(()));
    }
}

#[test]
#[cfg(not(all(miri, target_os = "windows")))]
fn iter() {
    let (tx, rx) = unbounded::<usize>();

    scoped_thread(|scope| {
        let rx = rx.clone();

        scope.spawn(move |_| {
            for i in 0..5 {
                tx.send(i).unwrap();
            }
        });

        scope.spawn(move |_| {
            for (i, j) in rx.iter().enumerate() {
                assert_eq!(i, j);
            }
        });
    })
    .unwrap();

    assert_ne!(rx.try_recv().err(), None);
}

#[test]
#[cfg(not(all(miri, target_os = "windows")))]
fn into_iter() {
    let (tx, rx) = unbounded::<usize>();

    scoped_thread(|scope| {
        let rx = rx.clone();

        scope.spawn(move |_| {
            for i in 0..5 {
                tx.send(i).unwrap();
            }
        });

        scope.spawn(move |_| {
            for (i, j) in rx.into_iter().enumerate() {
                assert_eq!(i, j);
            }
        });
    })
    .unwrap();

    assert_ne!(rx.try_recv().err(), None);
}
