#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[test]
fn test_rc() {
    let number = Point { x: 1, y: 2 };
    let rc_num = std::rc::Rc::new(number);
    let cloned = rc_num.clone();
    println!("{:?} {:?}", rc_num, cloned);
}

#[test]
fn test_arc() {
    let number = Point { x: 1, y: 2 };
    let arc_num = std::sync::Arc::new(number);
    let mut threads: Vec<std::thread::JoinHandle<_>> = Vec::new();
    for i in 1..1000 {
        let num_cloned = arc_num.clone();
        let thread = std::thread::spawn(move || {
            let cur = std::thread::current();
            println!(
                "get cloned num {} {:?} at thread {:?}",
                i,
                num_cloned.clone(),
                cur
            );
        });
        threads.push(thread);
    }
    threads.into_iter().for_each(|task| task.join().unwrap());
}

#[derive(Debug)]
enum GradeLevel {
    A,
    B,
    C,
    D,
    E,
}

#[derive(Debug)]
struct UserInfo {
    num: u32,
    scope: u32,
    level: Option<GradeLevel>,
}

impl UserInfo {
    fn new(_num: u32, _scope: u32) -> UserInfo {
        UserInfo {
            num: _num,
            scope: _scope,
            level: None,
        }
    }
}

#[test]
fn test_grade_sum() {
    use rand::Rng;
    use GradeLevel::*;
    let mut classmates = Vec::<UserInfo>::new();
    let mut rng = rand::thread_rng();
    for i in 1..101 {
        let scope: u32 = rng.gen_range(1..100);
        classmates.push(UserInfo::new(i, scope));
    }

    let calu_task = std::thread::spawn(move || {
        classmates.into_iter().for_each(|mate| {
            let level = match mate.scope {
                90..=100 => A,
                80..=89 => B,
                70..=79 => C,
                60..=69 => D,
                _ => E,
            };
            println!(
                "num: {}, scope: {}, level {:?}",
                mate.num, mate.scope, level
            );
        });
    });

    calu_task.join().unwrap();
}
