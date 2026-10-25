//! トークンの定義。

use std::fmt;

/// ソース上の位置。`start` / `end` は文字単位のオフセット(バイトではない)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub col: u32,
}

/// 予約語。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    Var,
    Let,
    Final,
    Const,
    Function,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Loop,
    Break,
    Continue,
    Switch,
    Case,
    Default,
    Struct,
    Enum,
    Trait,
    Impl,
    Class,
    Type,
    Module,
    Namespace,
    Import,
    Using,
    Public,
    Private,
    Static,
    Unsafe,
    True,
    False,
    SelfValue,
    SelfType,
    Move,
    Mut,
    New,
}

impl Keyword {
    pub const ALL: [Keyword; 38] = [
        Keyword::Var,
        Keyword::Let,
        Keyword::Final,
        Keyword::Const,
        Keyword::Function,
        Keyword::Return,
        Keyword::If,
        Keyword::Else,
        Keyword::While,
        Keyword::For,
        Keyword::In,
        Keyword::Loop,
        Keyword::Break,
        Keyword::Continue,
        Keyword::Switch,
        Keyword::Case,
        Keyword::Default,
        Keyword::Struct,
        Keyword::Enum,
        Keyword::Trait,
        Keyword::Impl,
        Keyword::Class,
        Keyword::Type,
        Keyword::Module,
        Keyword::Namespace,
        Keyword::Import,
        Keyword::Using,
        Keyword::Public,
        Keyword::Private,
        Keyword::Static,
        Keyword::Unsafe,
        Keyword::True,
        Keyword::False,
        Keyword::SelfValue,
        Keyword::SelfType,
        Keyword::Move,
        Keyword::Mut,
        Keyword::New,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Keyword::Var => "var",
            Keyword::Let => "let",
            Keyword::Final => "final",
            Keyword::Const => "const",
            Keyword::Function => "function",
            Keyword::Return => "return",
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::While => "while",
            Keyword::For => "for",
            Keyword::In => "in",
            Keyword::Loop => "loop",
            Keyword::Break => "break",
            Keyword::Continue => "continue",
            Keyword::Switch => "switch",
            Keyword::Case => "case",
            Keyword::Default => "default",
            Keyword::Struct => "struct",
            Keyword::Enum => "enum",
            Keyword::Trait => "trait",
            Keyword::Impl => "impl",
            Keyword::Class => "class",
            Keyword::Type => "type",
            Keyword::Module => "module",
            Keyword::Namespace => "namespace",
            Keyword::Import => "import",
            Keyword::Using => "using",
            Keyword::Public => "public",
            Keyword::Private => "private",
            Keyword::Static => "static",
            Keyword::Unsafe => "unsafe",
            Keyword::True => "true",
            Keyword::False => "false",
            Keyword::SelfValue => "self",
            Keyword::SelfType => "Self",
            Keyword::Move => "move",
            Keyword::Mut => "mut",
            Keyword::New => "new",
        }
    }

    pub fn from_ident(s: &str) -> Option<Keyword> {
        Self::ALL.iter().copied().find(|k| k.as_str() == s)
    }
}

/// 記号。
///
/// `>>` は字句解析では1つのトークンにしない(`Array<Array<Int>>` のように
/// ジェネリクスの閉じ括弧と衝突するため)。構文解析側で、`Gt` が2つ続き、
/// かつ最初の `Token::joint` が true のときに右シフトとして扱う。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Punct {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Colon,
    ColonColon,
    Dot,
    DotDot,
    Question,
    Tilde,
    Eq,
    EqEq,
    FatArrow,
    Arrow,
    Plus,
    PlusEq,
    Minus,
    MinusEq,
    Star,
    StarEq,
    Slash,
    SlashEq,
    Percent,
    PercentEq,
    Bang,
    NotEq,
    Lt,
    LtEq,
    Shl,
    Gt,
    GtEq,
    Amp,
    AndAnd,
    AmpEq,
    Pipe,
    OrOr,
    PipeEq,
    Caret,
    CaretEq,
}

impl Punct {
    pub fn as_str(self) -> &'static str {
        match self {
            Punct::LParen => "(",
            Punct::RParen => ")",
            Punct::LBrace => "{",
            Punct::RBrace => "}",
            Punct::LBracket => "[",
            Punct::RBracket => "]",
            Punct::Comma => ",",
            Punct::Semi => ";",
            Punct::Colon => ":",
            Punct::ColonColon => "::",
            Punct::Dot => ".",
            Punct::DotDot => "..",
            Punct::Question => "?",
            Punct::Tilde => "~",
            Punct::Eq => "=",
            Punct::EqEq => "==",
            Punct::FatArrow => "=>",
            Punct::Arrow => "->",
            Punct::Plus => "+",
            Punct::PlusEq => "+=",
            Punct::Minus => "-",
            Punct::MinusEq => "-=",
            Punct::Star => "*",
            Punct::StarEq => "*=",
            Punct::Slash => "/",
            Punct::SlashEq => "/=",
            Punct::Percent => "%",
            Punct::PercentEq => "%=",
            Punct::Bang => "!",
            Punct::NotEq => "!=",
            Punct::Lt => "<",
            Punct::LtEq => "<=",
            Punct::Shl => "<<",
            Punct::Gt => ">",
            Punct::GtEq => ">=",
            Punct::Amp => "&",
            Punct::AndAnd => "&&",
            Punct::AmpEq => "&=",
            Punct::Pipe => "|",
            Punct::OrOr => "||",
            Punct::PipeEq => "|=",
            Punct::Caret => "^",
            Punct::CaretEq => "^=",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Keyword(Keyword),
    /// 整数リテラル。`_` を取り除いた生の文字列(`0xFF` などの接頭辞は残す)。
    Int(String),
    /// 浮動小数リテラル。`_` を取り除いた生の文字列。
    Float(String),
    /// 文字列リテラル。エスケープは展開済み。
    Str(String),
    /// テンプレート文字列 `` `x = ${x}` ``。バッククォートの内側の生の文字列。
    /// `${ }` の中身の解析は構文解析側で行う。
    Template(String),
    Char(char),
    /// ライフタイム `'a`(先頭の `'` は含まない)。
    Lifetime(String),
    Punct(Punct),
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Ident(s) => write!(f, "ident {s}"),
            TokenKind::Keyword(k) => write!(f, "keyword {}", k.as_str()),
            TokenKind::Int(s) => write!(f, "int {s}"),
            TokenKind::Float(s) => write!(f, "float {s}"),
            TokenKind::Str(s) => write!(f, "string {s:?}"),
            TokenKind::Template(s) => write!(f, "template {s:?}"),
            TokenKind::Char(c) => write!(f, "char {c:?}"),
            TokenKind::Lifetime(s) => write!(f, "lifetime '{s}"),
            TokenKind::Punct(p) => write!(f, "punct {}", p.as_str()),
            TokenKind::Eof => write!(f, "eof"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    /// 直後に空白なしで次の文字が続くかどうか(`>>` の判定などに使う)。
    pub joint: bool,
}