//闭包
#[test]
fn test_basic_clourse() {
    let func = |x| (x*x) as u32;
    let val = test_pass_clourse(10, func);
    println!("{}", val);
    let val2 = test_pass_clourse(101 + func(50) as i32, func);
    println!("{} ", val2);
    
}


fn test_pass_clourse<F>(x: i32, func: F) -> u32 where F: Fn(i32) -> u32 {
    func(x)
}