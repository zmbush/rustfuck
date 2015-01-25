use std::io::fs::File;

fn main() {
    let file = File::open(&Path::new("test.bf")).read_to_string().unwrap_or("".to_string());
    let indent_by = "    ";
    let mut indent = indent_by.to_string();

    let mut counts: Vec<(char, u32)> = Vec::new();

    for ch in file.chars() {
        match counts.pop() {
            Some(c) => {
                let (c, mut count) = c;
                if c == ch {
                    count += 1;

                    counts.push((c, count));
                    continue;
                }
                counts.push((c, count));
            },
            None => {}
        }

        match ch {
            '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']' => {
                counts.push((ch, 1));
            }
            _ => {}
        }
    }

    let mut commands = String::new();
    let buffer_size = 50_000;

    for &(ch, count) in counts.iter() {
        macro_rules! mkline {
            ($str:expr) => {
                format!(concat!($str, "/*{count}{buffer_size}*/"), count=count, buffer_size=buffer_size)
            }
        }

        let line = match ch {
            '>' => mkline!("ptr += {count}; if ptr >= {buffer_size} {{ ptr = 0; }}"),
            '<' => mkline!("if ptr == 0 {{ ptr = {buffer_size} - 1; }} else {{ ptr -= {count} }}"),
            '+' => mkline!("array[ptr] += {count};"),
            '-' => mkline!("array[ptr] -= {count};"),
            _ => {
                for i in 0..count {
                    let mut level = indent.clone();
                    let cmd = match ch {
                        '.' => "print!(\"{}\", array[ptr] as char);",
                        ',' => "array[ptr] = io::stdin().read_char().unwrap_or('\0') as u8;",
                        '[' => {
                            indent.push_str(indent_by);
                            "while array[ptr] > 0 {"
                        },
                        ']' => {
                            for _ in indent_by.chars() {
                                indent.pop();
                            }
                            level = indent.clone();
                            "}"
                        },
                        _ => continue
                    };

                    commands = format!("{}{}{}\n", commands, level, cmd);
                }

                continue;
            }
        };

        commands = format!("{}{}{};\n", commands, indent, line);
    }

    macro_rules! src {
        ($($arg:tt)*) => { println!($($arg)*) }
    }

    src!("use std::io;");
    src!("fn main() {{");
    src!("{}let mut array = [0u8; {}];", indent_by, buffer_size);
    src!("{}let mut ptr = {};", indent_by, buffer_size / 2);
    src!("{}", commands);
    src!("}}");
}
