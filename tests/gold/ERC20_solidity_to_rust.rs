// UN1C⓪ v0.2: Solidity → Rust translation
// Expected output for examples/solidity/ERC20.sol

pub struct ERC20 {
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: u256,
    balance_of: Map<H160, u256>,
    allowance: Map<H160, Map<H160, u256>>,
}

impl ERC20 {
    pub fn new(name: String, symbol: String, decimals: u8, initial_supply: u256) -> Self {
        // UEG Lambda node with SafetyLineage::OWNED
        Self {
            name,
            symbol,
            decimals,
            total_supply: initial_supply,
            balance_of: Map::new(),
            allowance: Map::new(),
        }
    }

    pub fn transfer(&mut self, to: H160, value: u256) -> bool {
        // UEG tags: NO_OVERFLOW = true
        if self.balance_of[&msg_sender()] < value {
            panic!("Insufficient balance");
        }
        self.balance_of[&msg_sender()] -= value;
        self.balance_of[&to] += value;
        true
    }

    pub fn approve(&mut self, spender: H160, value: u256) -> bool {
        self.allowance[&msg_sender()][&spender] = value;
        true
    }

    pub fn transfer_from(&mut self, from: H160, to: H160, value: u256) -> bool {
        if self.balance_of[&from] < value {
            panic!("Insufficient balance");
        }
        if self.allowance[&from][&msg_sender()] < value {
            panic!("Allowance exceeded");
        }
        self.balance_of[&from] -= value;
        self.balance_of[&to] += value;
        self.allowance[&from][&msg_sender()] -= value;
        true
    }
}
