extern crate hashers;

pub mod public {
    use serde::{Serialize, Deserialize};

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
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        match File::create(format!("{}.txt", now)) {
            Ok(mut file) => file.write(data.as_bytes()),
            Err(_e) => panic!("create file failed!"),
        }.unwrap();
    }
}

#[cfg(test)]
pub mod test {
    use std::{collections::HashMap, mem::size_of};
    use std::hash::BuildHasherDefault;
    use std::time::SystemTime;
    use hashers::fx_hash::FxHasher;

    #[test]
    fn test_map() {
        let size = 10_000_00;
        let mut map = HashMap::<i32, &str, _>::with_capacity_and_hasher(size, BuildHasherDefault::<FxHasher>::default());
        let hello = "hello";
        let start = SystemTime::now();
        for i in 1..size {
            map.insert(i as i32, hello);
        }
        println!("duration time is {}ms", start.elapsed().unwrap().as_millis());
        map.clear();
    }

    use std::fmt::Debug;
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
} 