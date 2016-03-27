
use std::io::{self, BufRead, BufReader};
use std::fs::{self, File, DirEntry, Metadata};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::fmt::{self, Display};

use regex::Regex;

use serde;


pub fn print_dirname(ent: &DirEntry) {
	println!("{:?}", ent.file_name());
}

pub fn ps() -> Result<Vec<Process>, String> {
	let pids = match visit_dirs(Path::new("/proc")) {
		Ok(i) => i,
		Err(e) => return Err(e),
	};

	let mut processes: Vec<Process> = Vec::new();
	for pid in pids {
		Process::new(pid).map(|p| processes.push(p));
	}

	Ok(processes)
	
}

fn visit_dirs(dir: &Path) -> Result<Vec<Pid>, String> {
	match fs::metadata(dir) {
		Ok(dm) => {
			if !dm.is_dir() {
				 return Err(String::from("not a dir"));
			}
		},
		Err(e) => return Err(e.to_string()),
	}

	
	let mut pids: Vec<Pid> = Vec::new();

	let ens = fs::read_dir(dir);
	match ens {
		Ok(es) => {

			for ent in es {
				let ent = ent.unwrap();
				match ent.metadata() {
					Ok(d) => {
						if !d.is_dir() {
							continue;
						}
						let filename = ent.file_name();
						if filename.to_str().is_none() {
						    continue;
						}
						filename.to_str().unwrap().parse::<Pid>().map(|i| pids.push(i));
					},
					Err(e) => continue,
				}
			}

		},
		Err(e) => return Err(e.to_string()),
	}

	Ok(pids)
}

pub type Pid = u32;

#[derive(Deserialize, Debug, Clone)]
pub enum Status {
	// uninterruptible sleep (usually IO)
	D,

	// running or runnable (on run queue)
	R,

	// interruptible sleep (waiting for an event to complete)
	S,

	// stopped, either by a job control signal or because it is being traced
	T,

	// paging (not valid since the 2.6.xx kernel)
	W,

	// dead (should never be seen)
	X,

	// defunct ("zombie") process, terminated but not reaped by its parent
	Z,

	Unknown,
}

impl Status {
	pub fn from_str(s :&str) -> Status {
		match s {
			"D" | "d" => Status::D,
			"R" | "r" => Status::R,
			"S" | "s" => Status::S,
			"T" | "t" => Status::T,
			"W" | "w" => Status::W,
			"X" | "x" => Status::X,
			"Z" | "z" => Status::Z,
				   _  => Status::Unknown,
		}
	}

	pub fn to_str(&self) -> &str {
		match *self {
			Status::D => "D",
			Status::R => "R",
			Status::S => "S",
			Status::T => "T",
			Status::W => "W",
			Status::X => "X",
			Status::Z => "Z",
					_ => "unknown",
		}
	}
}

impl serde::Serialize for Status {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
		where S: serde::Serializer 
		{
			serializer.serialize_str(self.to_str())
	}
}

// pub type ProcessErrorResult = Result<Process, ProcessError>;
pub type ProcessErrorResult = Result<Process, ProcessError>;

#[derive(Debug)]
pub enum ProcessError {
	ProcessNonExsit,
	ProcessProcNotDir,
	ProcessErrorUnknow,
	ProcessErrorOther(String),
}

impl Display for ProcessError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

// impl Error for ProcessError {
// 	pub fn description(&self) -> &str {
// 		match *self {
// 			ProcessError::ProcessNonExsit => "process not exsit",
// 			ProcessError::ProcessProcNotDir => "proc of process not dir",
// 			ProcessError::ProcessErrorUnknow => "unknow",
// 		}
// 	}
// }



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Process {
	name: String,
	state: Status,
	pid: Pid,
	ppid: Pid,
	vmsize: u64, //KB 
	vmrss: u64, //KB

}

impl Process {
	pub fn new(pid: Pid) -> ProcessErrorResult {
		parse_proc_process(pid)
	}
}


fn parse_proc_process(pid: Pid) -> ProcessErrorResult {
	let mut pf = PathBuf::from(format!("/proc/{}", pid));

	let md = pf.metadata().map_err(|x| ->ProcessError { ProcessError::ProcessErrorOther(x.to_string())});
	match md {
		Ok(i) => {
			if !i.is_dir() {
				return Err(ProcessError::ProcessProcNotDir)
			}

			pf.push("status");

			let mut p = Process{
				name: String::new(),
				state: Status::Unknown,
				pid: 0,
				ppid: 0,
				vmsize: 0,
				vmrss: 0,
			};

			match File::open(pf) {
				Ok(f) => {
					let re = Regex::new(r"^(\w+):\s+(\S+)\s*.*$").expect("regex new");

					let mut f_status = BufReader::new(&f);
					for line in f_status.lines() {
						let l = line.expect("line");

						let cap = match re.captures(&l) {
							Some(i) => i,
							None => continue,
						};

						let key = cap.at(1).unwrap();
						let value = cap.at(2).unwrap();

						match key {
							"Name" => p.name = value.to_string(),
							"State" => p.state = Status::from_str(value),
							"Pid"  => p.pid = value.parse::<Pid>().unwrap(),
							"PPid" => p.ppid = value.parse::<Pid>().unwrap(),
							"VmSize" => p.vmsize = value.parse::<u64>().unwrap(),
							"VmRSS" => p.vmrss = value.parse::<u64>().unwrap(),
							_ => continue,
						};

					}

					// println!("{:?}", p);
					Ok(p)
				},
				Err(e) => return Err(ProcessError::ProcessErrorOther(e.to_string())),
			}

		},

		Err(e) => Err(e),
	}
}

