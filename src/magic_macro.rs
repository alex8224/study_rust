mod magic {

    macro_rules! create_func {
        ($func_name:ident) => {
            fn $func_name() {
                println!("function {:?} is called", stringify!($func_name))
            }
        };
    }

    #[test]
    fn basic_macro() {
        create_func!(foo);
        foo();
    }
}
