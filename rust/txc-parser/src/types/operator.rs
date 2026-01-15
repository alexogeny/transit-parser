//! TXC Operator types.

use serde::{Deserialize, Serialize};

/// A transit operator in TXC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcOperator {
    /// Unique identifier for the operator.
    pub id: String,
    /// National operator code.
    pub national_operator_code: Option<String>,
    /// Operator short name.
    pub operator_short_name: Option<String>,
    /// Operator name code.
    pub operator_name_on_licence: Option<String>,
    /// Trading name.
    pub trading_name: Option<String>,
    /// Licence number.
    pub licence_number: Option<String>,
}

impl TxcOperator {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            national_operator_code: None,
            operator_short_name: None,
            operator_name_on_licence: None,
            trading_name: None,
            licence_number: None,
        }
    }

    /// Get the best display name for this operator.
    pub fn display_name(&self) -> &str {
        self.trading_name
            .as_deref()
            .or(self.operator_short_name.as_deref())
            .or(self.operator_name_on_licence.as_deref())
            .unwrap_or(&self.id)
    }
}
