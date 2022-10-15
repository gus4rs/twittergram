use std::io::BufRead;
use std::str::FromStr;

/// Reads input and converts to the adequate type
///
/// # Arguments
///  * `name` - The name of the data to read, e.g., "API ID". Will prompt the user the type it in
///  * `input` - The source of data, e.g. `stdin()`
///
/// # Examples
/// ```
/// use std::io::stdin;
///     use twittergram::read_input;
///     let age: i32 = read_input("Age".to_string(),&mut stdin());
/// ```
/// Asks the user for the age and store it in a ```i32``` variable
///
pub fn read_input<T: FromStr>(name: String, input: &mut impl BufRead) -> T {
    let mut user_input = String::new();
    loop {
        println!("Please type {}", name);
        match input.read_line(&mut user_input) {
            Ok(_) => break,
            Err(_) => continue,
        }
    }
    return T::from_str(user_input.trim()).ok().unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_input() {
        let integer: i32 = read_input("m".to_string(), &mut "123456".as_bytes());
        assert_eq!(integer, 123456 as i32);

        let float: f32 = read_input("m".to_string(), &mut "123.456".as_bytes());
        assert_eq!(float, 123.456 as f32);

        let string: String = read_input("m".to_string(), &mut "123.456".as_bytes());
        assert_eq!(string, "123.456");
    }
}
