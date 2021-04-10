use std::{collections::HashMap, fmt::Display, net::ToSocketAddrs};

use hashers::{builtin::DefaultHasher, fnv::FNV1aHasher64, fx_hash::FxHasher};
use std::hash::BuildHasherDefault;
use std::time::SystemTime;

const SIZE: usize = 10000;

fn test_map_with_hasher<T>(hasher: BuildHasherDefault<T>)
where
    T: std::hash::Hasher + Default,
{
    let mut map = HashMap::<i32, &str, _>::with_capacity_and_hasher(SIZE, hasher);
    let hello_str = "hello";
    for _ in 0..100 {
        let now = SystemTime::now();
        for i in 1..SIZE {
            map.insert(i as i32, hello_str);
        }
        println!("耗时 {} map val size", now.elapsed().unwrap().as_millis());
        map.clear();
    }
}

#[test]
fn test_fx_hasher() {
    let hasher = BuildHasherDefault::<FxHasher>::default();
    test_map_with_hasher(hasher);
}

#[test]
fn test_fnv64_hasher() {
    let hasher = BuildHasherDefault::<FNV1aHasher64>::default();
    test_map_with_hasher(hasher);
}

#[test]
fn test_default_hasher() {
    let hasher = BuildHasherDefault::<DefaultHasher>::default();
    test_map_with_hasher(hasher);
}

#[test]
fn test_hash24() {
    let hasher = BuildHasherDefault::<std::collections::hash_map::DefaultHasher>::default();
    test_map_with_hasher(hasher);
}

#[derive(Debug)]
struct Person {
    name: String,
}

impl Person {
    fn new<S: Into<String>>(arg: S) -> Person {
        Person { name: arg.into() }
    }
}

#[test]
fn test_into() {
    let p1 = Person::new("aaa");
    let p2 = Person::new("string".to_string());
    println!("{:?}, {:?}", p1, p2);
}
