//! 字句解析(Lexer)。手書き。
//!
//! - コメントは `//` と `/* */`(ネスト可 ★未決)
//! - `'a`(ライフタイム)と `'a'`(文字)は、閉じの `'` の有無で区別する
//! - 識別子は ASCII の英数字と `_`

use crate::token::{Keyword, Punct, Span, Token, TokenKind};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.span.line, self.span.col, self.message)
    }
}

#[derive(Debug)]
pub struct LexOutput {
    /// 末尾は必ず `TokenKind::Eof`。
    pub tokens: Vec<Token>,
    pub errors: Vec<LexError>,
}

pub fn lex(src: &str) -> LexOutput {
    let mut lx = Lexer::new(src);
    lx.run();
    LexOutput {
        tokens: lx.tokens,
        errors: lx.errors,
    }
}

type Mark = (usize, u32, u32);

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: u32,
    col: u32,
    tokens: Vec<Token>,
    errors: Vec<LexError>,
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

impl Lexer {
    fn new(src: &str) -> Self {
        Lexer {
            chars: src.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn mark(&self) -> Mark {
        (self.pos, self.line, self.col)
    }

    fn span_from(&self, m: Mark) -> Span {
        Span {
            start: m.0,
            end: self.pos,
            line: m.1,
            col: m.2,
        }
    }

    fn error(&mut self, span: Span, msg: impl Into<String>) {
        self.errors.push(LexError {
            message: msg.into(),
            span,
        });
    }

    fn run(&mut self) {
        loop {
            self.skip_trivia();
            let m = self.mark();
            let Some(c) = self.peek() else {
                let span = self.span_from(m);
                self.tokens.push(Token {
                    kind: TokenKind::Eof,
                    span,
                    joint: false,
                });
                break;
            };
            let kind = if is_ident_start(c) {
                self.ident()
            } else if c.is_ascii_digit() {
                self.number()
            } else {
                match c {
                    '"' => self.string(m),
                    '`' => self.template(m),
                    '\'' => self.char_or_lifetime(m),
                    _ => self.punct(m),
                }
            };
            if let Some(kind) = kind {
                let span = self.span_from(m);
                let joint = matches!(self.peek(), Some(n) if !n.is_whitespace());
                self.tokens.push(Token { kind, span, joint });
            }
        }
    }

    // ---- 空白とコメント ----

    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.peek_at(1) == Some('/') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                Some('/') if self.peek_at(1) == Some('*') => self.block_comment(),
                _ => break,
            }
        }
    }

    fn block_comment(&mut self) {
        let m = self.mark();
        self.bump();
        self.bump();
        let mut depth = 1;
        while depth > 0 {
            match (self.peek(), self.peek_at(1)) {
                (Some('/'), Some('*')) => {
                    self.bump();
                    self.bump();
                    depth += 1;
                }
                (Some('*'), Some('/')) => {
                    self.bump();
                    self.bump();
                    depth -= 1;
                }
                (Some(_), _) => {
                    self.bump();
                }
                (None, _) => {
                    let span = self.span_from(m);
                    self.error(span, "unterminated block comment");
                    return;
                }
            }
        }
    }

    // ---- 識別子・キーワード ----

    fn ident(&mut self) -> Option<TokenKind> {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if is_ident_continue(c) {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        Some(match Keyword::from_ident(&s) {
            Some(k) => TokenKind::Keyword(k),
            None => TokenKind::Ident(s),
        })
    }

    // ---- 数値 ----

    fn number(&mut self) -> Option<TokenKind> {
        let m = self.mark();
        let mut text = String::new();

        if self.peek() == Some('0')
            && matches!(self.peek_at(1), Some('x') | Some('X') | Some('b') | Some('B'))
        {
            let radix_char = self.peek_at(1).unwrap();
            let is_hex = radix_char.eq_ignore_ascii_case(&'x');
            text.push('0');
            text.push(radix_char);
            self.bump();
            self.bump();
            let mut digits = 0;
            while let Some(c) = self.peek() {
                if c == '_' {
                    self.bump();
                } else if (is_hex && c.is_ascii_hexdigit()) || (!is_hex && (c == '0' || c == '1')) {
                    text.push(c);
                    digits += 1;
                    self.bump();
                } else {
                    break;
                }
            }
            if digits == 0 {
                let span = self.span_from(m);
                self.error(span, "expected digits after radix prefix");
                return None;
            }
            return self.finish_number(m, TokenKind::Int(text));
        }

        self.digits_into(&mut text);
        let mut is_float = false;

        if self.peek() == Some('.') && matches!(self.peek_at(1), Some(d) if d.is_ascii_digit()) {
            is_float = true;
            text.push('.');
            self.bump();
            self.digits_into(&mut text);
        }

        if matches!(self.peek(), Some('e') | Some('E')) {
            let has_sign = matches!(self.peek_at(1), Some('+') | Some('-'));
            let digit_at = if has_sign { 2 } else { 1 };
            if matches!(self.peek_at(digit_at), Some(d) if d.is_ascii_digit()) {
                is_float = true;
                text.push('e');
                self.bump();
                if has_sign {
                    text.push(self.bump().unwrap());
                }
                self.digits_into(&mut text);
            }
        }

        let kind = if is_float {
            TokenKind::Float(text)
        } else {
            TokenKind::Int(text)
        };
        self.finish_number(m, kind)
    }

    fn digits_into(&mut self, text: &mut String) {
        while let Some(c) = self.peek() {
            if c == '_' {
                self.bump();
            } else if c.is_ascii_digit() {
                text.push(c);
                self.bump();
            } else {
                break;
            }
        }
    }

    fn finish_number(&mut self, m: Mark, kind: TokenKind) -> Option<TokenKind> {
        if matches!(self.peek(), Some(c) if is_ident_start(c) || c.is_ascii_digit()) {
            while matches!(self.peek(), Some(c) if is_ident_continue(c)) {
                self.bump();
            }
            let span = self.span_from(m);
            self.error(span, "invalid suffix on number literal");
            return None;
        }
        Some(kind)
    }

    // ---- 文字列 ----

    fn string(&mut self, m: Mark) -> Option<TokenKind> {
        self.bump();
        let mut s = String::new();
        loop {
            match self.bump() {
                None => {
                    let span = self.span_from(m);
                    self.error(span, "unterminated string literal");
                    return None;
                }
                Some('"') => return Some(TokenKind::Str(s)),
                Some('\\') => {
                    let em = self.mark();
                    match self.read_escape() {
                        Ok(c) => s.push(c),
                        Err(msg) => {
                            let span = self.span_from(em);
                            self.error(span, msg);
                        }
                    }
                }
                Some(c) => s.push(c),
            }
        }
    }

    /// `\` の直後から読む。
    fn read_escape(&mut self) -> Result<char, String> {
        match self.bump() {
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('0') => Ok('\0'),
            Some('\\') => Ok('\\'),
            Some('"') => Ok('"'),
            Some('\'') => Ok('\''),
            Some('$') => Ok('$'),
            Some('`') => Ok('`'),
            Some('u') => {
                if !self.eat('{') {
                    return Err("expected '{' after \\u".to_string());
                }
                let mut hex = String::new();
                loop {
                    match self.bump() {
                        Some('}') => break,
                        Some(c) if c.is_ascii_hexdigit() && hex.len() < 6 => hex.push(c),
                        _ => return Err("invalid unicode escape".to_string()),
                    }
                }
                u32::from_str_radix(&hex, 16)
                    .ok()
                    .and_then(char::from_u32)
                    .ok_or_else(|| "invalid unicode escape".to_string())
            }
            Some(c) => Err(format!("unknown escape sequence '\\{c}'")),
            None => Err("unterminated escape sequence".to_string()),
        }
    }

    // ---- テンプレート文字列 ----

    fn template(&mut self, m: Mark) -> Option<TokenKind> {
        self.bump();
        let mut raw = String::new();
        if self.scan_template_body(&mut raw) {
            Some(TokenKind::Template(raw))
        } else {
            let span = self.span_from(m);
            self.error(span, "unterminated template string");
            None
        }
    }

    /// 開きのバッククォートの直後から読み、閉じのバッククォートまで進める。
    /// `${ }` の内側のネストしたテンプレートにも対応する。
    fn scan_template_body(&mut self, out: &mut String) -> bool {
        loop {
            match self.bump() {
                None => return false,
                Some('`') => return true,
                Some('\\') => {
                    out.push('\\');
                    match self.bump() {
                        Some(c) => out.push(c),
                        None => return false,
                    }
                }
                Some('$') if self.peek() == Some('{') => {
                    out.push('$');
                    out.push('{');
                    self.bump();
                    let mut depth = 1;
                    while depth > 0 {
                        match self.bump() {
                            None => return false,
                            Some('{') => {
                                depth += 1;
                                out.push('{');
                            }
                            Some('}') => {
                                depth -= 1;
                                out.push('}');
                            }
                            Some('`') => {
                                out.push('`');
                                if !self.scan_template_body(out) {
                                    return false;
                                }
                                out.push('`');
                            }
                            Some(c) => out.push(c),
                        }
                    }
                }
                Some(c) => out.push(c),
            }
        }
    }

    // ---- 文字リテラルとライフタイム ----

    fn char_or_lifetime(&mut self, m: Mark) -> Option<TokenKind> {
        self.bump(); // 開きの '
        match self.peek() {
            Some('\\') => {
                self.bump();
                let em = self.mark();
                match self.read_escape() {
                    Ok(c) => {
                        if self.eat('\'') {
                            Some(TokenKind::Char(c))
                        } else {
                            let span = self.span_from(m);
                            self.error(span, "unterminated character literal");
                            None
                        }
                    }
                    Err(msg) => {
                        let span = self.span_from(em);
                        self.error(span, msg);
                        None
                    }
                }
            }
            Some(c) if self.peek_at(1) == Some('\'') && c != '\'' => {
                self.bump();
                self.bump();
                Some(TokenKind::Char(c))
            }
            Some(c) if is_ident_start(c) => {
                let mut s = String::new();
                while matches!(self.peek(), Some(c) if is_ident_continue(c)) {
                    s.push(self.bump().unwrap());
                }
                Some(TokenKind::Lifetime(s))
            }
            _ => {
                let span = self.span_from(m);
                self.error(span, "invalid character literal or lifetime");
                None
            }
        }
    }

    // ---- 記号 ----

    fn punct(&mut self, m: Mark) -> Option<TokenKind> {
        let c = self.bump()?;
        let p = match c {
            '(' => Punct::LParen,
            ')' => Punct::RParen,
            '{' => Punct::LBrace,
            '}' => Punct::RBrace,
            '[' => Punct::LBracket,
            ']' => Punct::RBracket,
            ',' => Punct::Comma,
            ';' => Punct::Semi,
            ':' => {
                if self.eat(':') {
                    Punct::ColonColon
                } else {
                    Punct::Colon
                }
            }
            '.' => {
                if self.eat('.') {
                    Punct::DotDot
                } else {
                    Punct::Dot
                }
            }
            '?' => Punct::Question,
            '~' => Punct::Tilde,
            '=' => {
                if self.eat('=') {
                    Punct::EqEq
                } else if self.eat('>') {
                    Punct::FatArrow
                } else {
                    Punct::Eq
                }
            }
            '-' => {
                if self.eat('>') {
                    Punct::Arrow
                } else if self.eat('=') {
                    Punct::MinusEq
                } else {
                    Punct::Minus
                }
            }
            '+' => {
                if self.eat('=') {
                    Punct::PlusEq
                } else {
                    Punct::Plus
                }
            }
            '*' => {
                if self.eat('=') {
                    Punct::StarEq
                } else {
                    Punct::Star
                }
            }
            '/' => {
                if self.eat('=') {
                    Punct::SlashEq
                } else {
                    Punct::Slash
                }
            }
            '%' => {
                if self.eat('=') {
                    Punct::PercentEq
                } else {
                    Punct::Percent
                }
            }
            '!' => {
                if self.eat('=') {
                    Punct::NotEq
                } else {
                    Punct::Bang
                }
            }
            '<' => {
                if self.eat('=') {
                    Punct::LtEq
                } else if self.eat('<') {
                    Punct::Shl
                } else {
                    Punct::Lt
                }
            }
            '>' => {
                if self.eat('=') {
                    Punct::GtEq
                } else {
                    Punct::Gt
                }
            }
            '&' => {
                if self.eat('&') {
                    Punct::AndAnd
                } else if self.eat('=') {
                    Punct::AmpEq
                } else {
                    Punct::Amp
                }
            }
            '|' => {
                if self.eat('|') {
                    Punct::OrOr
                } else if self.eat('=') {
                    Punct::PipeEq
                } else {
                    Punct::Pipe
                }
            }
            '^' => {
                if self.eat('=') {
                    Punct::CaretEq
                } else {
                    Punct::Caret
                }
            }
            other => {
                let span = self.span_from(m);
                self.error(span, format!("unexpected character '{other}'"));
                return None;
            }
        };
        Some(TokenKind::Punct(p))
    }
}

