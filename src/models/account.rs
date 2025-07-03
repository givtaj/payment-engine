use rust_decimal::Decimal;

/// Represents a client account state.
#[derive(serde::Serialize, Debug)]
pub struct Account {
    pub client_id: u16,

    #[serde(with = "rust_decimal::serde::str")]
    pub available: Decimal,

    #[serde(with = "rust_decimal::serde::str")]
    pub held: Decimal,

    pub locked: bool,
}
