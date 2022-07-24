#![feature(once_cell)]
mod utils;
mod textgen;
mod tui;
mod evaluator;
use std::{io::BufReader, fs::File, time::Instant};

use crossterm::event;
use rodio::{OutputStream, Decoder, Source, Sink};
use anyhow::Result;

pub struct Typlap {
    pub ui: tui::Tui,
    pub evaluator: evaluator::Evaluator,
    pub started: bool,
    pub done: bool,
    // stream_handle: OutputStreamHandle,
}

impl Typlap {
    pub fn new(file_path: &str) -> Self {
        let mut tg = textgen::TextGenerator::new();
        tg.read_content(file_path).unwrap();
        let iter = tg.into_iter();
        Self { 
            ui: tui::Tui::new(iter).unwrap(),
            evaluator: evaluator::Evaluator::new(),
            started: false,
            done: false,
        }
    }


    pub fn typing(&mut self) -> Result<()>{
        self.ui.init()?;
        let file = BufReader::new(File::open("./bee.wav").unwrap());
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let source = Decoder::new(file).unwrap();
        let source_buf = source.buffered();
        let mut start_at = Instant::now();
        loop {
            match event::read()? {
                event::Event::Key(event) => {
                    if !self.started {
                        self.evaluator.reset();
                        self.started = true;
                        start_at = Instant::now();
                    }
                    let sink = Sink::try_new(&stream_handle).unwrap();
                    sink.append(source_buf.clone());
                    sink.detach();
                    match (event.code, event.modifiers) {
                        (event::KeyCode::Char(mut ch), event::KeyModifiers::NONE | event::KeyModifiers::SHIFT)=> {
                            if self.done {
                                continue
                            }
                            if event.modifiers == event::KeyModifiers::SHIFT {
                                ch = ch.to_ascii_uppercase()
                            }
                            match self.ui.display_c(&ch) {
                                Ok(correct) => {
                                    if correct {
                                        self.evaluator.final_chars_typed_correctly += 1;
                                    } else {
                                        self.evaluator.total_char_errors += 1;
                                        self.evaluator.final_uncorrected_errors += 1;
                                    }
                                    self.evaluator.total_chars_typed += 1;
                                },
                                Err(e) => return Err(e)
                            }
                        }

                        (event::KeyCode::Backspace, event::KeyModifiers::NONE) => {
                            if self.done {
                                continue
                            }
                            match self.ui.display_with_backspace() {
                                Ok(res) => {
                                    if res == -1 {
                                        self.evaluator.final_chars_typed_correctly -= 1;
                                    } else if res == 1 {
                                        self.evaluator.final_uncorrected_errors -= 1;
                                    }
                                },
                                Err(e) => return Err(e),
                            }
                        }

                        (event::KeyCode::Enter, event::KeyModifiers::NONE) => {
                            if self.done {
                                continue
                            }
                            if let Ok((done, correct)) = self.ui.move_next_line() {
                                if correct {
                                    self.evaluator.final_chars_typed_correctly += 1;
                                } else {
                                    self.evaluator.final_uncorrected_errors += 1;
                                }
                                self.evaluator.total_chars_typed += 1;
                                if done == true {
                                    let end_at = std::time::Instant::now();
                                    self.ui.display_result(self.evaluator.done(end_at.duration_since(start_at)))?;
                                    self.started = false;
                                    self.done = done;
                                }
                            }
                        }
                        
                        (event::KeyCode::Char('n'), event::KeyModifiers::CONTROL) => {
                            self.evaluator.reset();
                            self.started = false;
                            self.done = false;
                            self.ui.init()?;
                        }

                        (event::KeyCode::Char('r'), event::KeyModifiers::CONTROL) => {
                            self.evaluator.reset();
                            self.started = false;
                            self.done = false;
                            self.ui.reset()?;
                        }

                        (event::KeyCode::Esc, event::KeyModifiers::NONE) => {
                            break;
                        } 
                        _ => {}
                    }
                }
                _ => {}
            }
            if self.started {
                let end_at = std::time::Instant::now();
                self.ui.display_result(self.evaluator.snap(end_at.duration_since(start_at)))?;
            }
        }
        Ok(())
    }
}



fn main() -> Result<()> {
    let mut t = Typlap::new("./text/it.txt");
    t.typing().unwrap();
    Ok(())
}