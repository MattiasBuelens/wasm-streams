use std::future::Future;

pub struct LeakGuard {
    expected: u32,
}

impl LeakGuard {
    pub fn new() -> Self {
        Self {
            expected: wasm_bindgen::externref_heap_live_count(),
        }
    }

    #[inline]
    pub async fn run<Fn, Fut>(f: Fn)
    where
        Fn: FnOnce() -> Fut,
        Fut: Future<Output = ()>,
    {
        let guard = Self::new();
        {
            f().await;
        }
        drop(guard)
    }
}

impl Drop for LeakGuard {
    fn drop(&mut self) {
        let actual = wasm_bindgen::externref_heap_live_count();
        assert_eq!(
            actual,
            self.expected,
            "leaked {} externrefs",
            actual - self.expected
        );
    }
}
