mod util;

use time::*;

use util::bank::Bank;
use util::btree::BTree;
use util::threadpool::Pool;

fn main() {
    let mut bank = Bank::new();
    let mut btree = BTree::<i32, i32>::new("./testbtree1.btree");
    let mut p = Pool::new(4);
    let mut isrunning = true;
    // 1555555555 不存在
    // 1222222222 不存在
    // 1333333333 207028
    // 1179572757 105566
    // 2045514596 249706
    // 2064098435 103950
    // 2119601595 205721
    // 2011199898 103037
    // 2085939837 87927

    /*
    演示：
    存款/取款：一个存在一个不存在，计时点的选择，
    转账：一个不存在一个存在
    发工资/发利息：中间插入一个不存在的账号
    以转账为例讲解：
    索引：b+树。建立过程+合并余额的方法，文件形式
    内存缓冲：内存中的账户集合，用互斥锁保护
    线程池
    优先级mpsc channel，如何实现批处理操作的暂停效果
    持久化的方式
    */

    let bank = &mut bank;
    while isrunning {
        let mut line = String::new();
        println!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            "银行记账系统 （20206964 臧家明）",
            "请选择您的操作序号：",
            "1.存款",
            "2.取款",
            "3.转账",
            "4.发工资",
            "5.发利息",
            "6.退出"
        );
        std::io::stdin().read_line(&mut line).unwrap();
        match line.trim().parse::<u32>().unwrap() {
            1 => {
                let mut account = String::new();
                let mut amount = String::new();
                println!("{}", "请输入账号：");
                std::io::stdin().read_line(&mut account).unwrap();
                let account = account.trim().to_string();
                println!("{}", "请输入存款金额：");
                std::io::stdin().read_line(&mut amount).unwrap();
                let amount = amount.trim().parse::<i32>().unwrap();
                let start = Instant::now(); //计时开始
                if !bank.check_account(account.clone()) {
                    match btree.get(&str::parse::<i32>(&account.clone()).unwrap()) {
                        Some(balance) => {
                            bank.add_account(account.clone(), balance);
                            let mut bank = bank.clone();
                            p.execute(
                                move || {
                                    println!("{}", account.clone());
                                    match bank.deposit(account.clone(), amount) {
                                        Ok(()) => {}
                                        Err(err) => println!("{}", err),
                                    };
                                    let duration = start.elapsed(); //操作成功计时点
                                    println!(
                                        "账户{}余额：{}，操作用时{}",
                                        account.clone(),
                                        bank.showbalance(account.clone()),
                                        duration.to_string()
                                    );
                                },
                                true,
                            )
                        }
                        None => {
                            let duration = start.elapsed(); //查询账户不存在操作用时的计时点
                            println!("{}", format!("账号不存在，用时{}", duration.to_string()));
                        }
                    };
                } else {
                    let mut bank = bank.clone();
                    p.execute(
                        move || {
                            println!("{}", account.clone());
                            match bank.deposit(account.clone(), amount) {
                                Ok(()) => {}
                                Err(err) => println!("{}", err),
                            };
                            let duration = start.elapsed(); //操作成功计时点
                            println!(
                                "账户{}余额：{}，操作用时{}",
                                account.clone(),
                                bank.showbalance(account.clone()),
                                duration.to_string()
                            );
                        },
                        true,
                    )
                }
            }
            2 => {
                let mut account = String::new();
                let mut amount = String::new();
                println!("{}", "请输入账号：");
                std::io::stdin().read_line(&mut account).unwrap();
                let account = account.trim().to_string();
                println!("{}", "请输入取款金额：");
                std::io::stdin().read_line(&mut amount).unwrap();
                let amount = amount.trim().parse::<i32>().unwrap();
                let start = Instant::now(); //计时开始
                if !bank.check_account(account.clone()) {
                    match btree.get(&str::parse::<i32>(&account.clone()).unwrap()) {
                        Some(balance) => {
                            bank.add_account(account.clone(), balance);
                            let mut bank = bank.clone();
                            p.execute(
                                move || {
                                    match bank.withdraw(account.clone(), amount) {
                                        Ok(()) => {}
                                        Err(err) => println!("{}", err),
                                    };
                                    let duration = start.elapsed(); //操作成功计时点
                                    println!(
                                        "账户{}余额：{}，操作用时{}",
                                        account.clone(),
                                        bank.showbalance(account.clone()),
                                        duration.to_string()
                                    );
                                },
                                true,
                            )
                        }
                        None => {
                            let duration = start.elapsed(); //操作成功计时点
                            println!("{}", format!("账号不存在，用时{}", duration.to_string()));
                        }
                    };
                } else {
                    let mut bank = bank.clone();
                    p.execute(
                        move || {
                            // println!("{}",account.clone());
                            match bank.withdraw(account.clone(), amount) {
                                Ok(()) => {}
                                Err(err) => println!("{}", err),
                            };
                            let duration = start.elapsed(); //操作成功计时点
                            println!(
                                "账户{}余额：{}，操作用时{}",
                                account.clone(),
                                bank.showbalance(account.clone()),
                                duration.to_string()
                            );
                        },
                        true,
                    )
                }
            }
            3 => {
                let mut fromaccount = String::new();
                let mut toaccount = String::new();
                let mut amount = String::new();
                println!("{}", "请输入付款账户：");
                std::io::stdin().read_line(&mut fromaccount).unwrap();
                let fromaccount = fromaccount.trim().to_string();
                println!("{}", "请输入收款账户：");
                std::io::stdin().read_line(&mut toaccount).unwrap();
                let toaccount = toaccount.trim().to_string();
                println!("{}", "请输入转账金额：");
                std::io::stdin().read_line(&mut amount).unwrap();
                let amount = amount.trim().parse::<i32>().unwrap();

                let start = Instant::now(); //计时开始
                if !bank.check_account(fromaccount.clone()) {
                    match btree.get(&str::parse::<i32>(&fromaccount.clone()).unwrap()) {
                        Some(balance) => {
                            bank.add_account(fromaccount.clone(), balance);
                        }
                        None => {
                            let duration = start.elapsed(); //操作成功计时点
                            println!("付款账号不存在！，操作用时{}", duration.to_string());
                        }
                    }
                }
                if !bank.check_account(toaccount.clone()) {
                    match btree.get(&str::parse::<i32>(&toaccount.clone()).unwrap()) {
                        Some(balance) => {
                            bank.add_account(toaccount.clone(), balance);
                        }
                        None => {
                            let duration = start.elapsed(); //操作成功计时点
                            println!("收款账号不存在！，操作用时{}", duration.to_string());
                        }
                    }
                }
                if bank.check_account(fromaccount.clone()) && bank.check_account(toaccount.clone())
                {
                    let mut bank = bank.clone();
                    p.execute(
                        move || {
                            match bank.transfer(amount, fromaccount.clone(), toaccount.clone()) {
                                Ok(()) => {
                                    let duration = start.elapsed(); //操作成功计时点
                                    println!(
                                        "账户{}余额：{}",
                                        fromaccount.clone(),
                                        bank.showbalance(fromaccount.clone())
                                    );
                                    println!(
                                        "账户{}余额：{}",
                                        toaccount.clone(),
                                        bank.showbalance(toaccount.clone())
                                    );
                                    println!("操作用时{}", duration.to_string())
                                }
                                Err(err) => println!("{}", err),
                            };
                        },
                        true,
                    )
                }

                for (account, balance) in bank.get_accounts() {
                    btree
                        .set(&str::parse::<i32>(&account).unwrap(), &balance)
                        .unwrap();
                }
            }
            4 => {
                let mut accounts = Vec::new();
                let mut flag = true;
                while flag {
                    let mut account = String::new();
                    println!("{}", "请输入账号,输入0结束：");
                    std::io::stdin().read_line(&mut account).unwrap();
                    let account = account.trim().to_string();
                    if account == "0" {
                        flag = false;
                    } else {
                        if !bank.check_account(account.clone()) {
                            match btree.get(&str::parse::<i32>(&account.clone()).unwrap()) {
                                Some(balance) => {
                                    bank.add_account(account.clone(), balance);
                                    accounts.push(account.clone());
                                }
                                None => {
                                    println!("账号不存在！");
                                }
                            };
                        }
                    }
                }
                println!("{}", "正在给每个人发工资！");
                for account in accounts {
                    let mut bank = bank.clone();
                    p.execute(
                        move || {
                            match bank.payroll(account.clone()) {
                                Ok(()) => {
                                    println!(
                                        "账户{}余额：{}",
                                        account.clone(),
                                        bank.showbalance(account.clone())
                                    );
                                }
                                Err(err) => println!("{}", err),
                            };
                        },
                        false,
                    );
                }
            }
            5 => {
                let mut accounts = Vec::new();
                let mut flag = true;
                while flag {
                    let mut account = String::new();
                    println!("{}", "请输入账号,输入0结束：");
                    std::io::stdin().read_line(&mut account).unwrap();
                    let account = account.trim().to_string();
                    if account == "0" {
                        flag = false;
                    } else {
                        if !bank.check_account(account.clone()) {
                            match btree.get(&str::parse::<i32>(&account.clone()).unwrap()) {
                                Some(balance) => {
                                    bank.add_account(account.clone(), balance);
                                    accounts.push(account.clone());
                                }
                                None => {
                                    println!("账号不存在！");
                                }
                            };
                        }
                    }
                }
                println!("{}", "正在给每个人发利息！");
                for account in accounts {
                    let mut bank = bank.clone();
                    p.execute(
                        move || {
                            match bank.payinterest(account.clone()) {
                                Ok(()) => {
                                    println!(
                                        "账户{}余额：{}",
                                        account.clone(),
                                        bank.showbalance(account.clone())
                                    );
                                }
                                Err(err) => println!("{}", err),
                            };
                        },
                        false,
                    );
                }
            }
            6 => {
                isrunning = false;
                for (account, balance) in bank.get_accounts() {
                    btree
                        .set(&str::parse::<i32>(&account).unwrap(), &balance)
                        .unwrap();
                }
                println!("{}", "bye");
            }
            _ => {
                println!("{}", "请重新输入")
            }
        }
    }
}
