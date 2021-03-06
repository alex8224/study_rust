extern crate encoding;
extern crate md5;
// extern crate wry;

use std::{env, process::Command, thread::sleep, time::Duration};
use std::{mem::size_of_val, process::exit};

use encoding::{all::GBK, DecoderTrap, Encoding};

use rust_a::{public::*, util::*};
use rust_a::redis_mo::do_redis_code;

#[test]
fn json_test() {
    let mut pnt_list: Vec<Point> = Vec::new();
    let point = Point {
        x: 10,
        y: 20,
        name: "张",
        other: None,
    };
    let json_str = serde_json::to_string(&point).expect("convert to json failed!");
    for idx in 0..100000 {
        let mut pack_json: Point =
            serde_json::from_str(json_str.as_str()).expect("parse str to point failed!");
        // println!("{}, point {:?}, sizeof {}", json_str, &pack_json, size_of_val(&pack_json));
        pack_json.x = idx + 1;
        pack_json.y = pack_json.y * idx;
        pnt_list.push(pack_json);
    }
    let size: usize = pnt_list.into_iter().map(|item| size_of_val(&item)).sum();
    println!("all point memsize is {}", size);
    let empty = Point {
        x: 1,
        y: 2,
        name: "z",
        other: None,
    };
    println!("point memsize is {}", size_of_val(&empty));
}

// use wry::{Application, Result};


fn call_redis() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("need redis url somethisn like: redis://host:port/db")
    } else {
        let url = args[1].as_str();
        println!("redis url is {}", url);
        match do_redis_code(url) {
            Err(err) => {
                println!("Could not execute example:");
                println!("  {}: {}", err.category(), err);
            }
            Ok(()) => {}
        }
    }
}
fn main() {
    // call_redis();
    let args: Vec<String> = env::args().collect();
    println!("{}", args.len());
    for i in 0..args.len() {
        println!("{}", args.get(i).unwrap());
    }
    let g1 = args.get(1).unwrap();
    rust_a::test_dom(g1.parse::<u32>().unwrap());
    // use rust_a::fullindex::call_full_basic;
    // call_full_basic();
}
// fn test_wry() -> wry::Result<()> {
//     let mut app = Application::new()?;
//     app.add_window(Default::default())?;
//     app.run();
//     Ok(())
// }

#[test]
fn test_common() {
    let args: Vec<String> = env::args().collect();
    let mut secs = 1;
    if args.len() == 2 {
        secs = args[1].parse::<u64>().expect("parse args to u64 failed!")
    }
    let duration = Duration::from_secs(secs);
    let mut hex = String::from("a");
    for _ in 1..2 {
        let cmd = Command::new("tasklist")
            .output()
            .expect("spawn process failed!");
        let cmd_out = GBK
            .decode(cmd.stdout.as_slice(), DecoderTrap::Ignore)
            .expect("decode output failed!");
        let mut str_array: Vec<&str> = cmd_out
            .split("\r\n")
            .into_iter()
            .skip(4)
            .filter(|item| !item.starts_with("System Idle Process"))
            .map(|item| {
                if let Some(end_idx) = item.find(" ") {
                    &item[0..end_idx]
                } else {
                    ""
                }
            })
            .collect();
        str_array.sort();
        let joined = str_array.join("\r\n");
        let tmp_hex = format!("{:x}", md5::compute(&joined));
        if hex.eq("a") {
            hex = tmp_hex;
            println!("first get hash {}", hex);
            rust_a::util::write_log(&joined);
        } else if hex.ne(&tmp_hex) {
            println!("got difference hash {}, {}", tmp_hex, hex);
            hex = tmp_hex;
            write_log(&joined);
        }
        sleep(duration);
    }
}
