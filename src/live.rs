
trait WhatEver {}

#[derive(Debug)]
struct LifeCycle(u32);

impl LifeCycle {
    fn new(val: u32) -> Self {
        println!("instance val {}", val);
        Self(val)
    }
}

impl WhatEver for LifeCycle {
}

impl Drop for LifeCycle {
    fn drop(&mut self) {
        println!("{:?} droped", self);
    }
}

fn nop_drop<T>(t: T) where T: WhatEver {

}

fn test_str<'a, 'b: 'a>(x: &'a str, y: &'b str) -> &'a str {
    if true {
        x
    }else{
        y
    }
}

#[test]
fn test_lifecycle() {

    let val = test_str("a", "b");
    println!("get ret val {}", val);
    let l1 = LifeCycle::new(0);
    nop_drop(l1);
    for i in 1..10 {
        let val = LifeCycle::new(i);
    }

    println!("exit.")
}

