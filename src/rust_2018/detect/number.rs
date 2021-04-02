//! Detects a number literal, like `12.34` or `0b100100`.

/// Detects a number literal, like `12.34` or `0b100100`.
/// 
/// ### Arguments
/// * `orig` The original Rust code, assumed to conform to the 2018 edition
/// * `pos` The character position in `orig` to look at
/// 
/// ### Returns
/// If `pos` begins a valid looking number literal, `detect_number()`
/// returns the character position after the end of the number.  
/// Otherwise, `detect_number()` just returns the `pos` argument.
pub fn detect_number(orig: &str, pos: usize) -> usize {
    // If the current char is past the last char in `orig`, bail out!
    let len = orig.len();
    if pos >= len { return pos }
    let c = get_aot(orig, pos);
    // If the current char is not a digit, then it does not begin a char.
    if c < "0" || c > "9" { return pos }
    // If the digit is the input code’s last character, we’re finished.
    if len == pos + 1 { return len }
    // If the digit at `pos` is not zero, this is a decimal number:
    if c != "0" { return detect_number_decimal(orig, pos, len) }
    // If the digit is zero, and the next char is "b", "x" or "o":
    match get_aot(orig, pos + 1) {
        // Use the binary, hex or octal detector function, as appropriate.
        "b" => detect_number_binary(orig, pos, len),
        "x" => detect_number_hex(orig, pos, len),
        "o" => detect_number_octal(orig, pos, len),
        // Otherwise, this is a decimal number which starts with a zero.
        _ => detect_number_decimal(orig, pos, len),
    }
}

// Returns the ascii character at a position, or tilde if invalid or non-ascii.
fn get_aot(orig: &str, p: usize) -> &str { orig.get(p..p+1).unwrap_or("~") }

fn detect_number_binary(orig: &str, pos: usize, len: usize) -> usize {
    let mut has_digit = false; // binary literals must have at least one digit
    for i in pos+2..len { // +2, because we already found "0b"
        let c = get_aot(orig, i);
        // If the character is an underscore, do nothing.
        if c == "_" {
        // Otherwise, if this char is a binary digit:
        } else if c == "0" || c == "1" {
            has_digit = true;
        // Otherwise, if this is a digit (can only be 2 to 9, here) or a dot:
        } else if (c >= "0" && c <= "9") || c == "." {
            // Reject the whole of 0b101021, don’t just accept the 0b1010 part.
            // And reject the whole of 0b11.1, don’t just accept the 0b11 part.
            return pos
        } else {
            // Advance to the character after the binary number.
            return if has_digit { i } else { pos }
        }
    }
    // We’ve reached the end of the input string.
    if has_digit { len } else { pos }
}

fn detect_number_decimal(orig: &str, pos: usize, len: usize) -> usize {
    let mut has_dot = false; // decimal literals may have one "."
    let mut has_e = false; // decimal literals may have one "e" or "E"
    let mut pos_dot = 0; // helps detect invalid numbers like "1._2"
    let mut pos_e = 0; // helps detect invalid numbers like "10E2+3" and "10E"
    let mut pos_eu = 0; // helps detect invalid numbers like "10E_"
    let mut pos_s = 0; // helps detect numbers with invalid signs, like "10E+"

    for i in pos+1..len { // +1, because we already found a digit, 0 to 9
        let c = get_aot(orig, i);

        // If the character is an underscore:
        if c == "_" {
            // Reject a number like "1._2", where the "." is followed by "_".
            if has_dot && pos_dot == i { return pos }
            // Guard against a dangling underscore, eg "7.5e_".
            if has_e && pos_e == i { pos_eu = i + 1 }

        // If the previous char was "e" or "E" and this is a "+" or "-":
        } else if has_e && pos_e == i && (c == "+" || c == "-") {
            // Guard against a dangling plus or minus sign, eg "7.5e-".
            pos_s = i + 1

        // If we haven’t found a decimal point yet, and this char is a dot:
        } else if ! has_dot && c == "." {
            // Reject a number like "1e2.3", where the exponent contains a dot.
            if has_e { return pos }
            // Else, record that a dot was found, and the position after it.
            // We are being verbose by setting two variables here, but hopefully
            // it makes the code clearer, and perhaps run a little faster.
            has_dot = true;
            pos_dot = i + 1;

        // If we haven’t found an exponent marker yet, and this is "e" or "E":
        } else if ! has_e && (c == "e" || c == "E") {
            // Record that an "e" or "E" was found, and the position after it.
            has_e = true;
            pos_e = i + 1;

        // Otherwise, if this char is not a digit:
        } else if c < "0" || c > "9" {
            // We’ve reached a char which can’t be part of a valid number.
            // Numbers can’t end "e", "E", "+", "-", "e_" or "E_".
            return if i == pos_e || i == pos_s || i == pos_eu { pos } else { i }
        }
    }

    // We’ve reached the end of the input string.
    // Numbers can’t end "e", "E", "+", "-", "e_" or "E_".
    if len == pos_e || len == pos_s || len == pos_eu { pos } else { len }
}

