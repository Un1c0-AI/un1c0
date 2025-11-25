// Simple COBOL-style banking transaction record
// Demonstrates UN1C0DE's ability to translate legacy mainframe code
// Target: 42 MLOC bank core equivalence

pub struct BankTransaction {
    pub account_number: String,      // PIC 9(10)
    pub transaction_type: String,    // PIC X(4)
    pub amount: f64,                 // PIC 9(13)V99
    pub balance: f64,                // PIC 9(13)V99
    pub timestamp: String,           // PIC X(26)
}

impl BankTransaction {
    pub fn new(account: &str, trans_type: &str, amt: f64) -> Self {
        BankTransaction {
            account_number: account.to_string(),
            transaction_type: trans_type.to_string(),
            amount: amt,
            balance: 0.0,
            timestamp: String::from("2025-11-25T00:00:00Z"),
        }
    }
    
    pub fn process_debit(&mut self, current_balance: f64) -> Result<f64, String> {
        if current_balance < self.amount {
            return Err(String::from("INSUFFICIENT-FUNDS"));
        }
        self.balance = current_balance - self.amount;
        Ok(self.balance)
    }
    
    pub fn process_credit(&mut self, current_balance: f64) -> Result<f64, String> {
        self.balance = current_balance + self.amount;
        Ok(self.balance)
    }
    
    pub fn display_record(&self) {
        println!("ACCT: {} TYPE: {} AMT: ${:.2} BAL: ${:.2}", 
                 self.account_number, 
                 self.transaction_type,
                 self.amount,
                 self.balance);
    }
}

pub fn main() {
    let mut trans = BankTransaction::new("1234567890", "DEBT", 500.00);
    match trans.process_debit(1000.00) {
        Ok(balance) => trans.display_record(),
        Err(e) => println!("ERROR: {}", e),
    }
}
