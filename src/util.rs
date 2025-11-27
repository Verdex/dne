
pub fn underline(input : &str, start : usize, end : usize) -> String {
    let dashes = if start == end {
        "^".to_string()
    }
    else {
        "-".repeat(1 + end - start)
    };
    let mut prev = "";
    let mut c = 0;
    let mut line = 1;
    for x in input.lines(){
        let l = x.len();
        if c <= start && start <= c + l + 1 {
            let spaces = " ".repeat(start - c);
            return format!("error at {line}\n{prev}\n{x}\n{spaces}{dashes}");
        }
        prev = x;
        c += l + 1;
        line += 1; 
    }

    panic!("underline {start} and {end} are outside of the input string with length: {}", input.len());
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_underline_first_line() {
        let input = "one two three four\nfive six seven";
        let output = underline(input, 8, 12);
        assert_eq!(output, "error at 1\n\none two three four\n        -----");

    }

    #[test]
    fn should_underline_last_line() {
        let input = "one two three four\nfive six seven\neight nine ten";
        let output = underline(input, 40, 43);
        assert_eq!(output, "error at 3\nfive six seven\neight nine ten\n      ----");
    }

    #[test]
    fn should_underline_middle_line() {
        let input = "one two three four\nfive six seven\neight nine ten";
        let output = underline(input, 24, 26);
        assert_eq!(output, "error at 2\none two three four\nfive six seven\n     ---");
    }
}
