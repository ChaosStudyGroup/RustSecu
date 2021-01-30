//! 暴露了内部使用的裸指针，导致 Safe Rust 代码出现段错误
//! https://github.com/krl/cache/issues/2



use cache;


/**

    `cache crate` 内部代码：

    ```rust
    pub enum Cached<'a, V: 'a> {
        /// Value could not be put on the cache, and is returned in a box
        /// as to be able to implement `StableDeref`
        Spilled(Box<V>),
        /// Value resides in cache and is read-locked.
        Cached {
            /// The readguard from a lock on the heap
            guard: RwLockReadGuard<'a, ()>,
            /// A pointer to a value on the heap
            // 漏洞风险
            ptr: *const ManuallyDrop<V>,
        },
        /// A value that was borrowed from outside the cache.
        Borrowed(&'a V),
    }

    ```
**/
fn main() {
    let c = cache::Cache::new(8, 4096);
    c.insert(1, String::from("test"));
    let mut e = c.get::<String>(&1).unwrap();

    match &mut e {
        cache::Cached::Cached { ptr, .. } => {
            // 将 ptr 设置为 空指针，导致段错误
            *ptr = std::ptr::null();
        },
        _ => panic!(),
    }
    // 输出：3851，段错误
    println!("Entry: {}", *e);
}