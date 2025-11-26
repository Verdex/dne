
pub fn underline(input : &str, start : usize, end : usize) -> String {
    let dashes = if start == end {
        "^".to_string()
    }
    else {
        "-".repeat(end - start)
    };
    let mut prev = "";
    let mut c = 0;
    let mut line = 1;
    for x in input.lines(){
        let l = x.len();
        if c <= start && start <= c + l {
            let spaces = " ".repeat(start - c);
            return format!("{prev}{x}\n{spaces}{dashes}");
        }
        prev = x;
        line += 1;
    }

    panic!("underline {start} and {end} are outside of the input string with length: {}", input.len());
}


