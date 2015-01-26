extern crate getopts;

use std::io::fs::File;
use std::io;
use std::os;
use getopts::{optopt,optflag, getopts,OptGroup,usage};
use std::io::Command;

mod tokenizer;

fn print_usage(program: &str, opts: &[OptGroup]) {
    let brief = format!("Usage: {} [options] INPUT_FILE", program);
    print!("{}", usage(brief.as_slice(), opts));
}

fn main() {
    let args = os::args();
    let program = args[0].clone();
    let opts = &[
        optflag("h", "help", "print this help menu"),
        optopt("o", "output", "set output filename", "OUTPUT"),
        optflag("r", "rust", "print out rust source"),
    ];

    macro_rules! u {
        () => {
            {
                print_usage(program.as_slice(), opts);
                return;
            }
        }
    };

    let matches = match getopts(args.tail(), opts) {
        Ok(m) => m,
        Err(f) => {
            u!();
        }
    };

    if matches.opt_present("h") { u!(); }
    if matches.free.is_empty() { u!(); }

    let path = Path::new(&matches.free[0]);

    let outname = match matches.opt_str("o") {
        Some(o) => o,
        None => path.filestem_str().unwrap_or("a.out").to_string()
    };
    let file = File::open(&path).read_to_string().unwrap_or("".to_string());

    let indent_by = "    ";
    let mut indent = indent_by.to_string();

    let mut commands = String::new();
    let buffer_size = 50_000;

    for tok in tokenizer::BFToken::parse_file(&path).iter() {
        use tokenizer::BFToken::*;

        let mut level = indent.clone();
        let line = match *tok {
            RShift(count) =>
                format!("ptr += {c}; if ptr >= {bs} {{ ptr = 0; }}", c=count, bs=buffer_size),
            LShift(count) =>
                format!("if ptr == 0 {{ ptr = {bs} - 1; }} else {{ ptr -= {c} }}", c=count, bs=buffer_size),
            Increment(count) =>
                format!("array[ptr] += {count};", count=count),
            Decrement(count) =>
                format!("array[ptr] -= {count};", count=count),
            Comment(ref val) => {
                let val = val.clone();
                let v = val.replace("\n", " ")
                           .replace("\r", " ");
                let v = v.trim_matches(|&: c: char| c == ' ');
                if v.len() > 0 {
                    format!("// {}", v)
                } else {
                    "".to_string()
                }
            },
            WriteChar =>
                "print!(\"{}\", array[ptr] as char);".to_string(),
            ReadChar =>
                "array[ptr] = io::stdin().read_char().unwrap_or('\\0') as u8;".to_string(),
            StartLoop => {
                indent.push_str(indent_by);
                "while array[ptr] > 0 {".to_string()
            },
            EndLoop => {
                for _ in indent_by.chars() {
                    indent.pop();
                }
                level = indent.clone();
                "}".to_string()
            }
        };

        if line.len() > 0 {
            commands = format!("{}{}{}\n", commands, level, line);
        }
    }

    let compile = !matches.opt_present("r");

    let mut child = if compile {
        Some(match Command::new("rustc")
                      .arg("-o").arg(outname)
                      .arg("-").spawn() {
            Ok(child) => child,
            Err(e) => panic!("Failed to execute child: {:?}", e)
        })
    } else {
        None
    };

    macro_rules! src {
        ($($arg:tt)*) => {
            if compile {
                match child.as_mut() {
                    Some(child) => {
                        write!(child.stdin.as_mut().unwrap(), $($arg)*);
                    },
                    None => unreachable!()
                }
            } else {
                write!(&mut io::stdout(), $($arg)*);
            }
        }
    }

    src!("use std::io;\n");
    src!("fn main() {{\n");
    src!("{}let mut array = [0u8; {}];\n", indent_by, buffer_size);
    src!("{}let mut ptr = {};\n", indent_by, buffer_size / 2);
    src!("{}", commands);
    src!("}}\n");

    if compile {
        match child.unwrap().wait_with_output() {
            Ok(out) => {
                println!("{}", String::from_utf8_lossy(out.output.as_slice()));
            },
            Err(_) => {}
        }
    }
}
