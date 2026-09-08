use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;
use rand::Rng;
use crate::epoch::EpochManager;

const MAX_LEVEL: usize = 16;

struct Node<K, V> {
    key: K,
    value: V,
    height: usize,
    next: Vec<AtomicPtr<Node<K, V>>>,
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V, height: usize) -> *mut Self {
        let mut next = Vec::with_capacity(height);
        for _ in 0..height {
            next.push(AtomicPtr::new(ptr::null_mut()));
        }
        Box::into_raw(Box::new(Node { key, value, height, next }))
    }
}

pub struct LockFreeSkipList<K, V> {
    head: *mut Node<K, V>,
    epoch_mgr: EpochManager,
}

unsafe impl<K: Send + Sync, V: Send + Sync> Send for LockFreeSkipList<K, V> {}
unsafe impl<K: Send + Sync, V: Send + Sync> Sync for LockFreeSkipList<K, V> {}

impl<K: Ord + Default + Clone, V: Clone> LockFreeSkipList<K, V> {
    pub fn new() -> Self {
        let head = Node::new(K::default(), V::clone(&V::default()), MAX_LEVEL);
        Self {
            head,
            epoch_mgr: EpochManager::new(),
        }
    }

    fn random_level(&self) -> usize {
        let mut rng = rand::thread_rng();
        let mut level = 1;
        while level < MAX_LEVEL && rng.gen_bool(0.5) {
            level += 1;
        }
        level
    }

    pub fn insert(&self, key: K, value: V) -> bool {
        let _guard = self.epoch_mgr.enter();
        let height = self.random_level();
        let new_node = Node::new(key.clone(), value, height);

        unsafe {
            let mut curr = self.head;
            for level in (0..height).rev() {
                let next_ptr = (*curr).next[level].load(Ordering::Acquire);
                (*new_node).next[level].store(next_ptr, Ordering::Relaxed);
            }

            // Atomic CAS to link head at lowest level
            let res = (*self.head).next[0].compare_exchange(
                ptr::null_mut(),
                new_node,
                Ordering::Release,
                Ordering::Relaxed,
            );

            if res.is_err() {
                let _ = Box::from_raw(new_node);
                self.epoch_mgr.exit();
                return false;
            }
        }
        self.epoch_mgr.exit();
        true
    }

    pub fn find(&self, key: &K) -> Option<V> {
        let _guard = self.epoch_mgr.enter();
        unsafe {
            let mut curr = self.head;
            for level in (0..MAX_LEVEL).rev() {
                let mut next = (*curr).next[level].load(Ordering::Acquire);
                while !next.is_null() && &(*next).key < key {
                    curr = next;
                    next = (*curr).next[level].load(Ordering::Acquire);
                }
            }

            let candidate = (*curr).next[0].load(Ordering::Acquire);
            if !candidate.is_null() && &(*candidate).key == key {
                let val = (*candidate).value.clone();
                self.epoch_mgr.exit();
                return Some(val);
            }
        }
        self.epoch_mgr.exit();
        None
    }
}

impl<K, V> Drop for LockFreeSkipList<K, V> {
    fn drop(&mut self) {
        unsafe {
            let mut curr = self.head;
            while !curr.is_null() {
                let next = (*curr).next[0].load(Ordering::Relaxed);
                let _ = Box::from_raw(curr);
                curr = next;
            }
        }
    }
}
