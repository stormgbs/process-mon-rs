use std::io::prelude::*;
use std::fs::File;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use std::ops::Drop;
use std::io::SeekFrom;
use std::time::Duration;
use std::collections::HashMap;
use std::cell::RefCell;

use redis::Commands;
use redis::{Client, Connection};
use redis::RedisError;

use iron::prelude::*;
use iron::{status, Handler};

// #[derive(Debug)]
pub struct LogFile {
    pub name: String,
    ip: String,
    // rdsclient: Client,
    conn: Connection,

    qps: Mutex<RefCell<i32>>,

    receiver: Mutex<Receiver<u32>>,
    stop: Mutex<Sender<bool>>,
}


pub type LogFileResult = Result<LogFile, String>;

const N: u8 = '\n' as u8;

impl LogFile {
    pub fn new(ip: String, name: String, conn: Connection) -> LogFileResult {
        let (rx, stop) = watch_log_qps(&name).unwrap();

        Ok(LogFile {
            name: name,
            ip: ip,
            conn: conn,

            qps: Mutex::new(RefCell::new(-1)),

            receiver: Mutex::new(rx),
            stop: Mutex::new(stop),
        })
    }

    pub fn stop(&self, exit: bool) {
        let stop = self.stop.lock().unwrap();
        if exit {
            stop.send(true);
        } else {
            stop.send(false);
        }
    }

    pub fn loop_update_qps(&self) {
        loop {
            // self.receiver
            //    .lock()
            //    .map_err(|e| Err("".to_owned()));

            match self.receiver.lock().unwrap().recv() {
                Ok(i) => {
                    {
                        let mut bqps = self.qps.lock().unwrap();
                        *(*bqps).borrow_mut() = i as i32;
                    }

                    let qps = self.qps.lock().unwrap();
                    println!("{}: QPS {:?}", self.name, *(*qps).borrow());

                    self.write_redis(String::from("qps"), i);
                }
                Err(e) => {
                    println!("Got error: {:?}", e);
                    break;
                }
            }
        }
    }

    fn write_redis(&self, key: String, value: u32) {
        let mut full_key = self.ip.clone();
        full_key.push(':');
        full_key.push_str(&*key);
        let _: Result<u32, RedisError> = self.conn.set(full_key, value);
    }
}

fn open_seek_end(name: &String) -> Result<File, String> {
    File::open(&name)
        .map_err(|e| e.to_string())
        .and_then(|mut f| {
            f.seek(SeekFrom::End(0))
                .map_err(|e| e.to_string())
                .and_then(|_| Ok(f))
        })
}

fn watch_log_qps(name: &String) -> Result<(Receiver<u32>, Sender<bool>), String> {
    let (notify_tx, notify_rx) = mpsc::channel::<bool>();

    let (ch_send, ch_recv) = mpsc::channel::<u32>();
    let sender = ch_send.clone();

    let name = name.to_owned();

    thread::spawn(move || {
        let mut buf: Vec<u8> = vec![0; 10*1024*1024];

        'outer: loop {
            let mut fp = match open_seek_end(&name) {
                Ok(f) => f,
                Err(e) => {
                    println!("Open and seek file error: {:?}", e);
                    return ();
                }
            };

            let mut zero_cnt = 0i32;
            'inner: loop {
                match notify_rx.try_recv() {
                    Ok(t) => {
                        if !t {
                            break 'inner;
                        } else {
                            break 'outer;
                        }
                    }
                    Err(_) => (),
                }

                thread::sleep(Duration::from_secs(2));

                let size = match fp.read(&mut buf) {
                    Ok(size) => size,
                    Err(e) => {
                        println!("read error {}", e.to_string());
                        0
                    }
                };

                if size == 0 {
                    sender.send(0);

                    zero_cnt += 1;
                    if zero_cnt >= 3 {
                        println!("INFO: reloading log file {}", &name);
                        break 'inner;
                    }

                    continue;
                }

                let mut qps: u32 = 0;
                let mut itr = buf[..size].into_iter();

                loop {
                    match itr.next() {
                        Some(&N) => {
                            qps += 1;
                        }
                        Some(_) => {}
                        None => break,
                    }
                }



                let qps = if qps > 0 && qps / 2 == 0 { 1 } else { qps / 2 };



                match sender.send(qps) {
                    // Ok(_) => println!("QPS: {}", qps),
                    Ok(_) => {}
                    Err(e) => println!("Send err: {:?}", e),
                };
            }
        }
    });

    Ok((ch_recv, notify_tx))
}

// impl Handler for LogFile {
// 	fn handle(&self, req: &mut Request) -> IronResult<Response> {
// 		let qps = self.qps.lock().unwrap();
// 		Ok(Response::with((status::Ok, *(*qps).borrow())))
// 	}
// }

// impl Drop for LogFile {
// 	fn drop(&mut self) {
// 		// close threads
// 		// close file
// 		drop();
// 	}
// }

// struct Router {
// 	routes: HashMap<String, LogFile>,
// }

// impl Router {
// 	fn new() -> Self {
// 		Router{ routes: HashMap::new() }
// 	}

// 	fn add(&mut self, logfile_name: String, lf: LogFile) {
// 		self.routes.insert(logfile_name, lf);
// 	}
// }


// impl Handler for Router {
// 	fn handle(&self, _: &mut Request) -> IronResult<Response> {
// 		match self.routes.get("/home/work/logs/mae/mirouter.log") {
// 			Some(lf) => {
// 				let qps = lf.qps.lock().unwrap();
// 				Ok(Response::with( (status::Ok, format!("{:?}", (*qps).clone() )) ))
// 			},
// 			None => Ok(Response::with(status::NotFound)),
// 		}
// 	}
// }
