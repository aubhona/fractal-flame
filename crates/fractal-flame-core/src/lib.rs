pub mod app;
pub mod domain;
pub mod infra;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4)
    }
}
