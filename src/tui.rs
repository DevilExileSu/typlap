use std::io::{Stdout, stdout, Write};
use anyhow::Result;
use crossterm::{terminal::{self, size, Clear, SetSize}, execute, cursor::{self, MoveTo}, style::{Print, Stylize, Color, Attribute}};
use crate::{textgen, utils::util::{self, is_chinese, transform_punctuation,}, evaluator::EvalResult};

const MIN_HEIGHT:u16 = 6;
const MIN_WIDTH:u16 = 45;

pub struct TextArea {
    pub word_iter: textgen::IntoIter,
    pub raw_text: Vec<Vec<char>>,
    pub pinyin_text: Vec<String>,
    pub pos: Vec<LinePos>,
    pub cols: u16,
    pub rows: u16,
}

pub struct LinePos {col: u16, row: u16}


pub struct Tui {
    pub stdout: Stdout,
    pub text: TextArea,
    pub cols: u16,
    pub rows: u16,
    pub cursor_col: u16,
    pub cursor_row: u16,
    pub input: String,
    pub chinese: bool,
}

impl Tui {
    pub fn new(word_iter: textgen::IntoIter) -> Result<Self> {
        let (cols, rows) = size()?;
        let mut stdout = stdout();
        if cols < MIN_WIDTH || rows < MIN_HEIGHT {
            execute!(stdout, SetSize(MIN_WIDTH, MIN_HEIGHT)).map_err(|_| anyhow::Error::msg("终端窗口太小!"))?;
        }
        Ok(Self { 
            stdout: stdout,
            text: TextArea {
                word_iter: word_iter,
                raw_text: Vec::new(),
                pinyin_text: Vec::new(), 
                pos: Vec::new(), 
                cols: 0, 
                rows: 0,
            },
            cols: cols,
            rows: rows,
            cursor_col: 0,
            cursor_row: 0,
            input: String::new(),
            chinese: false,
        })
    }

    pub fn set_size(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
    }

    pub fn init(&mut self) -> Result<()>{
        terminal::enable_raw_mode()?;
        execute!(self.stdout, Clear(terminal::ClearType::All), terminal::SetSize(self.cols, self.rows), cursor::Show, cursor::SetCursorShape(cursor::CursorShape::Line))?;
        self.init_bound()?;
        self.init_text()?;
        self.init_footer()?;
        self.cursor_col  = self.text.pos[0].col;
        self.cursor_row = self.text.pos[0].row;
        execute!(self.stdout, MoveTo(self.cursor_col, self.cursor_row))?;
        self.stdout.flush()?;
        Ok(())
    }

    fn init_bound(&mut self) -> Result<()>{
        for y in 0..self.rows {
            for x in 0..self.cols {
              if y == 0 || y == self.rows - 1 {
                execute!(self.stdout, cursor::MoveTo(x,y), Print("-"))?;
              }
              if x == 0 || x == self.cols - 1 {
                execute!(self.stdout, cursor::MoveTo(x,y), Print("|"))?;
              }
            }
        }
        Ok(())
    }

    pub fn display_result(&mut self, res: EvalResult) -> Result<()> {
        match res {
            EvalResult::Snap(acc, wpm) => {
                let clear_pad = " ".repeat((self.cols - 2) as usize);
                let acc_prefix = "current Accuracy: ";
                let wpm_prefix = ", current Wpm: ";
                let acc = format!("{:.1}%", acc * 100.0).with(Color::Magenta);
                let wpm = format!("{:.1}", wpm).with(Color::Magenta);
                let length = acc_prefix.len() + wpm_prefix.len() + acc.content().len() + wpm.content().len();
                let cols = (self.cols - length as u16) / 2;
                execute!(
                    self.stdout,
                    cursor::MoveTo(1,1),
                    Print(clear_pad),
                    cursor::MoveTo(cols,1),
                    Print(acc_prefix),
                    Print(acc),
                    Print(wpm_prefix),
                    Print(wpm),
                    cursor::MoveTo(self.cursor_col, self.cursor_row),
                )?;

            },
            EvalResult::Done(delta, acc, real_acc, wpm) => {
                execute!(self.stdout, Clear(terminal::ClearType::All), terminal::SetSize(self.cols, self.rows), cursor::Hide)?;
                self.init_bound()?;
                let first = format!("Tooks {}s for {} words", delta.as_secs(), self.count_char());
                let acc = format!("Accuracy: {:.1}%", acc * 100.0).with(Color::Magenta);
                let real_acc = format!("Real Accuracy: {:.1}%", real_acc * 100.0).with(Color::Cyan);
                let wpm_prefix = format!("Speed: ");
                let wpm = format!("{:.1} wpm", wpm).with(Color::Green);
                let wpm_suffix = format!(" (words per minute)");
                
                let rows = self.rows/2 - 2;
                let wpm_length = (wpm_prefix.len() + wpm.content().len() + wpm_suffix.len()) as u16;
                execute!(
                    self.stdout,
                    MoveTo((self.cols - first.len()  as u16)/2, rows),
                    Print(first),
                    MoveTo((self.cols - acc.content().len()  as u16)/2, rows+1),
                    Print(acc),
                    MoveTo((self.cols - real_acc.content().len()  as u16)/2, rows+2),
                    Print(real_acc),
                    MoveTo((self.cols - wpm_length)/2, rows+3),
                    Print(wpm_prefix),
                    Print(wpm),
                    Print(wpm_suffix),
                )?;
                self.init_footer()?;
            },
        }
        Ok(())
    }

