pub struct View {
    value: u64,
}

impl View {
    pub fn new() -> Self {
        Self { value: 1 }
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}