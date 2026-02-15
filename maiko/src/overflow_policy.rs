use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OverflowPolicy {
    #[default]
    Fail,
    Drop,
    Block,
}

impl OverflowPolicy {
    pub fn is_fail(&self) -> bool {
        matches!(self, OverflowPolicy::Fail)
    }

    pub fn is_drop(&self) -> bool {
        matches!(self, OverflowPolicy::Drop)
    }

    pub fn is_block(&self) -> bool {
        matches!(self, OverflowPolicy::Block)
    }
}

impl fmt::Display for OverflowPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OverflowPolicy::Fail => write!(f, "Fail"),
            OverflowPolicy::Drop => write!(f, "Drop"),
            OverflowPolicy::Block => write!(f, "Block"),
        }
    }
}