    pub fn init_text(&mut self) -> Result<()>{
        self.text.raw_text.clear();
        self.text.pinyin_text.clear();
        self.text.pos.clear();
        self.input.clear();
        let max_text_rows = self.rows / 4;
        let max_text_cols = self.cols / 5 * 3;

        let cursor_rows = (self.rows - max_text_rows) / 2 - self.rows / 6;
        let mut is_empty = false;
        while self.text.raw_text.len() < max_text_rows as usize && !is_empty{
            let mut line = String::new();
            let mut raw_line = String::new();
            let mut chinese_cnt = Vec::<usize>::new();
            loop {
                let word = self.text.word_iter.next();
                match word {
                    Some(w) => {
                        let (cnt, pinyin) = util::transform(&w);
                        if line.len() + pinyin.len() + 1 < max_text_cols as usize {
                            line.push_str(&pinyin);
                            line.push(' ');
                            raw_line.push_str(&w);
                            raw_line.push(' ');
                            chinese_cnt.push(cnt);
                        } else {
                            raw_line.pop();
                            line.pop();
                            break
                        }
                    }
                    None => {
                        is_empty = true;
                        break
                    }
                }
            }
            line.push('↵');
            raw_line.push('↵');

            if line.ne(&raw_line) {
                let raw_line = line.split_ascii_whitespace().zip(raw_line.split_ascii_whitespace()).zip(chinese_cnt).map(|((pinyin, hans), cnt)| {
                    let length = pinyin.len();
                    let sent_len = hans.chars().count() + cnt;
                    let mut new = String::new();
                    if sent_len < length {
                        let pad_cnt_left = (length - sent_len) / 2;
                        let pad_cnt_right = length - pad_cnt_left - sent_len;
                        new.push_str(&" ".repeat(pad_cnt_left));
                        new.push_str(&hans);
                        new.push_str(&" ".repeat(pad_cnt_right));
                    } else {
                        new.push_str(&hans);
                    }
                    new
                }).collect::<Vec<String>>().join(" ");

                execute!(
                    self.stdout,
                    MoveTo((self.cols - line.len() as u16) / 2, cursor_rows + (self.text.raw_text.len() * 2) as u16),
                    Print(&raw_line),
                    MoveTo((self.cols - line.len() as u16) / 2, cursor_rows + (self.text.raw_text.len() * 2 + 1) as u16),
                    Print(&line)
                )?;
                self.text.pos.push(LinePos { 
                    col: (self.cols - line.len() as u16) / 2,
                    row: cursor_rows + (self.text.raw_text.len() * 2 + 1) as u16,
                });
                self.text.raw_text.push(line.chars().collect::<Vec<char>>());
                self.text.pinyin_text.push(raw_line);
            } else {
                self.text.pos.push(LinePos { 
                    col: (self.cols - line.len() as u16) / 2,
                    row: cursor_rows + self.text.raw_text.len() as u16,
                });
                execute!(self.stdout, MoveTo((self.cols - line.len() as u16) / 2, cursor_rows + self.text.raw_text.len() as u16), Print(&line))?;
                self.text.raw_text.push(line.chars().collect::<Vec<char>>());
                self.text.pinyin_text.push(String::new());
            }
        }
        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        self.input.clear();
        self.cursor_col = self.text.pos[0].col;
        self.cursor_row = self.text.pos[0].row;
        execute!(self.stdout,  Clear(terminal::ClearType::All))?; 
        self.init_bound()?;
        for (idx, (line, pinyin)) in self.text.raw_text.iter().zip(self.text.pinyin_text.iter()).enumerate() {
            let line = line.into_iter().collect::<String>();
            execute!(
                self.stdout, 
                MoveTo(self.text.pos[idx].col, self.text.pos[idx].row-1),
                Print(&pinyin),
                MoveTo(self.text.pos[idx].col, self.text.pos[idx].row),
                Print(&line)
            )?;
        }
        self.init_footer()?;
        execute!(self.stdout, MoveTo(self.cursor_col, self.cursor_row))?;
        Ok(())
    }


