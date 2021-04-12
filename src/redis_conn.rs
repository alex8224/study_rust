
use std::{collections::HashMap, thread::current};

use redis::{Commands, FromRedisValue};

use crate::*;

pub trait Connection { 
    fn init_db_info(&mut self);
}


#[derive(Debug)]
struct ConnectionHolder<T: Connection> {
    conns: Vec<T>, //
    current: i32, //current redis connection index within conns
}

type RedisCon = redis::Connection;

impl Connection for redis::Connection {
    fn init_db_info(&mut self) {
        todo!()
    }
}

struct DashBorad {
    dbs: u32, //how many dbs in
}

impl DashBorad {

}

impl ConnectionHolder<redis::Connection> {

    fn new() -> ConnectionHolder<redis::Connection> {
        Self {
            conns: Vec::<redis::Connection>::new(),
            current:-1,
        }
    }

    fn put(&mut self, uri: &str) -> redis::RedisResult<()>{
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

        let ret:Vec<String> = self.conns[0 as usize].keys("*").unwrap();

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

    fn list_db(&mut self) -> usize {
        let mut cur_conn = &mut self.conns[self.current as usize];
        let dbs: HashMap<String, usize>= redis::cmd("config").arg("GET").arg("databases").query(cur_conn).unwrap();
        *dbs.get("databases").unwrap()
    }
    
    fn get_cfg<T: FromRedisValue>(&mut self, key: &str) -> Option<&T> {
        let mut cur_conn = &mut self.conns[self.current as usize];
        let cfg: HashMap<String, T> = redis::cmd("config").arg("get").arg(key).query(cur_conn).unwrap();
        let val = cfg.get(key).unwrap();
        let t = val.to_owned();
        Some(t)
    }
}


#[test]
fn test_create_connectholder() {
    let mut holder = ConnectionHolder::new();
    holder.put("redis://192.168.10.217:6379/1").unwrap();
    println!("{}, db size {}", holder.size(), holder.list_db());
    
    let dbfile:String  = holder.get_cfg("dbfilename");
    // let dbname = dbfile.get("dbfilename").unwrap();
    println!("dbfile name is {}", dbfile);
    
 }