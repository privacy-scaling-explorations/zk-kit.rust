/// Converts a hexadecimal string to a binary string.
///
/// # Arguments
///
/// * `n` - The hexadecimal string to convert.
///
/// # Returns
///
/// The binary representation of the hexadecimal string.
pub fn hex_to_bin(n: &str) -> String {
    let mut chars = n.chars();
    let first_char = chars.next().unwrap();
    let mut bin = format!(
        "{:b}",
        u8::from_str_radix(&first_char.to_string(), 16).unwrap()
    );

    for c in chars {
        bin += &format!("{:04b}", u8::from_str_radix(&c.to_string(), 16).unwrap());
    }

    bin
}

/// Converts a hexadecimal key to a path represented as a vector of usize.
///
/// For each key, it is possible to obtain an array of 256 padded bits.
///
/// # Arguments
///
/// * `key` - The hexadecimal key to convert.
///
/// # Returns
///
/// The path represented as a vector of usize.
pub fn key_to_path(key: &str) -> Vec<usize> {
    let bits = if let Ok(num) = u128::from_str_radix(key, 16) {
        format!("{:b}", num)
    } else {
        hex_to_bin(key)
    };

    let padded_bits = format!("{:0>256}", bits).chars().rev().collect::<String>();
    let bits_array = padded_bits
        .chars()
        .map(|c| c.to_digit(10).unwrap() as usize)
        .collect();

    bits_array
}

/// Returns the index of the last non-zero element in the array.
///
/// # Arguments
///
/// * `array` - The array of hexadecimal strings.
///
/// # Returns
///
/// The index of the last non-zero element in the array, or -1 if no non-zero element is found.
pub fn get_index_of_last_non_zero_element(array: Vec<&str>) -> isize {
    for (i, &item) in array.iter().enumerate().rev() {
        if u128::from_str_radix(item, 16).unwrap_or(0) != 0 {
            return i as isize;
        }
    }

    -1
}

/// Returns the first common elements between two arrays.
///
/// # Arguments
///
/// * `array1` - The first array.
/// * `array2` - The second array.
///
/// # Returns
///
/// The first common elements between the two arrays.
pub fn get_first_common_elements<T: PartialEq + Clone>(array1: &[T], array2: &[T]) -> Vec<T> {
    let min_length = std::cmp::min(array1.len(), array2.len());

    for i in 0..min_length {
        if array1[i] != array2[i] {
            return array1[0..i].to_vec();
        }
    }

    array1[0..min_length].to_vec()
}

/// Checks if a string is a valid hexadecimal string.
///
/// # Arguments
///
/// * `s` - The string to check.
///
/// # Returns
///
/// `true` if the string is a valid hexadecimal string, `false` otherwise.
pub fn is_hexadecimal(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_bin() {
        assert_eq!(hex_to_bin("A"), "1010");
        assert_eq!(hex_to_bin("F"), "1111");
        assert_eq!(hex_to_bin("1A"), "11010");
        assert_eq!(hex_to_bin("FF"), "11111111");
        assert_eq!(hex_to_bin("12"), "10010");
    }

    #[test]
    fn test_key_to_path() {
        let path = key_to_path("17");
        assert_eq!(path.len(), 256);
        assert_eq!(&path[0..5], vec![1, 1, 1, 0, 1]);
    }

    #[test]
    fn test_get_index_of_last_non_zero_element() {
        assert_eq!(get_index_of_last_non_zero_element(vec![]), -1);
        assert_eq!(get_index_of_last_non_zero_element(vec!["0", "0", "0"]), -1);

        assert_eq!(get_index_of_last_non_zero_element(vec!["0", "0", "1"]), 2);
        assert_eq!(get_index_of_last_non_zero_element(vec!["0", "1", "0"]), 1);
        assert_eq!(get_index_of_last_non_zero_element(vec!["1", "0", "0"]), 0);

        assert_eq!(
            get_index_of_last_non_zero_element(vec!["0", "1", "0", "1", "0"]),
            3
        );
        assert_eq!(
            get_index_of_last_non_zero_element(vec!["1", "0", "1", "0", "0"]),
            2
        );
        assert_eq!(
            get_index_of_last_non_zero_element(vec!["0", "0", "0", "1", "1"]),
            4
        );
        assert_eq!(
            get_index_of_last_non_zero_element(vec![
                "0", "17", "3", "0", "3", "0", "3", "2", "0", "0"
            ]),
            7
        )
    }

    #[test]
    fn test_get_first_common_elements() {
        assert_eq!(get_first_common_elements::<u32>(&[], &[]), vec![]);

        assert_eq!(
            get_first_common_elements(&[1, 2, 3], &[1, 2, 3, 4, 5]),
            vec![1, 2, 3]
        );
        assert_eq!(
            get_first_common_elements(&[1, 2, 3, 4, 5], &[1, 2, 3]),
            vec![1, 2, 3]
        );

        assert_eq!(
            get_first_common_elements(&[1, 2, 3], &[1, 2, 4]),
            vec![1, 2]
        );
        assert_eq!(get_first_common_elements(&[1, 2, 3], &[4, 5, 6]), vec![]);
    }

    #[test]
    fn test_is_hexadecimal() {
        assert!(is_hexadecimal("be12"));
        assert!(is_hexadecimal("ABCDEF"));
        assert!(is_hexadecimal("1234567890abcdef"));

        assert!(!is_hexadecimal("gbe12"));
        assert!(!is_hexadecimal("123XYZ"));
        assert!(!is_hexadecimal("abcdefg"));
    }
}
