#![feature(io,path,core,os,plugin)]

#![plugin(regex_macros)]
#[no_link] extern crate regex_macros;
extern crate regex;

extern crate getopts;

use regex::Regex;
use std::os;
use std::old_io::{File, fs};
use std::old_io::fs::PathExtensions;
use std::collections::HashSet;
use getopts::Options;

struct Matcher {
    exp: Regex,
}

impl Matcher {
    fn new() -> Matcher {
        Matcher { exp: regex!(r"^shutterstock_(\d+)") }
    }

    fn image_number(&self, test: &str) -> String {
        self.exp.captures(test).unwrap().at(1).unwrap_or("").to_string()
    }
}

fn main() {
    let args: Vec<String> = os::args();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("i", "input-dir", "set folder to scan (required)", "DIRECTORY");
    opts.optopt("o", "output-file", "the file to output the log to", "OUTPUT");
    opts.optflag("d", "delete", "automatically delete the duplicate files");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(args.tail()) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let directory = match matches.opt_str("i") {
        Some(d) => d,
        None => {
            print_usage(&program, opts);
            return;
        },
    };

    let folder = fs::walk_dir(&Path::new(directory));

    let mut image_nums = Box::new(HashSet::new());
    let mut duplicates = Box::new(HashSet::new());

    let matcher = Matcher::new();

    match folder {
        Ok(results) => {
            for file_path in results {
                if file_path.is_file() {
                    let num = matcher.image_number(file_path.filename_str().unwrap());

                    if image_nums.contains(&num) && num != "" {
                        println!("Duplicate! {}", file_path.display());
                        duplicates.insert(file_path);
                    } else {
                        image_nums.insert(num);
                    }
                }
            }
        },
        Err(e) => println!("{}", e),
    }

    println!("{} duplicates found, {} files scanned", duplicates.len(), duplicates.len() + image_nums.len());

    let mut out_file = match matches.opt_str("o") {
        Some(f) => Option::Some(File::create(&Path::new(f)).unwrap()),
        None => Option::None,
    };
    let delete =  matches.opt_present("d");

    for dup in duplicates.iter() {
        if delete {
            fs::unlink(&dup);
        }
        match out_file {
            Some(ref mut f) => {
                f.write_str(format!("{}\n", dup.display()).as_slice());
            },
            _ => { },
        }
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(brief.as_slice()));
}
