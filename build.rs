use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::process::Command;
use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=generate_wordlists.py");
    println!("cargo:rerun-if-changed=requirements.txt");

    let out_str = &env::var("OUT_DIR").expect("OUT_DIR not set");
    let out = Path::new(&out_str);
    let venv_str = ".venv";
    let venv = Path::new(venv_str);

    let python = if cfg!(windows) {
        ".venv\\Scripts\\python.exe"
    } else {
        ".venv/bin/python"
    };
    let pip = if cfg!(windows) {
        ".venv\\Scripts\\pip.exe"
    } else {
        ".venv/bin/pip"
    };

    if !venv.exists() {
        let status = Command::new("python3")
            .args(["-m", "venv", venv_str])
            .status()
            .expect("failed to create virtual environment");

        assert!(status.success());
    }

    let status = Command::new(pip)
        .args(["install", "-r", "requirements.txt"])
        .status()
        .expect("failed to install Python dependencies");
    assert!(status.success());

    let status = Command::new(python)
        .args(["generate_wordlists.py", out_str])
        .status()
        .expect("failed to generate word lists");
    assert!(status.success());

    wordlists_txt_to_rs(out);
}

fn wordlists_txt_to_rs(out: &Path) {
    let out_file_name = "wordlists.rs";
    let out_file = File::create(out.join(out_file_name)).expect("failed to write wordlists.rs");
    let mut writer = BufWriter::new(out_file);
    let mut letter_counts = Vec::new();

    for entry in fs::read_dir(out).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };

        if !file_name.ends_with(".txt") || !file_name.starts_with("words") {
            continue;
        }
        let Ok(letter_count) =
            file_name[..(file_name.len() - ".txt".len())]["words".len()..].parse::<usize>()
        else {
            continue;
        };
        letter_counts.push(letter_count);

        let wordlist_string = fs::read_to_string(entry.path()).expect("failed to read wordlist");
        let wordlist = wordlist_string
            .lines()
            .inspect(|word| {
                if word.chars().count() != letter_count {
                    panic!("invalid word in wordlist {:?}: {:?}", file_name, word);
                }
            })
            .collect::<Vec<&str>>();

        write_wordlist(&mut writer, letter_count, &wordlist).expect("failed to write wordlists.rs");
    }

    write_metadata(&mut writer, &letter_counts).expect("failed to write wordlists.rs");
}

fn write_wordlist(
    writer: &mut impl Write,
    letter_count: usize,
    wordlist: &[&str],
) -> io::Result<()> {
    writeln!(
        writer,
        "pub static WORDS_{}: &[&str] = &[\"{}\"];",
        letter_count,
        wordlist.as_ref().join("\", \"")
    )?;

    Ok(())
}

fn write_metadata(writer: &mut impl Write, letter_counts: &[usize]) -> io::Result<()> {
    writeln!(
        writer,
        "static WORDLISTS: &[(usize, &[&str])] = &[{}];",
        letter_counts
            .iter()
            .map(|count| format!("({}, WORDS_{})", count, count))
            .collect::<Vec<String>>()
            .join(", ")
    )?;

    Ok(())
}
