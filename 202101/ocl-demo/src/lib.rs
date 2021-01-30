//! https://github.com/cogciprocate/ocl/issues/194
//!
//! 安全漏洞：Panic Safty: 使用了`ptr::read` 在panic的时候导致双重释放

/**
    ```rust
    //case 1
    macro_rules! from_event_option_array_into_event_list(
        ($e:ty, $len:expr) => (
            impl<'e> From<[Option<$e>; $len]> for EventList {
                 fn from(events: [Option<$e>; $len]) -> EventList {
                     let mut el = EventList::with_capacity(events.len());
                     for idx in 0..events.len() {
                        // 这个 unsafe 用法在 `event.into()`调用panic的时候会导致双重释放
                         let event_opt = unsafe { ptr::read(events.get_unchecked(idx)) };
                         if let Some(event) = event_opt { el.push::<Event>(event.into()); }
                     }
                     // 此处 mem::forget 就是为了防止 `dobule free`。
                     // 因为 `ptr::read` 也会制造一次 drop。
                     // 所以上面如果发生了panic，那就相当于注释了 `mem::forget`，导致`dobule free`
                     mem::forget(events);
                     el
                 }
            }
        )
    );

    // case2

     impl<'e, E> From<[E; $len]> for EventList where E: Into<Event> {
         fn from(events: [E; $len]) -> EventList {
             let mut el = EventList::with_capacity(events.len());
             for idx in 0..events.len() {
                // 同上
                 let event = unsafe { ptr::read(events.get_unchecked(idx)) };
                 el.push(event.into());
             }
             // Ownership has been unsafely transfered to the new event
             // list without modifying the event reference count. Not
             // forgetting the source array would cause a double drop.
             mem::forget(events);
             el
         }
     }

    // POC:以下代码证明了上面两个case会发生dobule free 问题

    use fil_ocl::{Event, EventList};
    use std::convert::Into;

    struct Foo(Option<i32>);

    impl Into<Event> for Foo {
        fn into(self) -> Event {
            /*
            根据文档，`Into <T>`实现不应出现 panic。但是rustc不会检查Into实现中是否会发生恐慌，
            因此用户提供的`into（）`可能会出现风险
            */
            println!("LOUSY PANIC : {}", self.0.unwrap()); // unwrap 是有 panic 风险

            Event::empty()
        }
    }

    impl Drop for Foo {
        fn drop(&mut self) {
            println!("I'm dropping");
        }
    }

    fn main() {
        let eventlist: EventList = [Foo(None)].into();
        dbg!(eventlist);
    }

    ```

    ### fix 漏洞的代码：

    ```rust
    macro_rules! from_event_option_array_into_event_list(
        ($e:ty, $len:expr) => (
            impl<'e> From<[Option<$e>; $len]> for EventList {
                fn from(events: [Option<$e>; $len]) -> EventList {
                    let mut el = ManuallyDrop::new(
                        EventList::with_capacity(events.len())
                    );

                    for idx in 0..events.len() {
                        let event_opt = unsafe {
                            ptr::read(events.get_unchecked(idx))
                        };

                        if let Some(event) = event_opt {
                            // Use `ManuallyDrop` to guard against
                            // potential panic within `into()`.
                            // 当 into 方法发生 panic 当时候，这里 ManuallyDrop 可以保护其不会`double free`
                            let event = ManuallyDrop::into_inner(
                                ManuallyDrop::new(event)
                                .into()
                            );
                            el.push(event);
                        }
                    }
                    mem::forget(events);
                    ManuallyDrop::into_inner(el)
                }
            }
        )
    );

    ```

**/
fn demo(){}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
