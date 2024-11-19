use crate::icons::Icon;

pub enum Stmt {
    Class(Icon),
    Title(Icon),
    Default(Icon),
    Empty(Icon),
    None
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Class,
    Title,
    Regex(String),
    Icon(char),
    Default,
    Empty,
    Reversed,
    EOF
}

pub struct Lexer {
    text: String,
    pos: usize
}

impl Lexer {
    pub fn new() -> Self {
        Self { text: "".to_string(), pos: 0 }
    }

    pub fn feed_next_line(&mut self, line: &str) {
        self.text = line.to_string();
        self.pos = 0;
    }

    fn get_current_char(&mut self) -> Option<char> {
        self.text.chars().nth(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.get_current_char() {
            if !c.is_whitespace() { break; }
            self.advance();
        }
    }

    fn id(&mut self) -> Token {
        let mut res = String::new();
        let mut last_char = ' ';
        while let Some(cur_char) = self.get_current_char() {
            if cur_char.is_whitespace() && last_char != '\\' { break; }
            res.push(cur_char);
            last_char = cur_char;
            self.advance();
        }

        match res.as_str() {
            "default" => Token::Default,
            "empty" => Token::Empty,
            "title" => Token::Title,
            "class" => Token::Class,
            "reversed" => Token::Reversed,
            _ => Token::Regex(res)
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        while let Some(cur_char) = self.get_current_char() {

            if cur_char.is_ascii() {
                return self.id()
            }
            
            self.advance();
            return Token::Icon(cur_char);
        }

        return Token::EOF;
    }
}

pub struct Parser {
    lexer: Lexer,
    current_token: Token
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self { lexer, current_token: Token::EOF }
    }

    pub fn feed_next_line(&mut self, line :&str) {
        self.lexer.feed_next_line(line);
        self.current_token = self.lexer.next_token();
    }

    fn eat(&mut self, expected_token: Token) -> Result<(), std::io::Error> {
        if self.current_token != expected_token {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("unexpected token eat: {:?}", &self.current_token)))
        }

        self.current_token = self.lexer.next_token();
        Ok(())
    }

    fn reversed(&mut self) -> Result<bool, std::io::Error> {
        let old_token = self.current_token.clone();
        if let Token::Reversed = old_token {
            self.eat(Token::Reversed)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn icon(&mut self) -> Result<char, std::io::Error> {
        let old_token = self.current_token.clone();

        if let Token::Icon(i) = old_token {
            self.eat(Token::Icon(i))?;
            Ok(i)
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("unexpected token icon: {:?}", old_token)))
        }
    }

    fn regex(&mut self) -> Result<String, std::io::Error> {
        let old_token = self.current_token.clone();
        if let Token::Regex(r) = old_token {
            self.eat(Token::Regex(r.to_string()))?;
            Ok(r)
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("unexpected token regex: {:?}", old_token)))
        }
    }

    fn icon_statement(&mut self) -> Result<Icon, std::io::Error> {
        let regex = self.regex()?;
        let icon = self.icon()?;
        let reversed = self.reversed()?;

        Ok(Icon::new(icon, &regex, reversed)?)
    }

    fn class_statement(&mut self) -> Result<Stmt, std::io::Error> {
        use Token::*;

        self.eat(Class)?;
        Ok(Stmt::Class(self.icon_statement()?))
    }

    fn title_statement(&mut self) -> Result<Stmt, std::io::Error> {
        use Token::*;

        self.eat(Title)?;
        Ok(Stmt::Title(self.icon_statement()?))
    }

    fn default_statement(&mut self) -> Result<Stmt, std::io::Error>  {
        self.eat(Token::Default)?;
        let icon = self.icon()?;
        let reversed = self.reversed()?;

        Ok(Stmt::Default(Icon::new(icon, "", reversed)?))
    }

    fn empty_statement(&mut self) -> Result<Stmt, std::io::Error>  {
        self.eat(Token::Empty)?;
        let icon = self.icon()?;
        let reversed = self.reversed()?;

        Ok(Stmt::Empty(Icon::new(icon, "", reversed)?))
    }

    pub fn parse(&mut self) -> Result<Stmt, std::io::Error> {
        use Token::*;

        return match &self.current_token {
            Default => self.default_statement(),
            Empty => self.empty_statement(),
            Class => self.class_statement(),
            Title => self.title_statement(),
            EOF => Ok(Stmt::None),
            _ => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("unexpected token parse: {:?}", &self.current_token)))
        }
    }


}