    fn init_footer(&mut self) -> Result<()> {
        let cursor_rows = self.rows - 1;
        let cursor_cols = (self.cols - 49) / 2;
        let restart = "ctrl-r"
        .with(Color::Blue)
        .attribute(Attribute::Bold);

        let quit = "ESC"
        .with(Color::Blue)
        .attribute(Attribute::Bold);

        let next = "ctrl-n"
        .with(Color::Blue)
        .attribute(Attribute::Bold);

        execute!(self.stdout,
            MoveTo(cursor_cols, cursor_rows),
            Print(restart),
            Print(" to restart, "),
            Print(next),
            Print(" to next, "),
            Print(quit),
            Print(" to quit"),
        )?;
        Ok(())
    }

    fn get_cur_text_line(&mut self) -> u16 {
        if self.text.pos.len() > 1 {
            (self.cursor_row - self.text.pos[0].row) / (self.text.pos[1].row - self.text.pos[0].row)
        } else {
            0
        }
    }

    pub fn display_c(&mut self, ch: &char) -> Result<bool>{
        let cur_line = self.get_cur_text_line() as usize;
        let line_pos = &self.text.pos[cur_line];

        if is_chinese(ch) || transform_punctuation(ch).is_some(){
            return Err(anyhow::Error::msg("请不要开启中文输入法！"));
        }
        // 获取所在行的 位置信息

        let mut stylize_ch = ch.attribute(Attribute::Bold);
        // 获取当前游标所在行的位置
        if self.cursor_col - line_pos.col < self.text.raw_text[cur_line].len() as u16 {
            self.input.push(ch.clone());
            let raw_idx = self.cursor_col - line_pos.col;
            let equal = self.text.raw_text[cur_line][raw_idx as usize] == *ch;
            self.cursor_col += 1;
            if equal {
                stylize_ch = stylize_ch.with(Color::Green);
                execute!(self.stdout, Print(stylize_ch))?; 
                self.stdout.flush()?;
                return Ok(true)
            } else {
                stylize_ch = stylize_ch.with(Color::Red);
                execute!(self.stdout, Print(stylize_ch))?; 
                self.stdout.flush()?;
                return Ok(false)
            }
        }
        Ok(false)
    }
    
    pub fn display_with_backspace(&mut self ) -> Result<i8> {
        let cur_line = self.get_cur_text_line() as usize;
        let line_pos = &self.text.pos[cur_line];
        let raw_idx = self.cursor_col - line_pos.col;

        if raw_idx > 0 {
            self.cursor_col -= 1;
            let old_ch = self.input.pop().unwrap();
            let ch = self.text.raw_text[cur_line][(raw_idx - 1) as usize];

            execute!(self.stdout, cursor::MoveLeft(1), Print(ch), cursor::MoveLeft(1))?;
            self.stdout.flush()?;
            if ch == old_ch {
                return Ok(-1);
            } else {
                return Ok(1);
            }
        } 
         if cur_line != 0{
            self.cursor_col = self.text.pos[cur_line-1].col + self.text.raw_text[cur_line-1].len() as u16;
            self.cursor_row = self.text.pos[cur_line-1].row;
            execute!(self.stdout, MoveTo(self.cursor_col, self.cursor_row))?;
        }

        self.stdout.flush()?;
        Ok(0)
    }

    pub fn move_next_line(&mut self ) -> Result<(bool, bool)>{
        let cur_line = self.get_cur_text_line() as usize;
        match self.display_c(&'↵') {
            Ok(m) => {
                let next_line = cur_line + 1;
                if m && next_line < self.text.raw_text.len() {
                    self.cursor_col = self.text.pos[next_line].col;
                    self.cursor_row = self.text.pos[next_line].row;
                    execute!(self.stdout, MoveTo(self.cursor_col, self.cursor_row))?;
                } else if next_line >= self.text.raw_text.len() && self.text.pos[cur_line].col + self.text.raw_text[cur_line].len() as u16 == self.cursor_col {
                    return Ok((true, m))
                }
                Ok((false, m))
            },
            Err(e) => Err(e),
        }
    }
    
    pub fn count_char(&self) -> usize{
        self.text.raw_text.iter().fold(0, |sum, line| sum + line.len())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
        execute!(self.stdout, Clear(terminal::ClearType::All), MoveTo(0,0)).unwrap();
    }
}

#[cfg(test)]
mod test {

    use crate::textgen;

    use super::*;
    #[test]
    fn test_init() {

        let mut tg = textgen::TextGenerator::new();
        tg.read_content("./text/en.txt").unwrap();
        let iter = tg.into_iter();
        // let text = iter.next().unwrap();
        let mut tui = Tui::new(iter).unwrap();
        tui.init().unwrap();
        println!("{}", tui.count_char());
        assert_eq!(tui.count_char(), 100);
    }
}