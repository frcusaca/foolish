grammar Foolish;

program : branes EOF ;

// A characterization is an optional identifier followed by an apostrophe
characterization
    : IDENTIFIER? APOSTROPHE
    ;

// Characterizable identifiers are characterized identifiers
characterizable_identifier
    : characterization* IDENTIFIER
    ;

// Characterizables can be identifiers, literals, or branes
characterizable
    : characterizable_identifier
    | characterization* (literal | brane)
    ;

branes: brane+ ;
brane
    : standard_brane
    | detach_brane
    | brane_search
    ;

standard_brane
    : LBRACE stmt* RBRACE
    ;

detach_brane
    : LBRACK detach_stmt* RBRACK
    ;

detach_stmt
    : characterizable_identifier (ASSIGN expr)? SEMI LINE_COMMENT?
    ;

brane_search : UP;

stmt
    : (
      assignment
    | expr
    ) SEMI LINE_COMMENT?
    ;
assignment : characterizable_identifier ASSIGN expr ;

expr
    : addExpr
    | ifExpr
    | branes
    ;

addExpr : mulExpr ((PLUS | MINUS) mulExpr)* ;

mulExpr
    : unaryExpr ((MUL | DIV) unaryExpr)*
    ;

unaryExpr
    : (PLUS|MINUS|MUL)? postfixExpr
    ;

postfixExpr
    : primary (regexp_operator regexp_expression)*
    ;

literal
    : INTEGER
    ;

primary
    : characterizable
    | LPAREN expr RPAREN
    | UNKNOWN
	;

ifExpr
    : ifExprHelperIf (ifExprHelperElif)* (ifExprHelperElse)? endIf?
    ;
ifExprHelperIf: IF  expr THEN expr ;
ifExprHelperElif: ELIF expr THEN expr ;
ifExprHelperElse: ELSE expr ;
endIf: FI ;

// Regexp operators for brane searching
regexp_operator
    : DOT_DOT        // '..'
    | DOT            // '.'
    ;

// Regexp expression with balanced parentheses validation
// The pattern is stored as a string (via getText()) but ANTLR enforces matching pairs
// Note: Must use tokens (IDENTIFIER, INTEGER) not fragments (LETTERS, DIGIT, INTRA_ID_SEPARATOR)
// since fragments cannot be referenced in parser rules
regexp_expression : regexp_element+ ;

regexp_element
    : REGEXP_SYMBOLS+
    | LPAREN regexp_element* RPAREN    // Balanced ()
    | LBRACE regexp_element* RBRACE    // Balanced {}
    | LBRACK regexp_element* RBRACK    // Balanced []
    ;

// Lexer rules (uppercase)
LBRACE : '{' ;
RBRACE : '}' ;
LPAREN : '(' ;
RPAREN : ')' ;
LBRACK : '[' ;
RBRACK : ']' ;
SEMI : ';' ;

LINE_COMMENT
    : '!' ~[\r\n]* -> skip
    ;
BLOCK_COMMENT
    : '!!' (.| '\r' | '\n' )*? '!!' -> skip
    ;


ASSIGN : '=' ;
PLUS : '+' ;
MINUS : '-' ;
MUL : '*' ;
DIV : '/' ;
CARET : '^';
ESLASH : '\\';
DOLLAR : '$';
QUESTION: '?';

DOT_DOT : '..' ;
DOT : '.' ;

IF  : 'if' ;
THEN : 'then' ;
ELIF : 'elif' ;
ELSE : 'else' ;
FI   :  'fi'  ;
UP   : '↑' ;
UNKNOWN : '???' ; // Unknowns are unknown

INTEGER : DIGIT+ ;

fragment LETTERS : ARABIC_PART | LATIN | GREEK_PART | CYRYLLIC_PART | HEBREW_PART | CHINESE_PART;
fragment DIGIT : [0-9] ;
fragment INTRA_ID_SEPARATOR : ' ' | '⁠' | '_' ;
// Skip whitespace
WS : [ \t\r\n]+ -> skip ;

IDENTIFIER : LETTERS (LETTERS|DIGIT|INTRA_ID_SEPARATOR)* ;
REGEXP_SYMBOLS : REGEXP_ID_CHARS|REGEXP_OPS;
fragment REGEXP_ID_CHARS:LETTERS|DIGIT|INTRA_ID_SEPARATOR;
fragment REGEXP_OPS: MUL|PLUS|CARET|APOSTROPHE|QUESTION|DOLLAR;

APOSTROPHE : '\'' ;
fragment LATIN  : [a-zA-Z]+;
fragment GREEK_PART: [αβΓγΔδεζηΘθΙικΛλμνΞξΟοΠπρΣσςτυΦφΨψΩω]+; // Greek letters that do not confuse with latin easily
fragment CYRYLLIC_PART: [БбвДдЖжЗзИиЙйКкЛлмнПптЦцЧчЩщЪъЫыЭэЮюЯя]+;
fragment HEBREW_PART : [אבגהחטכלמנסעפצקרשתםףץך]+;
fragment CHINESE_PART : ('\u4E00'..'\u9FFF')+;
fragment ARABIC_PART : ('\u0621'..'\u0652')+;
fragment SANSKRIT_PART 
    :   (   DEVANAGARI_CHAR
        |   VEDIC_EXT_CHAR
        |   DEVANAGARI_EXT_CHAR
        )+
    ;

// Fragment rules for specific Unicode blocks (these are not tokens themselves, but building blocks)
fragment DEVANAGARI_CHAR
    :   '\u0900'..'\u097F' // Devanagari block
    ;

fragment VEDIC_EXT_CHAR
    :   '\u1CD0'..'\u1CFF' // Vedic Extensions block
    ;

// The Devanagari Extended block may contain additional relevant characters
fragment DEVANAGARI_EXT_CHAR
    :   '\uA8E0'..'\uA8FF' // Devanagari Extended block
    ;
