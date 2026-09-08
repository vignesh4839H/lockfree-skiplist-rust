use lockfree_skiplist::LockFreeSkipList;
use std::sync::Arc;
use std::thread;

#[test]
fn test_basic_operations() {
    let list = LockFreeSkipList::<i32, i32>::new();
    assert!(list.insert(10, 100));
    assert_eq!(list.find(&10), Some(100));
    assert_eq!(list.find(&99), None);
}

#[test]
fn test_concurrent_insertions() {
    let list = Arc::new(LockFreeSkipList::<i32, i32>::new());
    let mut handles = vec![];

    for i in 0..4 {
        let list_clone = Arc::clone(&list);
        handles.push(thread::spawn(move || {
            for j in 0..100 {
                let key = i * 100 + j;
                list_clone.insert(key, key * 10);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
