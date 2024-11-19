use crate::icons::Icon;

pub enum Stmt {
    Class(Icon),
    Title(Icon),
    Default(Icon),
    Empty(Icon),
    FmtBefore(String),
    Fmt(String),
    FmtAfter(String),
    None
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Class,
    Title,
    String(String),
    Icon(char),
    Default,
    Empty,
    Reversed,
    Before,
    Fmt,
    After,
    Color(String),
    NormalColor,
    FocusedColor,
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

    fn get_current_char(&self) -> Option<char> {
        self.text.chars().nth(self.pos)
    }

    fn peek(&self) -> Option<char> {
        self.text.chars().nth(self.pos+1)
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

    fn skip_comment(&mut self) {
        self.advance();
        while let Some(c) = self.get_current_char() {
            if c == '\n' { break; }
            self.advance();
        }

        self.advance();
    }

    fn color(&mut self) -> Token {
        let mut res = String::new();
        while let Some(cur_char) = self.get_current_char() {
            self.advance();
            if cur_char.is_whitespace() { break; }
            res.push(cur_char);
        }

        Token::Color(res)
    }

    fn string(&mut self) -> Token {
        let mut res = String::new();
        let mut last_char = ' ';
        self.advance();

        while let Some(cur_char) = self.get_current_char() {
            self.advance();
            if cur_char == '\\' && last_char != '\\' { last_char = cur_char; continue; }
            if cur_char == '"' && last_char != '\\' { break; }
            last_char = cur_char;
            res.push(cur_char);
        }

        Token::String(res)
    }

    fn id(&mut self) -> Result<Token, std::io::Error> {
        let mut res = String::new();
        while let Some(cur_char) = self.get_current_char() {
            if cur_char.is_whitespace() { break; }
            res.push(cur_char);
            self.advance();
        }

        match res.as_str() {
            "default" => Ok(Token::Default),
            "empty" => Ok(Token::Empty),
            "title" => Ok(Token::Title),
            "class" => Ok(Token::Class),
            "reversed" => Ok(Token::Reversed),
            "before_fmt" => Ok(Token::Before),
            "fmt" => Ok(Token::Fmt),
            "after_fmt" => Ok(Token::After),
            "color" => Ok(Token::NormalColor),
            "focused_color" => Ok(Token::FocusedColor),
            _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "unknown token"))
        }
    }

    fn next_token(&mut self) -> Result<Token, std::io::Error> {
        self.skip_whitespace();
        while let Some(cur_char) = self.get_current_char() {

            if cur_char == '#' && self.peek() == Some('#') {
                self.skip_comment();
                self.skip_whitespace();
                continue;
            }

            if cur_char == '"' {
                return Ok(self.string())
            }

            if cur_char == '#' {
                return Ok(self.color())
            }

            if cur_char.is_ascii() {
                return self.id()
            }
            
            self.advance();
            return Ok(Token::Icon(cur_char));
        }

        return Ok(Token::EOF);
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
        self.current_token = self.lexer.next_token().unwrap();
    }

    fn eat(&mut self, expected_token: Token) -> Result<(), std::io::Error> {
        if self.current_token != expected_token {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("unexpected token eat: {:?}", &self.current_token)))
        }

        self.current_token = self.lexer.next_token()?;
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

    fn color(&mut self) -> Result<Option<String>, std::io::Error> {
        let old_token = self.current_token.clone();

        if let Token::Color(c) = old_token {
            self.eat(Token::Color(c.clone()))?;
            Ok(Some(c))
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("unexpected token color: {:?}", old_token)))
        }
    }

    fn color_focused(&mut self) -> Result<Option<String>, std::io::Error> {
        let old_token = self.current_token.clone();

        if let Token::FocusedColor = old_token {
            self.eat(Token::FocusedColor)?;
            Ok(self.color()?)
        } else {
            return Ok(None);
        }
    }


    fn color_normal(&mut self) -> Result<Option<String>, std::io::Error> {
        let old_token = self.current_token.clone();
        if let Token::NormalColor = old_token {
            self.eat(Token::NormalColor)?;
            Ok(self.color()?)
        } else {
            return Ok(None);
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

    fn string(&mut self) -> Result<String, std::io::Error> {
        let old_token = self.current_token.clone();
        if let Token::String(r) = old_token {
            self.eat(Token::String(r.to_string()))?;
            Ok(r)
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("unexpected token regex: {:?}", old_token)))
        }
    }

    fn icon_statement(&mut self) -> Result<Icon, std::io::Error> {
        let regex = self.string()?;
        let icon = self.icon()?;
        let color = self.color_normal()?;
        let fcolor = self.color_focused()?;
        let reversed = self.reversed()?;

        Ok(Icon::new(icon, &regex, color, fcolor, reversed)?)
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
        let color = self.color_normal()?;
        let fcolor = self.color_focused()?;
        let reversed = self.reversed()?;

        Ok(Stmt::Default(Icon::new(icon, "", color, fcolor, reversed)?))
    }

    fn empty_statement(&mut self) -> Result<Stmt, std::io::Error>  {
        self.eat(Token::Empty)?;
        let icon = self.icon()?;
        let color = self.color_normal()?;
        let fcolor = self.color_focused()?;
        let reversed = self.reversed()?;

        Ok(Stmt::Empty(Icon::new(icon, "", color, fcolor, reversed)?))
    }

    fn before_statement(&mut self) -> Result<Stmt, std::io::Error> {
        self.eat(Token::Before)?;
        Ok(Stmt::FmtBefore(self.string()?))
    }

    fn fmt_statement(&mut self) -> Result<Stmt, std::io::Error> {
        self.eat(Token::Fmt)?;
        Ok(Stmt::Fmt(self.string()?))
    }

    fn after_statement(&mut self) -> Result<Stmt, std::io::Error> {
        self.eat(Token::After)?;
        Ok(Stmt::FmtAfter(self.string()?))
    }

    pub fn parse(&mut self) -> Result<Stmt, std::io::Error> {
        use Token::*;

        return match &self.current_token {
            Default => self.default_statement(),
            Empty => self.empty_statement(),
            Class => self.class_statement(),
            Title => self.title_statement(),
            Before => self.before_statement(),
            Fmt => self.fmt_statement(),
            After => self.after_statement(),
            EOF => Ok(Stmt::None),
            _ => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("unexpected token parse: {:?}", &self.current_token)))
        }
    }


}

