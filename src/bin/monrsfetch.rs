extern crate redis;

use std::thread;
use std::time::Duration;

use redis::Commands;

enum Unit {
    K(f64),
    M(f64),
    G(f64),
    B(u32),
}

impl Unit {
    fn new(i: u64) -> Unit {
        if i / 1000000000 > 0 {
            Unit::G(i as f64 / 1000000000.0)
        } else if i / 1000000 > 0 {
            Unit::M(i as f64 / 1000000.0)
        } else if i / 1000 > 0 {
            Unit::K(i as f64 / 1000.0)
        } else {
            Unit::B(i as u32)
        }
    }

    fn fmt(&self) -> String {
        match *self {
            Unit::K(i) => format!("{}K", i),
            Unit::M(i) => format!("{}M", i),
            Unit::G(i) => format!("{}G", i),
            Unit::B(i) => format!("{}", i),
        }
    }
}


fn main() {
    let mut keys: Vec<String> = vec![String::from("10.108.67.34"), String::from("10.105.43.105")];

    let hosts = vec![String::from("inn-p01"), String::from("lg-r02")];

    loop {
        let rdsconn = match redis::Client::open("redis://10.237.2.170:5001/")
            .unwrap()
            .get_connection() {
            Ok(c) => {
                println!("connect...");
                c
            }
            Err(e) => {
                println!("Connection redis error: {}", e.to_string());
                thread::sleep(Duration::from_secs(2));
                continue;
            } 
        };

        let mut count = 0;
        loop {
            if count % 20 == 0 {
                for i in hosts.iter() {
                    print!("{} ", i);
                }
                println!("| SUM");
            }


            let mut sum: u64 = 0;
            'inner: for (i, k) in keys.iter().enumerate() {
                let mut k = k.clone();
                k.push_str(":qps");

                let v: Result<u32, redis::RedisError> = rdsconn.get(k);
                match v {
                    Ok(v) => {
                        sum += v as u64;
                        print!("{} ", v);
                    }
                    Err(e) => {
                        println!("redis get error: {}", e.to_string());
                        break 'inner;
                    }
                }

                if i == 7 || i == 25 || i == 29 || i == 41 || i == 45 {
                    print!("| ");
                }
            }
            print!("{}\n", Unit::new(sum).fmt());
            count += 1;

            thread::sleep(Duration::from_secs(2));
        }
    }
}
