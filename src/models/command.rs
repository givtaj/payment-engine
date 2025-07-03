use rust_decimal::Decimal;

/// Represents high-level parsed commands from input.
#[derive(Debug, Clone)]
pub enum Command {
    Deposit {
        client_id: u16,
        tx: u32,
        amount: Decimal,
    },
    Withdrawal {
        client_id: u16,
        tx: u32,
        amount: Decimal,
    },
    Dispute {
        client_id: u16,
        tx: u32,
    },
    Resolve {
        client_id: u16,
        tx: u32,
    },
    Chargeback {
        client_id: u16,
        tx: u32,
    },
}
