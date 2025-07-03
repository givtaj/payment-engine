use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::models::{
    account::Account,
    command::Command,
    transaction::{TransactionRecord, TransactionStatus},
};

/// State of the payments engine, owning all client accounts and transactions.
pub struct State {
    pub accounts: HashMap<u16, Account>,
    transactions: HashMap<u32, TransactionRecord>,
}

impl State {
    pub fn new() -> Self {
        State {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    /// Process a single Command and update state.
    pub fn process_single_command(&mut self, cmd: Command) {
        match cmd {
            Command::Deposit {
                client_id: client,
                tx,
                amount,
            } => {
                if self.transactions.contains_key(&tx) {
                    // Duplicate transaction ID, ignore
                    return;
                }

                if self.accounts.get(&client).is_some_and(|acc| acc.locked) {
                    return;
                }
                // Create account if not exist
                let account = self.accounts.entry(client).or_insert_with(|| Account {
                    client_id: client,
                    available: Decimal::ZERO,
                    held: Decimal::ZERO,
                    locked: false,
                });

                // Apply deposit
                account.available += amount;

                self.transactions.insert(
                    tx,
                    TransactionRecord {
                        client_id: client,
                        amount,
                        is_deposit: true,
                        status: TransactionStatus::Normal,
                    },
                );
            }
            Command::Withdrawal {
                client_id: client,
                tx,
                amount,
            } => {
                // Check for duplicate tx id FIRST
                if self.transactions.contains_key(&tx) {
                    // Duplicate transaction ID, ignore
                    return;
                }

                if self.accounts.get(&client).is_some_and(|acc| acc.locked) {
                    return;
                }

                let account = self.accounts.entry(client).or_insert_with(|| Account {
                    client_id: client,
                    available: Decimal::ZERO,
                    held: Decimal::ZERO,
                    locked: false,
                });

                // Only withdraw if sufficient available funds
                if account.available >= amount {
                    account.available -= amount;

                    // Record successful withdrawal
                    self.transactions.insert(
                        tx,
                        TransactionRecord {
                            client_id: client,
                            amount,
                            is_deposit: false,
                            status: TransactionStatus::Normal,
                        },
                    );
                }
                // If insufficient funds, withdrawal is ignored (no change, no record)
            }
            Command::Dispute {
                client_id: client,
                tx,
            } => {
                // Skip if the account is already locked
                if let Some(account) = self.accounts.get_mut(&client) {
                    if account.locked {
                        return; // account is frozen – ignore this dispute
                    }
                }

                // Only process if the referenced transaction exists and is a deposit not already disputed
                if let Some(record) = self.transactions.get_mut(&tx) {
                    if record.client_id != client {
                        return; // client ID mismatch, ignore
                    }
                    if !record.is_deposit || record.status != TransactionStatus::Normal {
                        return; // can only dispute normal deposits
                    }
                    // Mark transaction as disputed
                    record.status = TransactionStatus::Disputed;
                    // Adjust account balances: move funds from available to held
                    if let Some(account) = self.accounts.get_mut(&client) {
                        account.available -= record.amount;
                        account.held += record.amount;
                    }
                }
            }
            Command::Resolve {
                client_id: client,
                tx,
            } => {
                // Skip if the account is already locked
                if let Some(account) = self.accounts.get_mut(&client) {
                    if account.locked {
                        return; // ignore resolve on a frozen account
                    }
                }

                if let Some(record) = self.transactions.get_mut(&tx) {
                    if record.client_id != client {
                        return;
                    }
                    if record.status != TransactionStatus::Disputed {
                        return; // only resolve an active dispute
                    }
                    // Mark transaction back to normal (dispute resolved)
                    record.status = TransactionStatus::Normal;
                    // Release held funds back to available
                    if let Some(account) = self.accounts.get_mut(&client) {
                        account.held -= record.amount;
                        account.available += record.amount;
                    }
                }
            }
            Command::Chargeback {
                client_id: client,
                tx,
            } => {
                // Check the transaction first
                if let Some(record) = self.transactions.get_mut(&tx) {
                    if record.client_id != client || record.status != TransactionStatus::Disputed {
                        return; // only chargeback a valid disputed transaction
                    }

                    // Fetch the account
                    if let Some(account) = self.accounts.get_mut(&client) {
                        if account.locked {
                            return; // ignore chargeback on a frozen account
                        }

                        // Finalize chargeback
                        record.status = TransactionStatus::ChargedBack;

                        account.held -= record.amount;

                        // Ensure held does not go negative, if your design requires
                        if account.held < Decimal::ZERO {
                            account.held = Decimal::ZERO;
                        }

                        account.locked = true; // always lock after chargeback
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal::prelude::FromStr;

    #[test]
    fn test_deposit_and_withdraw() {
        let mut state = State::new();
        // Deposit into client 1
        state.process_single_command(Command::Deposit {
            client_id: 1,
            tx: 1,
            amount: Decimal::from_str("10.0").unwrap(),
        });
        // Withdraw some amount
        state.process_single_command(Command::Withdrawal {
            client_id: 1,
            tx: 2,
            amount: Decimal::from_str("3.0").unwrap(),
        });
        // Check resulting balances
        let acc = state.accounts.get(&1).expect("Account 1 should exist");
        assert_eq!(acc.available, Decimal::from_str("7.0").unwrap());
        assert_eq!(acc.held, Decimal::ZERO);
        assert!(!acc.locked);
        // Withdraw more than available (should be ignored)
        state.process_single_command(Command::Withdrawal {
            client_id: 1,
            tx: 3,
            amount: Decimal::from_str("10.0").unwrap(),
        });
        // Balance should remain unchanged
        let acc_after = state.accounts.get(&1).unwrap();
        assert_eq!(acc_after.available, Decimal::from_str("7.0").unwrap());
    }

    #[test]
    fn test_dispute_and_resolve() {
        let mut state = State::new();
        // Make a deposit and then dispute it
        state.process_single_command(Command::Deposit {
            client_id: 2,
            tx: 10,
            amount: Decimal::from_str("5.0").unwrap(),
        });
        state.process_single_command(Command::Dispute {
            client_id: 2,
            tx: 10,
        });
        let acc = state.accounts.get(&2).unwrap();
        // After dispute: available should decrease, held should increase by 5.0
        assert_eq!(acc.available, Decimal::ZERO);
        assert_eq!(acc.held, Decimal::from_str("5.0").unwrap());
        // Resolve the dispute
        state.process_single_command(Command::Resolve {
            client_id: 2,
            tx: 10,
        });
        let acc2 = state.accounts.get(&2).unwrap();
        assert_eq!(acc2.available, Decimal::from_str("5.0").unwrap());
        assert_eq!(acc2.held, Decimal::ZERO);
        assert!(!acc2.locked);
    }

    #[test]
    fn test_chargeback_locks_account() {
        let mut state = State::new();
        // Deposit then dispute
        state.process_single_command(Command::Deposit {
            client_id: 3,
            tx: 20,
            amount: Decimal::from_str("2.5").unwrap(),
        });
        state.process_single_command(Command::Dispute {
            client_id: 3,
            tx: 20,
        });
        // Chargeback the disputed transaction
        state.process_single_command(Command::Chargeback {
            client_id: 3,
            tx: 20,
        });
        let acc = state.accounts.get(&3).unwrap();
        // Funds held should be removed and account locked
        assert_eq!(acc.available, Decimal::ZERO);
        assert_eq!(acc.held, Decimal::ZERO);
        assert!(acc.locked);
        // Further deposits or withdrawals on locked account should be ignored
        state.process_single_command(Command::Deposit {
            client_id: 3,
            tx: 21,
            amount: Decimal::from_str("1.0").unwrap(),
        });
        let acc_after = state.accounts.get(&3).unwrap();
        assert_eq!(acc_after.available, Decimal::ZERO);
    }

    #[test]
    fn test_dispute_on_withdrawal_is_ignored() {
        let mut state = State::new();
        // Deposit and then withdraw
        state.process_single_command(Command::Deposit {
            client_id: 4,
            tx: 100,
            amount: Decimal::from_str("8.0").unwrap(),
        });
        state.process_single_command(Command::Withdrawal {
            client_id: 4,
            tx: 101,
            amount: Decimal::from_str("3.0").unwrap(),
        });
        // Try to dispute the withdrawal (should be ignored)
        state.process_single_command(Command::Dispute {
            client_id: 4,
            tx: 101,
        });
        let acc = state.accounts.get(&4).unwrap();
        // Balances should remain unchanged
        assert_eq!(acc.available, Decimal::from_str("5.0").unwrap());
        assert_eq!(acc.held, Decimal::ZERO);
        assert!(!acc.locked);
        // Transaction status should remain Normal
        let tx_record = state.transactions.get(&101).unwrap();
        assert_eq!(tx_record.status, TransactionStatus::Normal);
    }

    #[test]
    fn test_duplicate_transaction_id_is_ignored() {
        let mut state = State::new();
        state.process_single_command(Command::Deposit {
            client_id: 5,
            tx: 200,
            amount: Decimal::from_str("10.0").unwrap(),
        });
        // Attempt another deposit with the same tx id
        state.process_single_command(Command::Deposit {
            client_id: 5,
            tx: 200,
            amount: Decimal::from_str("5.0").unwrap(),
        });
        let acc = state.accounts.get(&5).unwrap();
        // Only the first deposit should be counted
        assert_eq!(acc.available, Decimal::from_str("10.0").unwrap());
    }

    #[test]
    fn test_dispute_with_wrong_client_is_ignored() {
        let mut state = State::new();
        state.process_single_command(Command::Deposit {
            client_id: 6,
            tx: 300,
            amount: Decimal::from_str("7.0").unwrap(),
        });
        // Dispute from wrong client
        state.process_single_command(Command::Dispute {
            client_id: 7,
            tx: 300,
        });
        let acc = state.accounts.get(&6).unwrap();
        assert_eq!(acc.available, Decimal::from_str("7.0").unwrap());
        assert_eq!(acc.held, Decimal::ZERO);
        let tx_record = state.transactions.get(&300).unwrap();
        assert_eq!(tx_record.status, TransactionStatus::Normal);
    }

    #[test]
    fn test_dispute_on_nonexistent_transaction_is_ignored() {
        let mut state = State::new();
        // Dispute a tx that doesn't exist
        state.process_single_command(Command::Dispute {
            client_id: 8,
            tx: 400,
        });
        // No account or transaction should be created
        assert!(state.accounts.get(&8).is_none());
        assert!(state.transactions.get(&400).is_none());
    }

    #[test]
    fn test_resolve_on_non_disputed_transaction_is_ignored() {
        let mut state = State::new();
        state.process_single_command(Command::Deposit {
            client_id: 9,
            tx: 500,
            amount: Decimal::from_str("12.0").unwrap(),
        });
        // Try to resolve without a dispute
        state.process_single_command(Command::Resolve {
            client_id: 9,
            tx: 500,
        });
        let acc = state.accounts.get(&9).unwrap();
        assert_eq!(acc.available, Decimal::from_str("12.0").unwrap());
        assert_eq!(acc.held, Decimal::ZERO);
        let tx_record = state.transactions.get(&500).unwrap();
        assert_eq!(tx_record.status, TransactionStatus::Normal);
    }

    #[test]
    fn test_chargeback_on_non_disputed_transaction_is_ignored() {
        let mut state = State::new();
        state.process_single_command(Command::Deposit {
            client_id: 10,
            tx: 600,
            amount: Decimal::from_str("15.0").unwrap(),
        });
        // Try to chargeback without a dispute
        state.process_single_command(Command::Chargeback {
            client_id: 10,
            tx: 600,
        });
        let acc = state.accounts.get(&10).unwrap();
        assert_eq!(acc.available, Decimal::from_str("15.0").unwrap());
        assert_eq!(acc.held, Decimal::ZERO);
        assert!(!acc.locked);
        let tx_record = state.transactions.get(&600).unwrap();
        assert_eq!(tx_record.status, TransactionStatus::Normal);
    }

    #[test]
    fn test_dispute_on_already_disputed_transaction_is_ignored() {
        let mut state = State::new();
        state.process_single_command(Command::Deposit {
            client_id: 11,
            tx: 700,
            amount: Decimal::from_str("20.0").unwrap(),
        });
        state.process_single_command(Command::Dispute {
            client_id: 11,
            tx: 700,
        });
        // Try to dispute again
        state.process_single_command(Command::Dispute {
            client_id: 11,
            tx: 700,
        });
        let acc = state.accounts.get(&11).unwrap();
        assert_eq!(acc.available, Decimal::ZERO);
        assert_eq!(acc.held, Decimal::from_str("20.0").unwrap());
        let tx_record = state.transactions.get(&700).unwrap();
        assert_eq!(tx_record.status, TransactionStatus::Disputed);
    }

    #[test]
    fn test_duplicate_tx_id_withdrawal_is_ignored() {
        let mut state = State::new();
        // Deposit funds to allow withdrawal
        state.process_single_command(Command::Deposit {
            client_id: 20,
            tx: 1000,
            amount: Decimal::from_str("10.0").unwrap(),
        });
        // First withdrawal succeeds
        state.process_single_command(Command::Withdrawal {
            client_id: 20,
            tx: 1001,
            amount: Decimal::from_str("5.0").unwrap(),
        });
        // Duplicate withdrawal tx id with different amount should be ignored
        state.process_single_command(Command::Withdrawal {
            client_id: 20,
            tx: 1001,
            amount: Decimal::from_str("3.0").unwrap(),
        });
        let acc = state.accounts.get(&20).unwrap();
        assert_eq!(acc.available, Decimal::from_str("5.0").unwrap());
    }

    #[test]
    fn test_insufficient_funds_withdrawal_not_recorded() {
        let mut state = State::new();
        state.process_single_command(Command::Withdrawal {
            client_id: 21,
            tx: 2000,
            amount: Decimal::from_str("5.0").unwrap(),
        });
        assert!(state.transactions.get(&2000).is_none());
    }

    #[test]
    fn test_deposit_to_locked_account_is_ignored() {
        let mut state = State::new();
        // Deposit and chargeback to lock account
        state.process_single_command(Command::Deposit {
            client_id: 22,
            tx: 3000,
            amount: Decimal::from_str("10.0").unwrap(),
        });
        state.process_single_command(Command::Dispute {
            client_id: 22,
            tx: 3000,
        });
        state.process_single_command(Command::Chargeback {
            client_id: 22,
            tx: 3000,
        });
        // Attempt deposit after lock
        state.process_single_command(Command::Deposit {
            client_id: 22,
            tx: 3001,
            amount: Decimal::from_str("5.0").unwrap(),
        });
        let acc = state.accounts.get(&22).unwrap();
        assert_eq!(acc.available, Decimal::ZERO);
    }

    #[test]
    fn test_deposit_withdraw_then_chargeback() {
        let mut state = State::new();

        // Step 1: User deposits $10
        state.process_single_command(Command::Deposit {
            client_id: 42,
            tx: 100,
            amount: Decimal::from_str("10.0").unwrap(),
        });

        // Step 2: User withdraws all $10
        state.process_single_command(Command::Withdrawal {
            client_id: 42,
            tx: 101,
            amount: Decimal::from_str("10.0").unwrap(),
        });

        // Assert available is now 0
        let acc = state.accounts.get(&42).unwrap();
        assert_eq!(acc.available, Decimal::ZERO);
        assert_eq!(acc.held, Decimal::ZERO);
        assert!(!acc.locked);

        // Step 3: User disputes their original deposit tx (attempting reversal)
        state.process_single_command(Command::Dispute {
            client_id: 42,
            tx: 100,
        });

        // Assert available becomes negative if dispute moves funds to held
        let acc = state.accounts.get(&42).unwrap();
        assert_eq!(acc.available, Decimal::from_str("-10.0").unwrap());
        assert_eq!(acc.held, Decimal::from_str("10.0").unwrap());
        assert!(!acc.locked);

        // Step 4: User issues chargeback on that deposit
        state.process_single_command(Command::Chargeback {
            client_id: 42,
            tx: 100,
        });

        // Assert account is locked and held funds removed
        let acc = state.accounts.get(&42).unwrap();
        assert_eq!(acc.available, Decimal::from_str("-10.0").unwrap());
        assert_eq!(acc.held, Decimal::ZERO);
        assert!(acc.locked);
    }
}
