pub enum TokenTypes {
    // Keywords
    VAR,
    FUN,
    FOR,
    WHILE,
    IF,
    ELSE,

    // Special character
    LBRAC,       // {
    RBRAC,       // }
    LPARENT,     // (
    RPARENT,     // )
    LSQUAREBRAC, // [
    RSQUAREBRAC, // ]
    COMMENT,     // //
    QUOTMARKS,   // ""
    COMMA,       // ,
    COLON,       // :

    // Identifier, literals
    IDENT,
    INTEGER,
    STRING,
    FALSE,
    TRUE,

    // Other
    ILLEGAL, // illeagl expression
    EOL,     // End of line

    // Arithmetic operators and alike
    PLUS,        // +
    MINUS,       // -
    DIVIDE,      // /
    MULTIPLY,    // *
    ASSIGN,      // =
    EQUAL,       // ==
    GREATERTHAN, // >
    LESSERTHAN,    // <
    GREATEROREQUALTHAN, // >=
    LESSEROREQUALTHAN,    // <=

    // Structures
    STRUCT,
    ENUM,
}

impl TokenTypes {
    pub fn literal(&self) -> &str {
        match self {
            TokenTypes::VAR => {"var"}
            TokenTypes::FUN => {"fun"}
            TokenTypes::FOR => {"for"}
            TokenTypes::WHILE => {"while"}
            TokenTypes::IF => {"if"}
            TokenTypes::ELSE => {"else"}

            TokenTypes::IDENT => {"IDENT"}
            TokenTypes::INTEGER => {"INT"}
            TokenTypes::STRING => {"STRING"}
            TokenTypes::TRUE => {"true"}
            TokenTypes::FALSE => {"false"}

            TokenTypes::EOL => {"EOL"}
            TokenTypes::ILLEGAL => {"ILLEGAL"}

            TokenTypes::ASSIGN => {"="}
            TokenTypes::PLUS => {"+"}
            TokenTypes::MINUS => {"-"}
            TokenTypes::MULTIPLY => {"*"}
            TokenTypes::DIVIDE => {"/"}
            
            TokenTypes::EQUAL => {"=="}
            TokenTypes::GREATERTHAN => {">"}
            TokenTypes::LESSERTHAN => {"<"}
            TokenTypes::GREATEROREQUALTHAN => {">="}
            TokenTypes::LESSEROREQUALTHAN => {"<="}

            TokenTypes::COMMENT => {"//"}
            
            TokenTypes::COLON => {":"}
            TokenTypes::COMMA => {","}
            TokenTypes::LBRAC => {"{"}
            TokenTypes::RBRAC => {"}"}
            TokenTypes::LPARENT => {"("}
            TokenTypes::RPARENT => {")"}
            TokenTypes::LSQUAREBRAC => {"["}
            TokenTypes::RSQUAREBRAC => {"]"}
            TokenTypes::QUOTMARKS => {"\""}

            TokenTypes::STRUCT => {"struct"}
            TokenTypes::ENUM => {"enum"}
        }
    }
}

pub struct Token(pub TokenTypes, pub &'static str);
