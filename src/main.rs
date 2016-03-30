#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate iron;
extern crate ssh2;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate redis;
// extern crate memchr;

use std::net::{TcpStream, TcpListener, IpAddr, Ipv4Addr};
use std::io::Read;
use std::path::Path;


use ssh2::Session;

mod process;
mod tcpsrv;
mod httpsrv;
mod logfile;


#[derive(Debug, Serialize)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    // let tcp = TcpStream::connect("10.237.2.170:22").unwrap();
    // let mut sess = Session::new().unwrap();

    // sess.handshake(&tcp).unwrap();
    // sess.userauth_password("root", "b2csa").unwrap();
    // assert!(sess.authenticated());


    // let mut ch = sess.channel_session().unwrap();
    // ch.exec("ls").unwrap();

    // let mut s = String::new();
    // ch.read_to_string(&mut s).unwrap();

    // println!("{:?}", s);
    // println!("{:?}", ch.exit_status().unwrap());

    // process::visit_dirs(Path::new("/Users/gaobushuang/projects/monrs/test"), &process::print_dirname);

    // let x = process::Process::new(100).unwrap();

    let ps = process::ps().unwrap();

    let p = Point { x: 1, y: 2 };

    let seri = serde_json::to_string(&ps).unwrap();

    println!("{}", seri);

    /// ////////////////// LogFile ////////////////
    let mut ipaddr: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    {
        let tcpstream = TcpStream::connect("10.237.2.170:5001").unwrap();
        let mut ip: &mut IpAddr = &mut ipaddr;
        *ip = tcpstream.local_addr().unwrap().ip();
    }

    let conn = redis::Client::open("redis://10.237.2.170:5001/").unwrap().get_connection().unwrap();

    // let mut lf = logfile::LogFile::new(String::from("test.log")).unwrap();
    let mut lf = logfile::LogFile::new(format!("{}", ipaddr),
                                       String::from("/home/work/logs/mae/mirouter.log"),
                                       conn)
                     .unwrap();
    // let (rx, stop) = lf.start().unwrap();

    lf.loop_update_qps();

    // loop {
    // 	match lf.receiver.recv() {
    // 		Ok(i) => {
    // 			println!("{}: QPS {}", lf.name, i);
    // 			// lf.stop();
    // 		},
    // 		Err(e) => {
    // 			println!("Got error: {:?}", e);
    // 			break;
    // 		},
    // 	}
    // }

    // never reach here.
    let l = TcpListener::bind("0.0.0.0:8888").unwrap();
    let srv = tcpsrv::TcpServer::new(&l);
    srv.serve();


}