// =================================================================
// ============================= Tests =============================
// =================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Keyword as K;
    use crate::token::Punct as P;
    use crate::token::TokenKind as T;

    fn kinds(src: &str) -> Vec<TokenKind> {
        let out = lex(src);
        assert!(out.errors.is_empty(), "errors: {:?}", out.errors);
        out.tokens
            .into_iter()
            .map(|t| t.kind)
            .filter(|k| *k != TokenKind::Eof)
            .collect()
    }

    fn ident(s: &str) -> TokenKind {
        T::Ident(s.to_string())
    }

    #[test]
    fn keywords_round_trip() {
        for k in K::ALL {
            assert_eq!(K::from_ident(k.as_str()), Some(k));
        }
        assert_eq!(K::from_ident("notakeyword"), None);
    }

    #[test]
    fn simple_declaration() {
        assert_eq!(
            kinds("var count = 0;"),
            vec![
                T::Keyword(K::Var),
                ident("count"),
                T::Punct(P::Eq),
                T::Int("0".into()),
                T::Punct(P::Semi),
            ]
        );
    }

    #[test]
    fn let_final_const() {
        assert_eq!(
            kinds("let x final y const Z"),
            vec![
                T::Keyword(K::Let),
                ident("x"),
                T::Keyword(K::Final),
                ident("y"),
                T::Keyword(K::Const),
                ident("Z"),
            ]
        );
    }

    #[test]
    fn self_keywords() {
        assert_eq!(
            kinds("self Self selfish"),
            vec![T::Keyword(K::SelfValue), T::Keyword(K::SelfType), ident("selfish")]
        );
    }

    #[test]
    fn numbers() {
        assert_eq!(
            kinds("42 1_000 0xFF 0b1010 3.14 1e-3 2.5e+2"),
            vec![
                T::Int("42".into()),
                T::Int("1000".into()),
                T::Int("0xFF".into()),
                T::Int("0b1010".into()),
                T::Float("3.14".into()),
                T::Float("1e-3".into()),
                T::Float("2.5e+2".into()),
            ]
        );
    }

    #[test]
    fn range_is_not_float() {
        assert_eq!(
            kinds("1..2"),
            vec![T::Int("1".into()), T::Punct(P::DotDot), T::Int("2".into())]
        );
    }

    #[test]
    fn method_call_on_int() {
        assert_eq!(
            kinds("1.foo"),
            vec![T::Int("1".into()), T::Punct(P::Dot), ident("foo")]
        );
    }

    #[test]
    fn strings_and_escapes() {
        assert_eq!(kinds(r#""a\nb\t\"q\"""#), vec![T::Str("a\nb\t\"q\"".into())]);
        assert_eq!(kinds(r#""\u{3042}""#), vec![T::Str("あ".into())]);
    }

    #[test]
    fn template_strings() {
        assert_eq!(
            kinds("`x = ${x}`"),
            vec![T::Template("x = ${x}".into())]
        );
        assert_eq!(
            kinds("`a ${`b ${c}`} d`"),
            vec![T::Template("a ${`b ${c}`} d".into())]
        );
    }

    #[test]
    fn char_vs_lifetime() {
        assert_eq!(
            kinds(r"&'a Foo 'x' '\n' 'あ'"),
            vec![
                T::Punct(P::Amp),
                T::Lifetime("a".into()),
                ident("Foo"),
                T::Char('x'),
                T::Char('\n'),
                T::Char('あ'),
            ]
        );
    }

    #[test]
    fn lifetime_in_generics() {
        assert_eq!(
            kinds("function first<'a>(xs: &'a [Int])"),
            vec![
                T::Keyword(K::Function),
                ident("first"),
                T::Punct(P::Lt),
                T::Lifetime("a".into()),
                T::Punct(P::Gt),
                T::Punct(P::LParen),
                ident("xs"),
                T::Punct(P::Colon),
                T::Punct(P::Amp),
                T::Lifetime("a".into()),
                T::Punct(P::LBracket),
                ident("Int"),
                T::Punct(P::RBracket),
                T::Punct(P::RParen),
            ]
        );
    }

    #[test]
    fn comments_including_nested() {
        assert_eq!(
            kinds("a // line\n b /* c /* d */ e */ f"),
            vec![ident("a"), ident("b"), ident("f")]
        );
    }

    #[test]
    fn closing_angle_brackets_stay_separate_and_joint() {
        let out = lex("Array<Array<Int>>");
        assert!(out.errors.is_empty());
        let toks: Vec<_> = out
            .tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Eof)
            .collect();
        assert_eq!(toks.len(), 7);
        assert_eq!(toks[5].kind, T::Punct(P::Gt));
        assert_eq!(toks[6].kind, T::Punct(P::Gt));
        assert!(toks[5].joint);
        assert!(!toks[6].joint);
    }

    #[test]
    fn operators() {
        assert_eq!(
            kinds("=> -> :: .. ? <= >= << == != && || += -= *= /= %= &= |= ^="),
            vec![
                T::Punct(P::FatArrow),
                T::Punct(P::Arrow),
                T::Punct(P::ColonColon),
                T::Punct(P::DotDot),
                T::Punct(P::Question),
                T::Punct(P::LtEq),
                T::Punct(P::GtEq),
                T::Punct(P::Shl),
                T::Punct(P::EqEq),
                T::Punct(P::NotEq),
                T::Punct(P::AndAnd),
                T::Punct(P::OrOr),
                T::Punct(P::PlusEq),
                T::Punct(P::MinusEq),
                T::Punct(P::StarEq),
                T::Punct(P::SlashEq),
                T::Punct(P::PercentEq),
                T::Punct(P::AmpEq),
                T::Punct(P::PipeEq),
                T::Punct(P::CaretEq),
            ]
        );
    }

    #[test]
    fn module_path_tokens() {
        assert_eq!(
            kinds("import module { Random.lib/Random };"),
            vec![
                T::Keyword(K::Import),
                T::Keyword(K::Module),
                T::Punct(P::LBrace),
                ident("Random"),
                T::Punct(P::Dot),
                ident("lib"),
                T::Punct(P::Slash),
                ident("Random"),
                T::Punct(P::RBrace),
                T::Punct(P::Semi),
            ]
        );
    }

    #[test]
    fn spans_track_line_and_column() {
        let out = lex("var x\n  let");
        let t = &out.tokens;
        assert_eq!((t[0].span.line, t[0].span.col), (1, 1));
        assert_eq!((t[1].span.line, t[1].span.col), (1, 5));
        assert_eq!((t[2].span.line, t[2].span.col), (2, 3));
    }

    #[test]
    fn reports_errors_and_keeps_going() {
        let out = lex("\"abc");
        assert_eq!(out.errors.len(), 1);

        let out = lex("12abc let");
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.tokens[0].kind, T::Keyword(K::Let));

        let out = lex("/* never closed");
        assert_eq!(out.errors.len(), 1);

        let out = lex("0x");
        assert_eq!(out.errors.len(), 1);

        let out = lex("a # b");
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.tokens.len(), 3); // a, b, eof
    }
}