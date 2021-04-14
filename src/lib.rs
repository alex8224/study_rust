extern crate hashers;
extern crate redis;

pub mod clourse;
pub mod fullindex;
pub mod live;
pub mod map;
pub mod pointer;
pub mod redis_conn;
pub mod redis_mo;
pub mod zen;

pub mod public {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Point<'a> {
        pub x: i32,
        pub y: i32,
        pub name: &'a str,
        pub other: Option<Box<Point<'a>>>,
    }
}

pub mod util {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{fs::File, io::Write};

    pub fn write_log(data: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        match File::create(format!("{}.txt", now)) {
            Ok(mut file) => file.write(data.as_bytes()),
            Err(_e) => panic!("create file failed!"),
        }
        .unwrap();
    }
}

#[cfg(test)]
pub mod test {
    use std::fmt::Debug;
    use std::hash::BuildHasherDefault;
    use std::time::SystemTime;
    use std::{collections::HashMap, mem::size_of};

    use hashers::fx_hash::FxHasher;
    use redis::{Commands, Connection};

    #[test]
    fn test_map() {
        let size = 10_000_00;
        let mut map = HashMap::<i32, &str, _>::with_capacity_and_hasher(
            size,
            BuildHasherDefault::<FxHasher>::default(),
        );
        let hello = "hello";
        let start = SystemTime::now();
        for i in 1..size {
            map.insert(i as i32, hello);
        }
        println!(
            "duration time is {}ms",
            start.elapsed().unwrap().as_millis()
        );
        map.clear();
    }

    fn debug<T: Debug>(t: T) {
        println!("{:?}", t);
    }

    fn debug_ref<T: Debug + ?Sized>(t: &T) {
        println!("{:?}", t);
    }

    #[test]
    fn test_sized() {
        debug("pass str with sized");
    }

    #[test]
    fn test_unsized() {
        debug_ref("pass str with unsized!");
    }

    struct Sample {}

    #[test]
    fn test_size_assert() {
        const WIDTH: usize = size_of::<&()>();
        const DOUBLE_WIDTH: usize = 2 * WIDTH;
        println!("{}", size_of::<Sample>());
        assert_eq!(WIDTH, size_of::<&Sample>());
    }

    struct ByteIter<'a> {
        remainer: &'a [u8],
    }

    impl<'a> ByteIter<'a> {
        fn next(&mut self) -> Option<&'a u8> {
            if self.remainer.is_empty() {
                None
            } else {
                let byte = &self.remainer[0];
                self.remainer = &self.remainer[1..];
                Some(byte)
            }
        }
    }

    #[test]
    fn test_lifetime() {
        let mut bytes = ByteIter { remainer: b"12" };
        let byte = &bytes.next();
        let byte2 = &bytes.next();
        if byte == byte2 {
            println!("equals both byte")
        }
    }

    fn do_redis_op(conn: &mut Connection) {
        let _: () = conn.set("a", "1").unwrap();
        println!("seted.!");
    }

    #[test]
    fn test_redis() {
        use std::time::Duration;
        let conn =
            redis::Client::open("redis://192.168.10.217:6379/0").expect("get redis conn failed!");
        let mut real_conn = conn
            .get_connection_with_timeout(Duration::from_secs(1))
            .unwrap();
        do_redis_op(&mut real_conn);
    }
}
