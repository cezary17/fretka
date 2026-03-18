pub mod max_length;

pub trait Truncator {
    fn truncate(&self, text: &str) -> String;
}