fn detect_number_hex(orig: &str, pos: usize, len: usize) -> usize {
    let mut has_digit = false; // hex literals must have at least one digit
    for i in pos+2..len { // +2, because we already found "0x"
        let c = get_aot(orig, i);
        // If the character is an underscore, do nothing.
        if c == "_" {
        // Otherwise, if this char is a hex digit 0-9A-Fa-f:
        } else if c.chars().all(|c| c.is_ascii_hexdigit()) {
            has_digit = true;
        // Otherwise, if this char is a point:
        } else if c == "." {
            // Reject the whole of 0xAB.C, don’t just accept the 0xAB part.
            return pos
        } else {
            // Advance to the character after the hex number.
            return if has_digit { i } else { pos }
        }
    }
    // We’ve reached the end of the input string.
    if has_digit { len } else { pos }
}

fn detect_number_octal(orig: &str, pos: usize, len: usize) -> usize {
    let mut has_digit = false; // octal literals must have at least one digit
    for i in pos+2..len { // +2, because we already found "0o"
        let c = get_aot(orig, i);
        // If the character is an underscore, do nothing.
        if c == "_" {
        // Otherwise, if this char is an digit 0-7:
        } else if c >= "0" && c <= "7" {
            has_digit = true;
        // Otherwise, if this char is a point:
        } else if c == "." {
            // Reject the whole of 0o56.7, don’t just accept the 0o56 part.
            return pos
        } else {
            // Advance to the character after the octal number.
            return if has_digit { i } else { pos }
        }
    }
    // We’ve reached the end of the input string.
    if has_digit { len } else { pos }
}


#[cfg(test)]
mod tests {
    use super::detect_number as detect;

