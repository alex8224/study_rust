use std::{collections::HashMap, thread::current};

use redis::{from_redis_value, Commands, ConnectionLike, FromRedisValue, RedisResult, ToRedisArgs};

use crate::*;
use std::fmt::Error;

pub trait Connection {
    fn init_db_info(&mut self);
}

struct ConnectionHolder {
    conns: Vec<redis::Connection>, //
    current: i32,                  //current redis connection index within conns
}

impl Connection for redis::Connection {
    fn init_db_info(&mut self) {
        todo!()
    }
}

struct DashBorad {
    dbs: u32, //how many dbs in
}

impl DashBorad {}

type CliResult<T> = Result<T, Error>;

impl ConnectionHolder {
    fn new() -> ConnectionHolder {
        Self {
            conns: Vec::<redis::Connection>::new(),
            current: -1,
        }
    }

    fn put(&mut self, uri: &str) -> redis::RedisResult<()> {
        let client = redis::Client::open(uri)?;
        let mut conn = client.get_connection()?;
        self.conns.push(conn);
        self.current = (self.conns.len() - 1) as i32;
        Ok(())
    }

    fn size(&self) -> usize {
        self.conns.len()
    }

    //TODO direct forward request to backend and read resp?
    fn execute(&mut self, cmd: &str) {
        let ret: Vec<String> = self.conns[0 as usize].keys("*").unwrap();
    }

    fn close(&mut self, index: usize) {
        let conn = &self.conns.remove(index);
        self.current = self.conns.len() as i32;
    }

    fn close_all(&mut self) {
        for index in 0..self.conns.len() {
            self.conns.remove(index as usize);
        }
        self.current = -1;
    }

    fn cur_conn(&mut self) -> Option<&mut redis::Connection> {
        if self.current > -1 {
            let conn = &mut self.conns[self.current as usize];
            Some(conn)
        } else {
            None
        }
    }

    fn list_db(&mut self) -> usize {
        let cur_conn = self.cur_conn().unwrap();
        let dbs: HashMap<String, usize> = redis::cmd("config")
            .arg("GET")
            .arg("databases")
            .query(cur_conn)
            .unwrap();
        *dbs.get("databases").unwrap()
    }

    fn query<RV: FromRedisValue>(&mut self, cmd_name: &str, p: Vec<&str>) -> RedisResult<RV> {
        let mut cmd = redis::cmd(cmd_name);
        let cur_conn = self.cur_conn().unwrap();
        for i in 0..p.len() {
            cmd.arg(p[i]);
        }
        cmd.query(cur_conn)
    }

    fn get_cfg<T: FromRedisValue>(&mut self, key: &str) -> HashMap<String, T> {
        let cur_conn = self.cur_conn().unwrap();
        let cfg: HashMap<String, T> = redis::cmd("config")
            .arg("get")
            .arg(key)
            .query(cur_conn)
            .unwrap();
        cfg
    }
}

#[test]
fn test_create_connectholder() {
    let mut holder = ConnectionHolder::new();
    holder.put("redis://192.168.10.217:6379/1").unwrap();
    let cfg_names = vec!["dbfilename", "logfile", "databases", "port", "*max*"];
    for i in 1..cfg_names.len() {
        let map = holder.get_cfg::<String>(cfg_names[i]);
        match map.get(cfg_names[i]) {
            Some(t) => {
                println!("{}={}", cfg_names[i], t);
            }
            None => {
                println!("no val for key {}", cfg_names[i]);
                ()
            }
        };
    }

    // let a = holder.cmd::<&str, String>("set", vec!["a", "aa"]);
}

fn detect_java_ver(bin: &[u8]) -> (&str, u16) {
    let magic = &bin[0..2];
    if magic[0] == 0xac && magic[1] == 0xed {
        let minor = &bin[2..4];
        let ver:u16 = u16::from_be_bytes([minor[0], minor[1]]);
        ("java searialize format",  ver)
    } else {
        ("other format", 999)
    }
}


fn write_file(data: &[u8]) {
    use std::fs::File;
    use std::io::prelude::*;
    let mut f = File::create("d:/uk.data").unwrap();
    f.write_all(data).unwrap();
}

#[test]
fn test_any_cmd_with_strargs() -> RedisResult<()> {
    let mut holder = ConnectionHolder::new();
    holder.put("redis://192.168.10.217:6379/0")?;
    holder.query("set", vec!["a", "1"])?;
    let get_val = holder.query::<String>("get", vec!["a"])?;
    println!("get val is {}", get_val);
    holder.query::<i32>("del", vec!["a"]).unwrap();
    println!(
        "after del then get val is {}",
        holder.query::<String>("get", vec!["a"]).is_ok()
    );

    let incr:u32 = holder.query("incr", vec!["incr_key"])?;
    println!("incr val is {}", incr);

    let decr: u32 = holder.query("decr", vec!["incr_key"])?;
    println!("incr val is {} after decr", decr);

    let _: u8 = holder.query("hset", vec!["myset", "name", "啧啧啧"])?;
    let hset_val:String = holder.query("hget", vec!["myset", "name"])?;
    println!("hset val is {}", hset_val);
    let _: u8 = holder.query("hdel", vec!["myset", "name"])?;

    let keys: Vec<String> = holder.query("keys", vec!["*"])?;
    for i in 0..keys.len() {
        let val_type: String = holder.query("type", vec![&keys[i]])?;
        println!("{} = {}", keys[i], val_type);

        match val_type.as_str() {
            "set" => {
                let set_size: usize = holder.query("scard", vec![&keys[i]])?;
                println!("set {}'s size is {}", keys[i], set_size);
                let members: Vec<String> = holder.query("smembers", vec![&keys[i]])?;
                for j in 0..members.len() {
                    println!("\t member {} of key {} ", members[j], keys[i]);
                }
            }
            "hash" => {
                let hash_len: usize = holder.query("hlen", vec![&keys[i]])?;
                println!("hash {}'s size is {}", keys[i], hash_len);
                let map: HashMap<String, Vec<u8>> = holder.query("hgetall", vec![&keys[i]])?;
                map.into_iter().for_each(|f| {
                    // println!("\t {} = {:?}", f.0, f.1);
                    let (format, ver) = detect_java_ver(&f.1);
                    if format.starts_with("java") {
                        println!("\tjava objectserialize stream, ver {}", ver);
                    }else{
                        // write_file(&f.1);
                        println!("\t other format maybe str");
                    }
                });
            }
            _ => (),
        }
    }
    Ok(())
}

#[test]
fn test_ret_ref() {
    let str = "hello";
    let ptr = str.as_ptr();
    let len = str.len();
    println!("{:p}", ptr);
    println!("{}", len);

    let a: i32 = 0;
    println!("a is {}", a.is_positive());
}

#[derive(Debug, PartialEq)]
struct Foo(i32);
#[derive(Debug, PartialEq)]
struct Bar(i32, i32);

trait Inst {
    fn new(n: i32) -> Self;
}

impl Inst for Foo {
    fn new(n: i32) -> Self {
        Foo(n)
    }
}

impl Inst for Bar {
    fn new(n: i32) -> Self {
        Bar(n, n + 10)
    }
}

fn foobar<T: Inst>(n: i32) -> T {
    T::new(n)
}

#[test]
fn infer() {
    let foo = foobar::<Foo>(10);
    let bar = foobar::<Bar>(20);
}
