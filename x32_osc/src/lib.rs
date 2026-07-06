mod types;
mod connections;

pub use types::X32Console;
pub use connections::ConnectionManager;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
