use serde::{Deserialize, Serialize};

use crate::traits::HypergraphClass;

/// Marker for main hypergrpah
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Main;

/// Marker for sub hypergrpah
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sub;

impl HypergraphClass for Main {
    fn new() -> Self {
        Main
    }
    fn is_main(&self) -> bool {
        true
    }
}
impl HypergraphClass for Sub {
    fn new() -> Self {
        Sub
    }
    fn is_sub(&self) -> bool {
        true
    }
}
