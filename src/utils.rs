pub mod util {
    use std::{collections::HashMap, lazy::SyncOnceCell};

    // use jieba_rs::Jieba;
    use pinyin::ToPinyin;

    // pub static TOKENIZER: SyncOnceCell<Jieba> = SyncOnceCell::new();  
    pub static TRANSFORM_PUNCTUATION: SyncOnceCell<HashMap<char, char>> = SyncOnceCell::new();
    
    /// 判断字符是否是中文
    pub fn is_chinese(cp: &char) -> bool {
        match *cp {
            '\u{4E00}'..='\u{9FFF}' | 
            '\u{2E80}'..='\u{2EFF}' | 
            '\u{31C0}' ..= '\u{31EF}' |
            '\u{2F00}' ..= '\u{2FFF}' | 
            '\u{3200}' ..= '\u{32FF}' |
            '\u{F900}' ..= '\u{FAFF}' => true,
            _ => false,
        }
    }

    // 将常见的中文标点转换为英文标点
    pub fn transform_punctuation(cp: &char) -> Option<u8>{
        let map = TRANSFORM_PUNCTUATION.get_or_init(|| {
            let mut map = HashMap::new();
            map.insert('。', '.');
            map.insert('，', ',');
            map.insert('！', '!');
            map.insert('‘', '\'');
            map.insert('’', '\'');
            map.insert('；',';');
            map.insert('：', ':'); 
            map.insert('“', '\"');
            map.insert('”', '\"');
            map.insert('、', '\\');
            map.insert('《', '<');
            map.insert('》', '>');
            map.insert('？', '?');
            map.insert('（', '(');
            map.insert('）', ')');
            map
        });
        map.get(cp).map(|v| *v as u8)
    }
    pub fn transform_pinyin(cp: &char) -> Vec<u8> {
        cp.to_pinyin().map(|pinyin| {
            pinyin.plain().as_bytes().into_iter().map(|&b| b).collect()
        }).unwrap()
    }

    pub fn transform(s: &str) -> (usize, String) {
        let mut res: Vec<u8> = Vec::new(); 
        let mut chinese_cnt = 0;
        s.chars().for_each(|cp| {
            match is_chinese(&cp) {
                true => {
                    res.append(&mut transform_pinyin(&cp));
                    chinese_cnt += 1;
                }
                false => {
                    if let Some(c) = transform_punctuation(&cp) {
                        res.push(c);
                        chinese_cnt += 1;
                    } else {
                        res.push(cp as u8);
                    }
                }
            }
        });
        (chinese_cnt, String::from_utf8(res).unwrap())
    }

    // pub fn tokenize(s: &str) -> Vec<&str> {
    //     let tokenize = TOKENIZER.get_or_init(|| {
    //         Jieba::new()
    //     });
    //     tokenize.cut(s, false)
    // }
}

#[cfg(test)]
mod test {
    use super::{util::*};

    #[test]
    fn test_is_chinese() {
        assert_eq!(is_chinese(&'曁'), true);
    }

    #[test]
    fn test_transform_punctuation() {
        // let c = '》';
        let punctuaton = "《》。，；‘’：“”";
        let en = "<>.,;\'\':\"\"";
        let mut res = Vec::new();
        for c in punctuaton.chars().into_iter() {
            println!("{} = {}, {}", c, c.escape_unicode(), is_chinese(&c));
            res.push(transform_punctuation(&c).unwrap());
        }
        assert_eq!(String::from_utf8(res).unwrap(), en);
    }
    #[test]
    fn test_transform_pinyin() {
        let s = "测试一下拼音!!asdji";
        let pinyin = "ceshiyixiapinyin";
        let mut pinyin_list = Vec::new();
        s.chars().filter(|ch| is_chinese(ch)).for_each(|ch| {
            let p = &mut transform_pinyin(&ch);
            pinyin_list.append(p);
        });
        let res = String::from_utf8(pinyin_list).unwrap();
        println!("{}", res);
        assert_eq!(res, pinyin);
    }

    #[test]
    fn test_transform() {
        let s = "①子曰：“不仁者不可以久处约，不可以长处乐。仁者安仁，知者利仁。”";
        // let t = "ceshiyixiapinyin!!asdji";
        let (_, res) = transform(s);
        println!("{}", res);
        // assert_eq!(res, t);
    }

    // #[test]
    // fn test_tokenize() {
    //     let token_list = tokenize("毫无疑问，银行系统的去杠杆化程度既不能满足监管者，也不能令市场满意，这限制了信用增长的势头。");
    //     let res = token_list.into_iter().map(|token| transform(token)).collect::<Vec<String>>().join(" ");
    //     println!("{}", res);
    // }
}
