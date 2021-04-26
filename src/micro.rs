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

struct Person {
    first: String,
    middle: Option<String>,
    last: String,
}

fn build_full_name(person: &Person) -> String {
    let mut full_name = String::new();
    full_name.push_str(&person.first);
    full_name.push_str(" ");
    if let Some(val) = &person.middle {
        full_name.push_str(val);
        full_name.push_str(" ");
    }

    // TODO: Implement the part of this function that handles the person's middle name.
    full_name.push_str(&person.last);
    full_name
}

#[test]
fn test_option() {
    let john = Person {
        first: String::from("James"),
        middle: Some(String::from("Oliver")),
        last: String::from("Smith"),
    };
    assert_eq!(build_full_name(&john), "James Oliver Smith");

    let alice = Person {
        first: String::from("Alice"),
        middle: None,
        last: String::from("Stevens"),
    };
    assert_eq!(build_full_name(&alice), "Alice Stevens");

    let bob = Person {
        first: String::from("Robert"),
        middle: Some(String::from("Murdock")),
        last: String::from("Jones"),
    };
    assert_eq!(build_full_name(&bob), "Robert Murdock Jones");
}

use std::fs::File;
use std::io::{Error as IoError, Read};
use std::path::PathBuf;

#[test]
fn test_result_type() {
    assert!(read_file_contents("d:/test.txt".into()).is_ok());
    assert!(read_file_contents("d:/test_not_found.txt".into()).is_err());
}

fn read_file_contents(path: PathBuf) -> Result<String, IoError> {
    let mut string = String::new();

    // TODO #1: Handle this match expression.
    // --------------------------------------
    // Pass the variable to the `file` variable on success, or
    // Return from the function early if it is an error.
    let mut file = match File::open(path) {
        Ok(file_handle) => file_handle,
        Err(io_error) => return Err(io_error),
    };

    // TODO #2: Handle this error.
    // ---------------------------
    // The success path is already filled in for you.
    // Return from the function early if it is an error.

    match file.read_to_string(&mut string) {
        Ok(_) => (),
        Err(io_error) => return Err(io_error),
    };

    // TODO #3: Return the `string` variable as expected by this function signature.
    Ok(string)
}

#[test]
fn test_lifetime() {
    let name1 = "Joe";
    let name2 = "Chris";
    let name3 = "Anne";

    let mut names = Vec::new();

    assert_eq!("Joe", copy_and_return(&mut names, &name1));
    assert_eq!("Chris", copy_and_return(&mut names, &name2));
    assert_eq!("Anne", copy_and_return(&mut names, &name3));

    assert_eq!(
        names,
        vec!["Joe".to_string(), "Chris".to_string(), "Anne".to_string()]
    )
}

fn copy_and_return<'a>(vector: &'a mut Vec<String>, value: &'a str) -> &'a str {
    vector.push(String::from(value));
    vector.get(vector.len() - 1).unwrap()
}

struct Container<T> {
    value: T,
}

impl<T> Container<T> {
    pub fn new(value: T) -> Self {
        Container { value }
    }

    fn println(&self) -> &T {
        &self.value
    }
}

#[test]
fn test_generic() {
    let container = Container::new(r#"1111"#);
    let val = container.println();
    println!("{}", val);
    assert_eq!(Container::new(42).value, 42);
    assert_eq!(Container::new(3.14).value, 3.14);
    assert_eq!(Container::new("Foo").value, "Foo");
    assert_eq!(
        Container::new(String::from("Bar")).value,
        String::from("Bar")
    );
    assert_eq!(Container::new(true).value, true);
    assert_eq!(Container::new(-12).value, -12);
    assert_eq!(Container::new(Some("text")).value, Some("text"));
}

struct Groups<T> {
    inner: Vec<T>,
    idx: usize,
}

impl<T> Groups<T> {
    fn new(inner: Vec<T>) -> Self {
        Groups {
            inner: inner,
            idx: 0,
        }
    }
}

impl<T: PartialEq + std::fmt::Debug> Iterator for Groups<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.is_empty() {
            return None;
        }
        let first = &self.inner[0];
        let mut next = 1;
        for i in &self.inner[1..] {
            if i == first {
                next += 1;
            } else {
                break;
            }
        }

        let ret = self.inner.drain(0..next).into_iter().collect::<Vec<_>>();

        if ret.len() > 0 {
            Some(ret)
        } else {
            None
        }
    }

    // TODO: Write the rest of this implementation.
}

#[test]
fn test_iter_generic() {
    let char = vec!['a', 'b', 'b', 'c'];
    assert_eq!(
        Groups::new(char).into_iter().collect::<Vec<Vec<_>>>(),
        vec![vec!['a'], vec!['b', 'b'], vec!['c']]
    );

    let string = vec!["123", "123", "456"];
    assert_eq!(
        Groups::new(string).into_iter().collect::<Vec<Vec<_>>>(),
        vec![vec!["123", "123"], vec!["456"]]
    );

    let data = vec![4, 1, 1, 2, 1, 3, 3, -2, -2, -2, 5, 5];
    // groups:     |->|---->|->|->|--->|----------->|--->|

    assert_eq!(
        Groups::new(data).into_iter().collect::<Vec<Vec<_>>>(),
        vec![
            vec![4],
            vec![1, 1],
            vec![2],
            vec![1],
            vec![3, 3],
            vec![-2, -2, -2],
            vec![5, 5],
        ]
    );

    let data2 = vec![1, 2, 2, 1, 1, 2, 2, 3, 4, 4, 3];
    // groups:      |->|---->|---->|----|->|----->|->|
    assert_eq!(
        Groups::new(data2).into_iter().collect::<Vec<Vec<_>>>(),
        vec![
            vec![1],
            vec![2, 2],
            vec![1, 1],
            vec![2, 2],
            vec![3],
            vec![4, 4],
            vec![3],
        ]
    )
}

pub fn is_even(num: i32) -> bool {
    num % 2 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_true_when_even() {
        assert!(is_even(2), true);
    }

    #[test]
    fn is_false_when_odd() {
        assert!(!is_even(3), true);
    }
}
