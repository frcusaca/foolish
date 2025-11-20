grammar Foolish;

program : branes EOF ;

characterizable
    : characterizable_identifier
    | (IDENTIFIER? APOSTROPHE)? (literal | brane)
    ;

characterizable_identifier
    : (IDENTIFIER? APOSTROPHE)? IDENTIFIER
    ;

branes: brane+ ;
brane
    : standard_brane
    | detach_brane
    | brane_search;

standard_brane
    : LBRACE stmt* RBRACE
    ;

detach_brane
    : LBRACK detach_stmt* RBRACK
    ;

detach_stmt
    : characterizable_identifier (ASSIGN expr)? SEMI LINE_COMMENT?
    ;

brane_search :  UP;

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
    : primary (DOT characterizable_identifier)*
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
