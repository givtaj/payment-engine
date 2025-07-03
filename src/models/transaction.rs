use rust_decimal::Decimal;
use serde::Deserialize;

use crate::models::command::Command;

/// CSV input record with optional amount field.
/// Uses direct Decimal deserialization for clarity.
#[derive(Deserialize, Debug)]
pub struct TransactionInput {
    #[serde(rename = "type")]
    kind: String,

    #[serde(rename = "client")]
    client_id: u16,

    tx: u32,

    #[serde(default, with = "rust_decimal::serde::str_option")]
    amount: Option<Decimal>,
}

impl TransactionInput {
    /// Converts TransactionInput into a Command, validating required fields.
    pub fn to_command(&self) -> Result<Command, String> {
        match self.kind.as_str() {
            "deposit" => {
                let amount = self.amount.ok_or("Missing amount in deposit")?;
                Ok(Command::Deposit {
                    client_id: self.client_id,
                    tx: self.tx,
                    amount,
                })
            }
            "withdrawal" => {
                let amount = self.amount.ok_or("Missing amount in withdrawal")?;
                Ok(Command::Withdrawal {
                    client_id: self.client_id,
                    tx: self.tx,
                    amount,
                })
            }
            "dispute" => Ok(Command::Dispute {
                client_id: self.client_id,
                tx: self.tx,
            }),
            "resolve" => Ok(Command::Resolve {
                client_id: self.client_id,
                tx: self.tx,
            }),
            "chargeback" => Ok(Command::Chargeback {
                client_id: self.client_id,
                tx: self.tx,
            }),
            _ => Err(format!("Unknown transaction type: {}", self.kind)),
        }
    }
}

/// Internal record of a transaction for dispute resolution.
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub client_id: u16,
    pub amount: Decimal,
    pub is_deposit: bool,
    pub status: TransactionStatus,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TransactionStatus {
    Normal,
    Disputed,
    ChargedBack,
    // TODO: add rejected ?
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    /// Helper to build TransactionInput instances concisely.
    fn make_input(kind: &str, client: u16, tx: u32, amount: Option<Decimal>) -> TransactionInput {
        TransactionInput {
            kind: kind.into(),
            client_id: client,
            tx,
            amount,
        }
    }

    #[test]
    fn test_command_parsing_success_cases() {
        let deposit = make_input("deposit", 1, 10, Some(Decimal::new(50, 1))); // 5.0
        match deposit.to_command().unwrap() {
            Command::Deposit {
                client_id: client,
                tx,
                amount,
            } => {
                assert_eq!(client, 1);
                assert_eq!(tx, 10);
                assert_eq!(amount, Decimal::new(50, 1));
            }
            _ => panic!("Expected deposit"),
        }

        let withdrawal = make_input("withdrawal", 2, 20, Some(Decimal::new(25, 1))); // 2.5
        match withdrawal.to_command().unwrap() {
            Command::Withdrawal {
                client_id: client,
                tx,
                amount,
            } => {
                assert_eq!(client, 2);
                assert_eq!(tx, 20);
                assert_eq!(amount, Decimal::new(25, 1));
            }
            _ => panic!("Expected withdrawal"),
        }

        let dispute = make_input("dispute", 3, 30, None);
        match dispute.to_command().unwrap() {
            Command::Dispute {
                client_id: client,
                tx,
            } => {
                assert_eq!(client, 3);
                assert_eq!(tx, 30);
            }
            _ => panic!("Expected dispute"),
        }

        let resolve = make_input("resolve", 4, 40, None);
        match resolve.to_command().unwrap() {
            Command::Resolve {
                client_id: client,
                tx,
            } => {
                assert_eq!(client, 4);
                assert_eq!(tx, 40);
            }
            _ => panic!("Expected resolve"),
        }

        let chargeback = make_input("chargeback", 5, 50, None);
        match chargeback.to_command().unwrap() {
            Command::Chargeback {
                client_id: client,
                tx,
            } => {
                assert_eq!(client, 5);
                assert_eq!(tx, 50);
            }
            _ => panic!("Expected chargeback"),
        }
    }

    #[test]
    fn test_command_parsing_failure_cases() {
        // Missing amount for deposit
        let deposit_missing_amount = make_input("deposit", 1, 60, None);
        let res = deposit_missing_amount.to_command();
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), "Missing amount in deposit");

        // Missing amount for withdrawal
        let withdrawal_missing_amount = make_input("withdrawal", 2, 70, None);
        let res = withdrawal_missing_amount.to_command();
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), "Missing amount in withdrawal");

        // Unknown command type
        let unknown = make_input("foobar", 3, 80, None);
        let res = unknown.to_command();
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), "Unknown transaction type: foobar");
    }
}
