use rust_decimal::Decimal;
use serde::Serialize;

use crate::models::account::Account;

/// Helper struct for serializing account output with total.
#[derive(Serialize)]
pub struct AccountOutput<'a> {
    pub client: u16,

    #[serde(with = "rust_decimal::serde::str")]
    pub available: &'a Decimal,

    #[serde(with = "rust_decimal::serde::str")]
    pub held: &'a Decimal,

    #[serde(with = "rust_decimal::serde::str")]
    pub total: &'a Decimal,

    pub locked: bool,
}

use std::{collections::HashMap, io::Write};

pub fn output_accounts<W: Write>(accounts: &HashMap<u16, Account>, writer: W) {
    let mut builder = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(writer);

    let _ = builder.write_record(["client", "available", "held", "total", "locked"]);

    for account in accounts.values() {
        let total = account.available + account.held;

        let output = AccountOutput {
            client: account.client_id,
            available: &account.available,
            held: &account.held,
            total: &total,
            locked: account.locked,
        };

        let _ = builder.serialize(&output);
    }

    let _ = builder.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::str::{self, FromStr};

    #[test]
    fn test_output_accounts_csv() {
        let mut accounts = HashMap::new();

        accounts.insert(
            1,
            Account {
                client_id: 1,
                available: Decimal::from_str("10.5").unwrap(),
                held: Decimal::from_str("2.5").unwrap(),
                locked: false,
            },
        );

        accounts.insert(
            2,
            Account {
                client_id: 2,
                available: Decimal::from_str("3.0").unwrap(),
                held: Decimal::ZERO,
                locked: true,
            },
        );

        let mut output = Vec::new();

        output_accounts(&accounts, &mut output);

        let csv_str = str::from_utf8(&output).unwrap();

        println!("CSV Output:\n{}", csv_str);

        // Assert it contains expected rows
        assert!(csv_str.contains("client,available,held,total,locked"));
        assert!(csv_str.contains("1,10.5,2.5,13.0,false"));
        assert!(csv_str.contains("2,3.0,0,3.0,true"));
    }
}