    #[test]
    fn detect_number_correct() {
        // Binary.
        let orig = "0b01 0b0_0_ 0b1A 0b__1_";
        assert_eq!(detect(orig, 0), 4); // 0b01
        assert_eq!(detect(orig, 1), 1); // b01
        assert_eq!(detect(orig, 2), 4); // 01 is recognised as decimal
        assert_eq!(detect(orig, 5), 11); // 0b0_0_
        assert_eq!(detect(orig, 12), 15); // the 0b1 part is accepted
        assert_eq!(detect(orig, 17), 23); // 0b__1_
        // Decimal integer.
        let orig = "7 0 3";
        assert_eq!(detect(orig, 0), 1); // 7
        assert_eq!(detect(orig, 1), 1); // space
        assert_eq!(detect(orig, 2), 3); // 0
        assert_eq!(detect(orig, 3), 3); // space
        assert_eq!(detect(orig, 4), 5); // 3
        let orig = "765 012 10";
        assert_eq!(detect(orig, 0), 3); // 765
        assert_eq!(detect(orig, 1), 3); // 65 no ‘lookbehind’ happens!
        assert_eq!(detect(orig, 2), 3); // 5
        assert_eq!(detect(orig, 3), 3); // space
        assert_eq!(detect(orig, 4), 7); // 012
        assert_eq!(detect(orig, 7), 7); // space
        assert_eq!(detect(orig, 8), 10); // 10
        assert_eq!(detect(orig, 9), 10); // 0
        // Decimal with underscores.
        let orig = "7_5 012___ 3_4_. 0_0.0_00__0_";
        assert_eq!(detect(orig, 0), 3); // 7_5
        assert_eq!(detect(orig, 1), 1); // _5 can’t start numbers that way!
        assert_eq!(detect(orig, 2), 3); // 5
        assert_eq!(detect(orig, 4), 10); // 012___
        assert_eq!(detect(orig, 11), 16); // 3_4_.
        assert_eq!(detect(orig, 17), 29); // 0_0.0_00__0_
        // Float no exponent.
        let orig = "7.5 0.12 34. 00.0__0_00";
        assert_eq!(detect(orig, 0), 3); // 7.5
        assert_eq!(detect(orig, 1), 1); // .5 is not a valid number
        assert_eq!(detect(orig, 2), 3); // 5
        assert_eq!(detect(orig, 3), 3); // space
        assert_eq!(detect(orig, 4), 8); // 0.12
        assert_eq!(detect(orig, 9), 12); // 34. is valid
        assert_eq!(detect(orig, 13), 23); // 00.0__0_00
        // Here, each "123." exercises a different conditional branch.
        let orig = "123. 123.";
        assert_eq!(detect(orig, 0), 4); // 123. part way through input
        assert_eq!(detect(orig, 5), 9); // 123. reaches end of input
        // Float with exponent.
        let orig = "0e0 9E9 1e+2 4E-3 8E1+2 54.32E+10";
        assert_eq!(detect(orig, 0), 3);   // 0e0 is 0
        assert_eq!(detect(orig, 4), 7);   // 9E9 is 9000000000
        assert_eq!(detect(orig, 8), 12);  // 1e+2 is 100
        assert_eq!(detect(orig, 13), 17); // 4E-3 is 0.004
        assert_eq!(detect(orig, 18), 21); // the 8E1 part is accepted
        assert_eq!(detect(orig, 24), 33); // 54.32E+10 is 543200000000
        let orig = "4_3.21e+10 43_.21e+10 43.2_1e+10 43.21_e+10 43.21e+_10 43.21e+1_0 43.21e+10_";
        assert_eq!(detect(orig, 0), 10);  // 4_3.21e+10 is ok .js
        assert_eq!(detect(orig, 11), 21); // 43_.21e+10 is invalid in .js
        assert_eq!(detect(orig, 22), 32); // 43.2_1e+10 is ok .js
        assert_eq!(detect(orig, 33), 43); // 43.21_e+10 is invalid in .js
        assert_eq!(detect(orig, 44), 54); // 43.21e+_10 is invalid in .js
        assert_eq!(detect(orig, 55), 65); // 43.21e+1_0 is ok .js
        assert_eq!(detect(orig, 66), 76); // 43.21e+10_ is invalid in .js
        assert_eq!(detect("43.21e_10", 0), 9); // 43.21e_10 is invalid in .js
        // Hex.
        let orig = "0x09 0xA_b_ 0xAG 0x__C_";
        assert_eq!(detect(orig, 0), 4);   // 0x09
        assert_eq!(detect(orig, 1), 1);   // x09
        assert_eq!(detect(orig, 2), 4);   // 09 is recognised as decimal
        assert_eq!(detect(orig, 5), 11);  // 0xA_b_ mixed case is ok
        assert_eq!(detect(orig, 12), 15); // the 0xA part is accepted
        assert_eq!(detect(orig, 17), 23); // 0x__C_
        // Octal.
        let orig = "0o07 0o7_3_ 0o7a 0o__5_";
        assert_eq!(detect(orig, 0), 4);   // 0o07
        assert_eq!(detect(orig, 1), 1);   // o07
        assert_eq!(detect(orig, 2), 4);   // 07 is recognised as decimal
        assert_eq!(detect(orig, 5), 11);  // 0o7_3_
        assert_eq!(detect(orig, 12), 15); // the 0o7 part is accepted
        assert_eq!(detect(orig, 17), 23); // 0o__5_
    }

