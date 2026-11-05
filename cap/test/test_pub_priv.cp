import { print } from "cap:io";

class BankAccount {
    priv balance: i32;
    pub owner: string;

    pub fn init(owner: string, initial_balance: i32) {
        this.owner = owner;
        this.balance = initial_balance;
    }

    pub fn deposit(mut this, amount: i32): i32 {
        this.balance += amount;
        return this.get_balance();
    }

    priv fn get_balance(this): i32 {
        return this.balance;
    }

    pub fn check_balance(this): i32 {
        return this.get_balance();
    }
}

pub fn main(): void {
    let mut acc = new BankAccount("Alice", 100);
    print(acc.owner);
    let b1 = acc.check_balance();
    print(b1);
    let b2 = acc.deposit(50);
    print(b2);
}
