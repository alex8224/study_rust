#[test]
fn zen_test() {
    println!("rust's zen : ownership, borrow, trait.....");
}

trait Add<RHS = Self> {
    type Output;
    fn add(self, rhs: RHS) -> Self::Output;
}

trait Hello {
    fn hello(&self);
}

impl Hello for u32 {
    fn hello(&self) {
        println!("hello {}", self);
    }
}

impl Add<u64> for u32 {
    type Output = u64;
    fn add(self, other: u64) -> Self::Output {
        self as u64 + other
    }
}

fn static_dispatch<T: Hello>(t: &T) {
    t.hello();
}

fn dyn_dispatch(t: &Hello) {
    t.hello();
}

#[test]
fn test_op_add() {
    let a: u64 = 1;
    let b: u32 = 100;
    let c = b.add(a);
    println!("{}", c);
}

#[test]
fn test_dispatch() {
    let i: u32 = 1;
    let i2: u32 = 10;
    static_dispatch(&i);
    dyn_dispatch(&i2);
}

struct Margin {
    a: i32,
    b: i32,
    c: char,
}

#[test]
fn test_margin() {
    println!("size of {} ", std::mem::size_of::<Margin>());
}

#[test]
fn test_owner_move() {
    let x = 1;
    let y = Box::new(33);
    println!("{:p}", y);
    let x1 = x;
    let y1 = y;
    println!("new x {}, y {}", x, y1);
    let mut list = Vec::<i32>::new();
    list.push(11);
}
