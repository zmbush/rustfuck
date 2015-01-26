use std::io::fs::File;

#[derive(Debug)]
pub enum BFToken {
    RShift(u32),
    LShift(u32),
    Increment(u32),
    Decrement(u32),
    WriteChar,
    ReadChar,
    StartLoop,
    EndLoop,
    Comment(String)
}

impl BFToken {
    fn new(ch: char) -> BFToken {
        use self::BFToken::*;

        match ch {
            '>' => RShift(1),
            '<' => LShift(1),
            '+' => Increment(1),
            '-' => Decrement(1),
            '.' => WriteChar,
            ',' => ReadChar,
            '[' => StartLoop,
            ']' => EndLoop,
            n => Comment(n.to_string())
        }
    }

    pub fn parse_file(path: &Path) -> Vec<BFToken> {
        use self::BFToken::*;

        let mut toks = Vec::new();

        let content = File::open(path).read_to_string().unwrap_or("".to_string());

        for ch in content.chars() {
            let t = BFToken::new(ch);
            let l = toks.pop();

            let tok = match (l, t) {
                (Some(RShift(a)), RShift(b)) => RShift(a + b),
                (Some(LShift(a)), LShift(b)) => LShift(a + b),
                (Some(Increment(a)), Increment(b)) => Increment(a + b),
                (Some(Decrement(a)), Decrement(b)) => Decrement(a + b),
                (Some(Comment(a)), Comment(b)) => Comment(format!("{}{}", a, b)),
                (Some(a), b) => {
                    toks.push(a);
                    b
                },
                (_, b) => b
            };

            toks.push(tok);
        }

        toks
    }
}
