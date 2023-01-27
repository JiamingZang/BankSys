use std::collections::HashMap;


use std::sync::{Arc, Mutex};

use super::btree;

#[derive(Clone)]
pub struct Bankaccount {
    account_number: String,
    balance:i32,
}
#[derive(Clone)]
pub struct Bank{
    // Arc是rust中的原子引用计数，线程安全的线程间数据共享的操作
    // Mutex互斥锁，保护共享数据
    accounts:HashMap<String,Arc<Mutex<Bankaccount>>>,
    payroll:i32,
    interest:i32,
}

impl Bank{

    pub fn new() -> Self{
        Bank{accounts:HashMap::new(),payroll:200,interest:10}
    }

    pub fn init(&mut self){
        self.accounts.insert("123".to_string(),Arc::new(Mutex::new(Bankaccount{account_number:"123".to_string(),balance: 0})));
        self.accounts.insert("234".to_string(),Arc::new(Mutex::new(Bankaccount{account_number:"234".to_string(),balance: 0})));
        self.accounts.insert("345".to_string(),Arc::new(Mutex::new(Bankaccount{account_number:"345".to_string(),balance: 200})));
        self.accounts.insert("456".to_string(),Arc::new(Mutex::new(Bankaccount{account_number:"456".to_string(),balance: 200})));
    }

    pub fn get_accounts(&self)->Vec<(String,i32)> {
        let mut result = Vec::new();
        for (k,v) in self.accounts.iter() {
            result.push((k.clone(),v.try_lock().unwrap().balance));
        }
        result
    }

    pub fn add_account(&mut self, account: String, amount:i32){
        self.accounts.insert(account.clone(),Arc::new(Mutex::new(Bankaccount{account_number:account,balance: amount})));
    }

    pub fn check_account(&mut self,account:String)->bool {
        if self.accounts.contains_key(&account.clone()) {
            return true
        }
        false
    }

    pub fn deposit(&mut self ,account:String, amount:i32)->Result<(),String>{

        let accounts = &self.accounts;  
        let account = match accounts.get(&account){
            Some(account) => account,
            None => return Err(format!("账户不存在"))
        };
        match account.try_lock().unwrap().deposit(amount) {
            Ok(()) =>{Ok(())},
            Err(err) => Err(err),
        }
    }

    pub fn withdraw(&mut self,account:String,amount:i32)->Result<(),String>{
        let accounts = &self.accounts; 
        let account = match accounts.get(&account){
            Some(account) => account,
            None => return Err(format!("账户不存在"))
        };
        match account.try_lock().unwrap().withdraw(amount) {
            Ok(())=>{Ok(())},
            Err(err) => return Err(err),
        }

    }
    
    pub fn transfer(&mut self,amount:i32,from:String, to:String)->Result<(),String>{
        let accounts = &self.accounts; 
        let fromaccount= accounts.get(&from).unwrap();
        match fromaccount.try_lock().unwrap().withdraw(amount){
            Err(err) =>{
                Err(err)
            },
            _=>{
                let toaccount = accounts.get(&to).unwrap();
                match toaccount.try_lock().unwrap().deposit(amount){
                    Ok(()) =>{Ok(())},
                    Err(err) =>{Err(err)},
                }
            },
        }

    }


    pub fn payroll(&mut self,account:String)->Result<(),String>{
        self.deposit(account,self.payroll)
    }

    pub fn payinterest(&mut self,account:String)->Result<(),String>{
        let accounts = &self.accounts;  
        match accounts.get(&account){
            Some(tempaccount) => {
                let amount = self.showbalance(account.clone())/ self.interest;
                match tempaccount.try_lock().unwrap().deposit(amount) {
                    Ok(()) =>{Ok(())},
                    Err(err) => Err(err),
                }
            },
            None => return Err(format!("账户不存在"))
        }
    }

    pub fn showbalance(&self,account_number: String)->i32{
        self.accounts.get(&account_number).unwrap().try_lock().unwrap().balance
    }
}

impl Bankaccount{
    pub fn deposit(&mut self,amount:i32)->Result<(),String>{
        if amount>0 {
            self.balance += amount;
            Ok(())
        }else{
            Err(format!("存款失败"))
        }
    }

    pub fn withdraw(&mut self,amount:i32)->Result<(),String>{
        if amount<0{
            Err(format!("取款失败"))
        }else if self.balance < amount{
            Err(format!("余额不足"))
        }else{
            self.balance -= amount;
            Ok(())
        }
    }

}


#[cfg(test)]
mod tests {
    use super::btree;

