use std::fmt::Formatter;

#[derive(Debug)]
struct Address {
    phone: String,
    zip_code: String,
    addr: String,
}

#[derive(Debug)]
struct UserInfo<'a, 'b> {
    addr: &'a Address,
    name: &'b str,
    age: u16,
}

impl<'a, 'b: 'a> UserInfo<'a, 'b> {
    fn new(_addr: &'a Address, _name: &'b str, _age: u16) -> Self {
        Self {
            addr: _addr,
            name: _name,
            age: _age,
        }
    }

    fn name(&self) -> &'a str {
       self.name
    }
}

fn get_name<'a>(user: &'a UserInfo) -> &'a str {
    user.name()
}

#[test]
pub fn test_1_1_1() {
    let addr = Address {
        phone: "13800138000".to_string(),
        zip_code: "518000".to_string(),
        addr: "湖南长沙".to_string(),
    };

    let info = UserInfo::new(&addr, "直接截屏", 18);

    println!("==={:?} \n {:?}, name is {}", addr, info, get_name(&info));
}
