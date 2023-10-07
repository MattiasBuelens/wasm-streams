pub struct LeakGuard {
    expected: u32,
}

impl LeakGuard {
    pub fn new() -> Self {
        Self {
            expected: wasm_bindgen::externref_heap_live_count(),
        }
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
