#[derive(Debug, Clone, PartialEq)]
pub enum OverflowPolicy {
    Fail,
    Drop,
    Block,
}
