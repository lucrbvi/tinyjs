pub fn fail(message: impl Into<String>) -> ! {
    panic!("{}", message.into());
}
