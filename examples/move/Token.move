module 0x1::Token {
    use std::signer;
    
    struct Coin has key {
        value: u64,
    }
    
    public fun mint(account: &signer, amount: u64) {
        move_to(account, Coin { value: amount });
    }
    
    public fun balance(addr: address): u64 acquires Coin {
        borrow_global<Coin>(addr).value
    }
    
    public fun transfer(from: &signer, to: address, amount: u64) acquires Coin {
        let from_addr = signer::address_of(from);
        let from_balance = &mut borrow_global_mut<Coin>(from_addr).value;
        *from_balance = *from_balance - amount;
        
        let to_balance = &mut borrow_global_mut<Coin>(to).value;
        *to_balance = *to_balance + amount;
    }
}