    #[test]
    fn detect_number_incorrect() {
        // Incorrect binary.
        let orig = "0b12 0b11.1 0b 0B11 0b___";
        assert_eq!(detect(orig, 0), 0); // 0b12 is not a valid number
        assert_eq!(detect(orig, 2), 4); // 12 is recognised as decimal
        assert_eq!(detect(orig, 5), 5); // 0b11.1 is not a valid number
        assert_eq!(detect(orig, 7), 11); // 11.1
        assert_eq!(detect(orig, 12), 12); // 0b is not a valid number
        assert_eq!(detect(orig, 15), 16); // 0B11 is not valid, but 0 is
        assert_eq!(detect(orig, 20), 20); // 0b___ is not a valid number
        // Decimal integer.
        // @TODO
        // Incorrect float no exponent.
        let orig = "1.2.3 .12 0..1";
        assert_eq!(detect(orig, 0), 3); // 1.2
        assert_eq!(detect(orig, 1), 1); // .2 is not a valid number
        assert_eq!(detect(orig, 2), 5); // 2.3
        assert_eq!(detect(orig, 5), 5); // space
        assert_eq!(detect(orig, 6), 6); // .12 is not a valid number
        assert_eq!(detect(orig, 7), 9); // 12
        assert_eq!(detect(orig, 10), 12); // 0.
        assert_eq!(detect(orig, 11), 11); // ..
        assert_eq!(detect(orig, 12), 12); // .1
        assert_eq!(detect(orig, 13), 14); // 1
        // Incorrect float with exponent.
        let orig = "10e 9E+ 1e2. 4E+-3 8Ee12 1+1 54.32E";
        assert_eq!(detect(orig, 0), 0);   // 10e has no exponent value
        assert_eq!(detect(orig, 4), 4);   // 9E+ has no exponent value
        assert_eq!(detect(orig, 8), 8);   // 1e2. exponent value contains "."
        assert_eq!(detect(orig, 13), 13); // 4E+-3 has "+" and "-"
        assert_eq!(detect(orig, 19), 19); // 8Ee12 has an extra "e"
        assert_eq!(detect(orig, 21), 21); // e12 has no digit at start
        assert_eq!(detect(orig, 25), 26); // 1+1 perhaps you meant 1e+1
        assert_eq!(detect(orig, 29), 29); // 54.32E has no exponent value
        // The last character of a string is an edge case which needs its own test.
        assert_eq!(detect("54.32e-", 0), 0); // 54.32e- has no exponent value
        // Here, each "43.21e_" exercises a different conditional branch.
        let orig = "43._21e+10 43.21e_+10 43.21e_+ 43.21e_ 43.21e_";
        assert_eq!(detect(orig, 0), 0);   // 43._21e+10
        assert_eq!(detect(orig, 11), 11); // 43.21e_+10
        assert_eq!(detect(orig, 22), 22); // 43.21e_+
        assert_eq!(detect(orig, 31), 31); // 43.21e_ part way through input
        assert_eq!(detect(orig, 39), 39); // 43.21e_ reaches end of input
        // Invalid hex.
        let orig = "0xGA 0xab.c 0x 0XAB 0x___";
        assert_eq!(detect(orig, 0), 0);   // 0xGA is not a valid number
        assert_eq!(detect(orig, 5), 5);   // 0xab.c is not a valid number
        assert_eq!(detect(orig, 7), 7);   // ab.c is valid, but not a number
        assert_eq!(detect(orig, 12), 12); // 0x is not a valid number
        assert_eq!(detect(orig, 15), 16); // 0XAB is not valid, but 0 is
        assert_eq!(detect(orig, 20), 20); // 0x___ is not a valid number
        // Incorrect octal.
        let orig = "0oa7 0o56.7 0o 0O34 0o___";
        assert_eq!(detect(orig, 0), 0);   // 0oa7 is not a valid number
        assert_eq!(detect(orig, 5), 5);   // 0o56.7 is not a valid number
        assert_eq!(detect(orig, 7), 11);  // 56.7 is recognised as decimal
        assert_eq!(detect(orig, 12), 12); // 0o is not a valid number
        assert_eq!(detect(orig, 15), 16); // 0O34 is not valid, but 0 is
        assert_eq!(detect(orig, 20), 20); // 0o___ is not a valid number
        // Number too large.
        // These numbers are larger than u128, so Rust won’t parse them.
        // However, detect_number() is just a scanner, and not that smart!
        // let _nope: u128 = 1234567890123456789012345678901234567890;
        let orig = "1234567890123456789012345678901234567890";
        assert_eq!(detect(orig, 0), 40);
        // let _nope: u128 = 0b1_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
        let orig = "0b1_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000";
        assert_eq!(detect(orig, 0), 147);
        // let _nope: u128 = 0o12345671234567123456712345671234567123456712;
        let orig = "0o12345671234567123456712345671234567123456712";
        assert_eq!(detect(orig, 0), 46);
        // let _nope: u128 = 0x1234567890abcdefABCDEF1234567890a;
        let orig = "0x1234567890abcdefABCDEF1234567890a";
        assert_eq!(detect(orig, 0), 35); // we also test 0-9A-Za-z here
    }

