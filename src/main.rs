use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use std::time::Instant;
use termion::color;
use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;

const WPM_LIMIT: u32 = 50;
const CORRECT_LIMIT: u32 = 90;

fn open_file() -> BufReader<File> {
    let args: Vec<String> = std::env::args().collect();
    let file = File::open(&args[1]).unwrap();
    BufReader::new(file)
}

fn get_key(keys: &mut termion::input::Keys<std::io::Stdin>) -> char {
    for key in keys {
        match key.unwrap() {
            Key::Ctrl('c') => panic!("got ctrl-c"),
            Key::Char(c) => return c,
            _ => continue,
        }
    }

    panic!("something bad happened");
}

fn left_pad(num: u32, len: usize) -> String {
    let mut p = String::new();
    let mut s = num.to_string();
    while (s.len() + p.len()) < len { p.push(' '); }
    p.push_str(&s);
    p
}

fn right_pad(num: u32, len: usize) -> String {
    let mut s = num.to_string();
    while s.len() < len { s.push(' '); }
    s
}

fn main() {
    let file = open_file();

    let mut stdout = std::io::stdout().into_raw_mode().unwrap();
    let mut keys = std::io::stdin().keys();

    for line in WrappedLines::new(file.lines(), 80) {
        let line: String = line
                            .trim()
                            .chars()
                            .filter(|c| c.is_ascii())
                            .map(|c| if c.is_whitespace() { ' ' } else { c } )
                            .collect();

        write!(
            stdout,
            "{}{}{}",
            "\r\n         ",
            line,
            "\r         "
        ).unwrap();
        stdout.flush().unwrap();

        let mut success = false;
        while !success {
            let start = Instant::now();
            let mut chars = 0;
            let mut errors = 0;

            for c in line.chars() {
                if !c.is_ascii() {
                    write!(stdout, "{}", termion::cursor::Right(1));
                    continue;
                }
                stdout.flush().unwrap();

                while c != get_key(&mut keys) {
                    errors += 1;

                    write!(stdout, "{}", 7 as char);
                    stdout.flush().unwrap();
                }

                chars += 1;

                write!(stdout, "{}", termion::cursor::Right(1));
                stdout.flush().unwrap();
            }

            if chars != 0 {
                let wpm = {
                    let words = (chars / 5) as f32;
                    let minutes = (start.elapsed().as_millis() as f32) / 60_000f32;
                    (words / minutes) as u32
                };

                let correct = (
                    ((chars as f32) / ((chars + errors) as f32)) * 100f32
                ) as u32;

                let wpm_success = wpm >= WPM_LIMIT || chars < 15;
                let correct_success = correct >= CORRECT_LIMIT;

                let wpm_color = if wpm_success {
                    format!("{}", color::Fg(color::LightBlack))
                } else {
                    format!("{}", color::Fg(color::Red))
                };

                let correct_color = if correct_success {
                    format!("{}", color::Fg(color::LightBlack))
                } else {
                    format!("{}", color::Fg(color::Red))
                };

                write!(
                    stdout,
                    "\r{}{}% {}{}{}",
                    correct_color,
                    left_pad(correct, 3),
                    wpm_color,
                    right_pad(wpm, 4),
                    color::Fg(color::Reset)
                );

                success = wpm_success && correct_success;
            } else {
                success = true;
            }

            if !success {
                write!(stdout, "\r{}", termion::cursor::Right(9));
            }
        }
    }
}

struct WrappedLines {
    lines: std::io::Lines<std::io::BufReader<std::fs::File>>,
    buffer: String,
    width: usize,
}

impl WrappedLines {
    fn new(lines: std::io::Lines<std::io::BufReader<std::fs::File>>, width: usize) -> WrappedLines {
        WrappedLines {
            lines,
            width,
            buffer: String::new()
        }
    }
}

impl Iterator for WrappedLines {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if self.buffer.len() == 0 {
            let next = self.lines.next();
            if let Some(Ok(next)) = next {
                self.buffer = next;
            } else {
                return None;
            }
        }

        if self.buffer.len() <= self.width {
            let mut s = String::new();
            std::mem::swap(&mut s, &mut self.buffer);
            return Some(s);
        } else {
            let mut tmp = self.buffer.split_off(self.width);
            std::mem::swap(&mut tmp, &mut self.buffer);
            return Some(tmp);
        }
    }
}
