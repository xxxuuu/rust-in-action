use clap::{App, Arg};
use nix::sys::uio::RemoteIoVec;
use nix::unistd::Pid;
use std::{
    fs::File,
    io::{BufRead, BufReader, IoSliceMut}, cmp::min,
};
#[derive(Debug)]
struct MapSegment {
    start_addr: usize,
    end_addr: usize,
    file: String,
}

impl MapSegment {
    fn is_anonmyous_map(&self) -> bool {
        self.file == "[stack]" || self.file == "[heap]"
    }
}

fn parse_maps_line(line: String) -> Result<MapSegment, String> {
    let _line: Vec<&str> = line.split(" ").collect();
    if _line.len() < 2 {
        return Err("parse line failed".to_string());
    }

    let addr: Vec<&str> = _line[0].split("-").collect();
    if addr.len() != 2 {
        return Err("parse address failed".to_string());
    }

    let (start_addr, end_addr) = (
        usize::from_str_radix(addr[0], 16).unwrap(),
        usize::from_str_radix(addr[1], 16).unwrap(),
    );
    let file = _line[_line.len() - 1].to_string();

    Ok(MapSegment {
        start_addr: start_addr,
        end_addr: end_addr,
        file: file,
    })
}

fn main() {
    let args = App::new("memscan")
        .version("0.1")
        .about("Scan the address of the specified i32 value in the given process")
        .arg(
            Arg::with_name("pid")
                .help("target process id")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("value")
                .help("specified i32 value")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let pid = Pid::from_raw(args.value_of("pid").unwrap().parse().unwrap());
    let find_val: i32 = args.value_of("value").unwrap().parse().unwrap();

    let f = File::open(format!("/proc/{}/maps", pid)).expect("open file failed");
    let reader = BufReader::new(f);

    for _line in reader.lines() {
        if let Err(_) = _line {
            continue;
        }

        let line = _line.unwrap();
        // println!("{}", line);
        let map_segment = parse_maps_line(line);

        if let Ok(seg) = map_segment {
            println!("{:?}", seg);
            if !seg.is_anonmyous_map() {
                continue;
            }

            println!(
                "scan {} from 0x{:0x} to 0x{:0x}",
                find_val, seg.start_addr, seg.end_addr
            );

            let mut base = seg.start_addr;
            let mut len = min(seg.end_addr - base, 1024*4);
            let mut buf = [0u8; 1024*4];
            let mut local_iov = [IoSliceMut::new(&mut buf)];
            let mut remote_iov = [RemoteIoVec { base, len }];
            while len > 0 {
                let size =
                    nix::sys::uio::process_vm_readv(pid, &mut local_iov, &remote_iov).expect("read process memory failed");

                for i in (0..size).rev().filter(|x| x%4 == 0) {
                    assert!(i+3 < size);
                    let val: i32 = (local_iov[0][i+3] as i32) << 24 
                                | (local_iov[0][i+2] as i32) << 16 
                                | (local_iov[0][i+1] as i32) << 8 
                                | local_iov[0][i] as i32;
                    if val == find_val {
                        println!("found value {} in addr {:0x}", val, base+i);
                    }
                }

                base += size;
                len = min(seg.end_addr - base, 1024*4);
                remote_iov[0].base = base;
                remote_iov[0].len = len;
            }
        }
    }
}