    #[test]
    fn detect_number_will_not_panic() {
        println!("{}", 0x1E+9);
        // Near the end of `orig`.
        assert_eq!(detect("", 0), 0); // empty string
        assert_eq!(detect("0", 0), 1); // 0
        assert_eq!(detect("0~", 0), 1); // 0
        // Binary, near the end of `orig`.
        assert_eq!(detect("0b", 0), 0); // rejected, no binary value
        assert_eq!(detect("0B", 0), 1); // 0, "B" is not like "b"
        assert_eq!(detect("0b_", 0), 0); // rejected, no binary value
        assert_eq!(detect("0b2", 0), 0); // rejected, out of range
        assert_eq!(detect("0b12", 0), 0); // rejected, out of range
        assert_eq!(detect("0b_1", 0), 4); // 0b_1
        assert_eq!(detect("0b1_", 0), 4); // 0b1_
        assert_eq!(detect("0b1.", 0), 0); // binary float is not allowed
        assert_eq!(detect("0b1.1", 0), 0); // binary float is not allowed
        assert_eq!(detect("0b1e1", 0), 3); // 0b1
        // Decimal integer, near the end of `orig`.
        assert_eq!(detect("1", 0), 1); // 1
        assert_eq!(detect("+1", 0), 0); // leading "+" can’t start the lexeme
        assert_eq!(detect("-1", 0), 0); // leading "-" can’t start the lexeme
        assert_eq!(detect("1_", 0), 2); // 1_
        assert_eq!(detect("_1", 0), 0); // leading underscore is not allowed
        assert_eq!(detect("1_1", 0), 3); // 1_1
        assert_eq!(detect("1__1", 0), 4); // 1__1
        // Float, near the end of `orig`.
        assert_eq!(detect("1.", 0), 2); // 1.
        assert_eq!(detect("1.1", 0), 3); // 1.1
        assert_eq!(detect("1e", 0), 0); // 1
        assert_eq!(detect("1E", 0), 0); // 1
        assert_eq!(detect("1e1", 0), 3); // 1e1
        assert_eq!(detect("1E1", 0), 3); // 1E1
        assert_eq!(detect("1.e1", 0), 4); // 1 // @TODO fix this!
        assert_eq!(detect("1.E1", 0), 4); // 1 // @TODO fix this!
        assert_eq!(detect("1.1e", 0), 0); // rejected, no exponent value
        assert_eq!(detect("1.1E", 0), 0); // rejected, no exponent value
        assert_eq!(detect("1e+1", 0), 4); // 1e+1
        assert_eq!(detect("1E+1", 0), 4); // 1E+1
        assert_eq!(detect("1e-1", 0), 4); // 1e-1
        assert_eq!(detect("1E-1", 0), 4); // 1E-1
        assert_eq!(detect("1e+", 0), 0); // rejected, trailing sign after +
        assert_eq!(detect("1E+", 0), 0); // rejected, trailing sign after +
        assert_eq!(detect("1e-", 0), 0); // rejected, trailing sign after -
        assert_eq!(detect("1E-", 0), 0); // rejected, trailing sign after -
        // Hex, near the end of `orig`.
        assert_eq!(detect("0x", 0), 0); // rejected, no hex value
        assert_eq!(detect("0X", 0), 1); // 0, "X" is not like "x"
        assert_eq!(detect("0x_", 0), 0); // rejected, no hex value
        assert_eq!(detect("0xG", 0), 0); // rejected, out of range
        assert_eq!(detect("0x1g", 0), 3); // 0x1 @TODO maybe follow "0b12" behaviour?
        assert_eq!(detect("0x_1", 0), 4); // 0x_1
        assert_eq!(detect("0x1_", 0), 4); // 0x1_
        assert_eq!(detect("0x1.", 0), 0); // hex float is not allowed
        assert_eq!(detect("0x1.1", 0), 0); // hex float is not allowed
        assert_eq!(detect("0x1e", 0), 4); // 0x1e not enterpreted as exp
        assert_eq!(detect("0x1E", 0), 4); // 0x1E not enterpreted as exp
        assert_eq!(detect("0x1e1", 0), 5); // 0x1e1 not enterpreted as exp
        assert_eq!(detect("0x1E1", 0), 5); // 0x1E1 not enterpreted as exp
        assert_eq!(detect("0x1e+1", 0), 4); // 0x1e1 not enterpreted as exp
        assert_eq!(detect("0x1E+1", 0), 4); // 0x1E1 not enterpreted as exp
        assert_eq!(detect("0x1e-1", 0), 4); // 0x1e not enterpreted as exp
        assert_eq!(detect("0x1E-1", 0), 4); // 0x1E not enterpreted as exp
        assert_eq!(detect("0x1e+", 0), 4); // 0x1e not enterpreted as exp
        assert_eq!(detect("0x1E+", 0), 4); // 0x1E not enterpreted as exp
        assert_eq!(detect("0x1e-", 0), 4); // 0x1e not enterpreted as exp
        assert_eq!(detect("0x1E-", 0), 4); // 0x1E not enterpreted as exp
        // Octal, near the end of `orig`.
        assert_eq!(detect("0o", 0), 0); // rejected, no hex value
        assert_eq!(detect("0O", 0), 1); // 0, "O" is not like "o"
        assert_eq!(detect("0o_", 0), 0); // rejected, no hex value
        assert_eq!(detect("0o8", 0), 0); // rejected, out of range
        assert_eq!(detect("0o18", 0), 3); // 0o1 @TODO maybe follow "0b12" behaviour?
        assert_eq!(detect("0o_1", 0), 4); // 0o_1
        assert_eq!(detect("0o1_", 0), 4); // 0o1_
        assert_eq!(detect("0o1.", 0), 0); // octal float is not allowed
        assert_eq!(detect("0o1.1", 0), 0); // octal float is not allowed
        assert_eq!(detect("0o1e1", 0), 3); // 0o1
        // Invalid `pos`.
        assert_eq!(detect("123", 2), 3); // 2 is before "3", so in range
        assert_eq!(detect("123", 3), 3); // 3 is after "3", so incorrect
        assert_eq!(detect("123", 4), 4); // 4 is out of range
        assert_eq!(detect("123", 100), 100); // 100 is way out of range
        // Non-ascii.
        assert_eq!(detect("€", 1), 1); // part way through the three eurobytes
        assert_eq!(detect("1€", 0), 1); // non-ascii after 1
        assert_eq!(detect("1.€", 0), 2); // non-ascii after 1.
        assert_eq!(detect("1_€'", 0), 2); // non-ascii after 1_
        assert_eq!(detect("1e€'", 0), 0); // non-ascii after 1e
        assert_eq!(detect("0€", 0), 1); // non-ascii after 0
        assert_eq!(detect("0b€", 0), 0); // non-ascii after 0b
        assert_eq!(detect("0b0€", 0), 3); // non-ascii after 0b0
        assert_eq!(detect("0x€", 0), 0); // non-ascii after 0b
        assert_eq!(detect("0x0€", 0), 3); // non-ascii after 0b0
        assert_eq!(detect("0o€", 0), 0); // non-ascii after 0o
        assert_eq!(detect("0o0€", 0), 3); // non-ascii after 0o0
    }

}
