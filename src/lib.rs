pub mod bubblegum;

#[cfg(test)]
mod tests {
    use super::*;
    use bubblegum::hello;

    #[test]
    fn it_works() {
        let result = hello();
        assert_eq!(result, "hello");
    }
}
