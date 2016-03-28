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
    let mut keys: Vec<String> = vec![
        String::from("10.108.67.34"),
        String::from("10.108.87.33"),
        String::from("10.108.68.29"),
        String::from("10.108.68.30"),
        String::from("10.108.71.19"),
        String::from("10.108.172.35"),
        String::from("10.108.172.36"),
        String::from("10.108.24.59"),
        String::from("10.108.43.20"),
        String::from("10.108.68.16"),
        String::from("10.108.68.31"),
        String::from("10.108.68.32"),
        String::from("10.108.67.27"),
        String::from("10.108.67.28"),
        String::from("10.108.67.29"),
        String::from("10.108.70.24"),
        String::from("10.108.70.25"),
        String::from("10.108.71.18"),
        String::from("10.108.42.26"),
        String::from("10.108.47.18"),
        String::from("10.108.46.18"),
        String::from("10.108.45.18"),
        String::from("10.108.70.10"),
        String::from("10.108.70.11"),
        String::from("10.108.70.12"),
        String::from("10.112.12.10"),
        String::from("10.105.43.123"),
        String::from("10.105.43.135"),
        String::from("10.105.45.114"),
        String::from("10.105.45.115"),
        String::from("10.101.20.113"),
        String::from("10.101.20.71"),
        String::from("10.101.20.72"),
        String::from("10.106.203.100"),
        String::from("10.101.20.166"),
        String::from("10.101.20.227"),
        String::from("10.105.43.106"),
        String::from("10.105.43.107"),
        String::from("10.105.18.21"),
        String::from("10.105.18.25"),
        String::from("10.105.18.26"),
        String::from("10.105.18.23"),
        String::from("10.108.67.12"),
        String::from("10.108.67.16"),
        String::from("10.105.43.104"),
        String::from("10.105.43.105")];

    let hosts = vec![
        String::from("c3-inn-p01"),
        String::from("c3-inn-p02"),
        String::from("c3-inn-p03"),
        String::from("c3-inn-p04"),
        String::from("c3-inn-p05"),
        String::from("c3-inn-p06"),
        String::from("c3-inn-p07"),
        String::from("c3-inn-p08"),
        String::from("c3-pub-p01"),
        String::from("c3-pub-p02"),
        String::from("c3-pub-p03"),
        String::from("c3-pub-p04"),
        String::from("c3-pub-p05"),
        String::from("c3-pub-p06"),
        String::from("c3-pub-p07"),
        String::from("c3-pub-p08"),
        String::from("c3-pub-p09"),
        String::from("c3-pub-p10"),
        String::from("c3-pub-p11"),
        String::from("c3-pub-p12"),
        String::from("c3-pub-p13"),
        String::from("c3-pub-p14"),
        String::from("c3-pub-p15"),
        String::from("c3-pub-p16"),
        String::from("c3-pub-p17"),
        String::from("c3-pub-p18"),
        String::from("lg-inn-p01"),
        String::from("lg-inn-p02"),
        String::from("lg-inn-p03"),
        String::from("lg-inn-p04"),
        String::from("lg-pub-p01"),
        String::from("lg-pub-p02"),
        String::from("lg-pub-p03"),
        String::from("lg-pub-p04"),
        String::from("lg-pub-p05"),
        String::from("lg-pub-p06"),
        String::from("lg-pub-p07"),
        String::from("lg-pub-p08"),
        String::from("lg-pub-p11"),
        String::from("lg-pub-p12"),
        String::from("lg-pub-p13"),
        String::from("lg-pub-p14"),
        String::from("c3-r01"),
        String::from("c3-r02"),
        String::from("lg-r01"),
        String::from("lg-r02")];

    //for i in &mut keys {
    //    i.push_str(":qps");
    //}

    let mut count = 0;
    loop {
        if count % 20 == 0 {
            for i in hosts.iter() {
                print!("{} ", i);
            }
            println!("| SUM");
        }
        
        let rdsconn = redis::Client::open("redis://10.237.2.170:5001/").unwrap().get_connection().unwrap();

        let mut sum:u64 = 0;
        for (i, k) in keys.iter().enumerate() {
            let mut k = k.clone();
            k.push_str(":qps");

            let v: Result<u32, redis::RedisError> = rdsconn.get(k);
            match v {
                Ok(v) => { sum+=v as u64; print!("{} ", v);},
                Err(e) => (),
            }

            if i == 7 || i == 25 || i == 29 || i == 41 || i == 45 {
                print!("| ");
            }
        }
        print!("{}\n", Unit::new(sum).fmt());
        count+=1;

        thread::sleep(Duration::from_secs(2));
    }
}