    use super::*;
    use std::{thread, io::{Read, BufReader, BufRead, Write}};
    #[test]
    pub fn test_transfer_succeeds(){
        let mut bank = Bank::new();
        bank.init();
        assert_eq!(bank.transfer(200, "345".to_string(), "123".to_string()),Ok(()));
        assert_eq!(bank.transfer(200, "123".to_string(), "345".to_string()),Ok(()));
        assert_eq!(bank.showbalance("123".to_string()),0);
        assert_eq!(bank.showbalance("345".to_string()),200);
    }

    #[test]
    pub fn test_transfer_fails() {
        let mut bank = Bank::new();
        bank.init();
        assert_eq!(bank.transfer(400, "234".to_string(), "123".to_string()),Err(format!("余额不足")));
        assert_eq!(bank.transfer(-20, "234".to_string(), "123".to_string()),Err(format!("取款失败")));
    }

    #[test]
    pub fn test_deposit_succeeds(){
        let mut bank = Bank::new();
        bank.init();
        assert_eq!(bank.deposit("123".to_string(), 200),Ok(()));
        assert_eq!(bank.showbalance("123".to_string()),200);
    }

    #[test]
    pub fn test_deposit_fails(){
        let mut bank = Bank::new();
        bank.init();
        assert_eq!(bank.deposit("123".to_string(), -20),Err(format!("存款失败")));
        assert_eq!(bank.deposit("3245".to_string(), -20),Err(format!("账户不存在")));
    }

    #[test]
    pub fn test_withdraw_succeeds(){
        let mut bank = Bank::new();
        bank.init();
        assert_eq!(bank.withdraw("345".to_string(), 200),Ok(()));
    }

    #[test]
    pub fn test_withdraw_fails() {
        let mut bank = Bank::new();
        bank.init();
        assert_eq!(bank.withdraw("123".to_string(), -20),Err(format!("取款失败")));
        assert_eq!(bank.withdraw("123".to_string(), 400),Err(format!("余额不足")));
        assert_eq!(bank.withdraw("3435".to_string(), 400),Err(format!("账户不存在")));
    }

    #[test]
    pub fn test_payroll_and_interest(){
        let mut bank = Bank::new();
        bank.init();
        assert_eq!(bank.payroll("123".to_string()),Ok(()));
        assert_eq!(bank.payroll("111".to_string()),Err(format!{"账户不存在"}));
        assert_eq!(bank.payinterest("345".to_string()),Ok(()));
        assert_eq!(bank.payinterest("111".to_string()),Err(format!{"账户不存在"}));
    }


    #[test]
    pub fn test_transfer_threads(){
        let mut bank =Bank::new();
        bank.init();
        let mut handles = vec![];
        for accounnnumber in vec!["234".to_string(), "345".to_string(),"456".to_string()]{
            let mut bank = bank.clone();
            let handle =  thread::spawn(move || {
                // bank.payroll("123".to_string());
                println!("{}",accounnnumber.clone());
                match bank.transfer(20, accounnnumber.clone(), "123".to_string()){
                    Ok(())=>{},
                    Err(err) => println!("{}",err),
                };
                println!("{}",bank.showbalance("123".to_string()));
            });
            handles.push(handle);
        }
        for handle in handles{
            handle.join().unwrap();
        }

    }

    // #[test]
    // pub fn test_btree_(){
    //     let mut btree = btree::BTree::<i32, i32>::new("./testbtree1.btree");
    //     let mut file = std::fs::File::open("account1.csv").unwrap();
    //     let reader = BufReader::new(file);
    //     for line in reader.lines() {
    //         // line 是 std::result::Result<std::string::String, std::io::Error> 类型
    //         // line 不包含换行符
    //         let line = line.unwrap();
    //         let line:Vec<&str> = line.split(',').collect();
    //         let account = line[0];
    //         let balance = line[1];
    //         match btree.get(&str::parse::<i32>(account).unwrap()) {
    //             Some(abalance) =>{
    //                 btree.set(&str::parse::<i32>(account).unwrap(),&(str::parse::<i32>(balance).unwrap()+abalance)).unwrap();
    //             },
    //             None =>{
    //                 btree.set(&str::parse::<i32>(account).unwrap(),&(str::parse::<i32>(balance).unwrap())).unwrap();
    //             }
    //         }
    //     }
    // }

    #[test]
    pub fn test_check_account(){
        let mut bank = Bank::new();
        bank.init();
        assert_eq!(bank.check_account("123".to_string()),true);
        assert_eq!(bank.check_account("222".to_string()),false);
    }

    #[test]
    pub fn test_add_account(){
        let mut bank = Bank::new();
        bank.init();
        assert_eq!(bank.check_account("222".to_string()),false);
        bank.add_account("222".to_string(),222);
        assert_eq!(bank.check_account("222".to_string()),true);
        assert_eq!(bank.showbalance("222".to_string()),222);
    }
}