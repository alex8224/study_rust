use std::collections::HashMap;

#[test]
fn indexing_tuple() {
    let numbers = (1, 2, 3);
    let second = numbers.1;

    assert_eq!(
        2, second,
        "This is not the 2nd number in the tuple: {}",
        second
    )
}

#[test]
fn indexing_array() {
    let characters = ['a', 'b', 'c', 'd', 'e'];
    let letter_d = characters[3];

    assert_eq!(
        'd', letter_d,
        "This is not the character for the letter d: {}",
        letter_d
    )
}
#[test]
fn test_fruit_basket() {
    let ret = fruit_basket();
    ret.into_iter().for_each(|f| {
        println!("{}={}\n", f.0, f.1);
    })
}

fn fruit_basket() -> HashMap<String, u32> {
    let mut basket = HashMap::<String, u32>::new();
    // Two bananas are already given for you :)
    basket.insert(String::from("banana"), 2);
    basket.insert(String::from("apple"), 3);
    basket.insert(String::from("pear"), 4);
    basket
}
