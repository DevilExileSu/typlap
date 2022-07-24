use std::{io::{Read}, path::{Path}, fs::File};
use anyhow::Result;
use rand::prelude::SliceRandom;


pub struct TextGenerator {
    words: Vec<String>,
    length: usize,
}

pub struct IntoIter {
    words: Vec<String>,
    choice_idx: Vec<usize>,
    cur_idx: usize,
}

impl TextGenerator {
    pub fn new() -> Self {
        Self { 
            words: Vec::new(), 
            length: 0, 
        }
    }

    pub fn read_content(&mut self, file_path: &str) -> Result<()>{
        let path =  Path::new(file_path);
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
       
        self.words = content
            .split_ascii_whitespace()
            .map(|line| String::from(line))
            .collect();

        self.length = self.words.len();
        Ok(())
    }

    pub fn into_iter(&self) -> IntoIter{
        let mut rng = rand::thread_rng();
        let mut choice_idx = (1..self.length).collect::<Vec<usize>>();
        choice_idx.shuffle(&mut rng);
        IntoIter { 
            words: self.words.clone(), 
            choice_idx: choice_idx, 
            cur_idx: 0, 
        }
    }

}

impl Iterator for IntoIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_idx >= self.choice_idx.len() {
            return None
        }
        let word = self.words[self.choice_idx[self.cur_idx]].clone();
        self.cur_idx += 1;
        Some(word)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn text_gen(){
        let mut tg = TextGenerator::new();
        tg.read_content("./src/text/it.txt").unwrap();
        let mut iter = tg.into_iter();
        for i in iter.next() {
            println!("{:?}", i);
        }
        "↵";
    }
}