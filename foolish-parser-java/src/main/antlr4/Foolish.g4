grammar Foolish;

@parser::members {
    private boolean areTokensAdjacent() {
        Token prev = _input.LT(-1);
        Token curr = _input.LT(1);
        // Check if there are any hidden tokens (like whitespace) between prev and curr
        int indexDiff = curr.getTokenIndex() - prev.getTokenIndex();
        return indexDiff == 1;
    }
}

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

branes: concatExpr ;
brane
    : standard_brane
    | detach_brane
    | brane_search
    ;

standard_brane
    : LBRACE stmt* stmt_body? RBRACE
    ;

detach_brane
    : LBRACK detach_stmt_list? RBRACK
    ;

detach_stmt_list
    : detach_stmt ((SEMI | COMMA) LINE_COMMENT? detach_stmt)* ((SEMI | COMMA) LINE_COMMENT?)?
    ;

detach_stmt
    : characterizable_identifier (ASSIGN expr)?
    ;

brane_search : UP;

stmt_body
    : assignment
    | expr
    ;

stmt
    : stmt_body ((SEMI | COMMA) LINE_COMMENT? | LINE_COMMENT)
    | LINE_COMMENT
    ;
assignment
    : characterizable_identifier LT_LT_EQ_GT_GT expr    // <<=>>> SFF assignment
    | characterizable_identifier LT_EQ_GT expr           // <=> SF assignment (constanic)
    | characterizable_identifier ASSIGN {areTokensAdjacent()}? DOLLAR expr
    | characterizable_identifier ASSIGN {areTokensAdjacent()}? CARET expr
    | characterizable_identifier ASSIGN expr
    ;

expr
    : addExpr
    | ifExpr
    ;

addExpr : mulExpr ((PLUS | MINUS) mulExpr)* ;

mulExpr
    : unaryExpr ((MUL | DIV) unaryExpr)*
    ;

unaryExpr
    : (PLUS|MINUS|MUL)? concatExpr
    ;

concatExpr
    : postfixExpr+
    ;

postfixExpr
    : primary (postfix_op)*
    ;

postfix_op
    : DOT characterizable_identifier
    | regexp_operator regexp_expression
    | HASH seek_index
    | CARET
    | DOLLAR
    ;

seek_index
    : MINUS? INTEGER
    ;

literal
    : INTEGER
    ;

primary
    : characterizable
    | LPAREN expr RPAREN
    | LT_LT expr GT_GT     // <<expr>> SFF marker
    | LT expr GT           // <expr> SF marker
    | UNKNOWN
    | HASH MINUS INTEGER  // Unanchored backward seek: #-1, #-2, etc.
    | regexp_operator regexp_expression // Unanchored regex search
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
    : QUESTION          // ? (backward search)
    | QUESTION_QUESTION // ?? (find-all backward - not yet implemented)
    | TILDE             // ~ (forward search)
    | TILDE_TILDE       // ~~ (find-all forward - not yet implemented)
    ;

// Regexp expression with balanced parentheses validation
// The pattern is stored as a string (via getText()) but ANTLR enforces matching pairs
// Note: Must use tokens (IDENTIFIER, INTEGER) not fragments (LETTERS, DIGIT, INTRA_ID_SEPARATOR)
// since fragments cannot be referenced in parser rules
regexp_expression : regexp_element ({areTokensAdjacent()}? regexp_element)* ;

regexp_element
    : IDENTIFIER         // Letters, digits, separators
    | INTEGER            // Numbers
    | APOSTROPHE         // ' (for characterization)
    | MUL                // *
    | PLUS               // +
    | CARET              // ^
    | DOLLAR             // $
    | DOT                // .
    // Special chars allowed ONLY inside parentheses to avoid ambiguity with operators
    | LPAREN regexp_inner* RPAREN    // Balanced () - can contain special chars
    | LBRACE regexp_inner* RBRACE    // Balanced {}
    | LBRACK regexp_inner* RBRACK    // Balanced []
    ;

// Inside parentheses/braces/brackets, we can use regexp special characters
regexp_inner
    : IDENTIFIER
    | INTEGER
    | APOSTROPHE
    | MUL                // * (for regexp, only inside parens)
    | PLUS               // + (for regexp, only inside parens)
    | CARET              // ^ (for regexp, only inside parens)
    | QUESTION           // ? (for regexp, only inside parens)
    | DOLLAR             // $ (for regexp, only inside parens)
    | ESLASH             // \ (for regexp, only inside parens)
    | DOT                // . (for regexp, only inside parens)
    | LPAREN regexp_inner* RPAREN    // Nested balanced ()
    | LBRACE regexp_inner* RBRACE    // Nested balanced {}
    | LBRACK regexp_inner* RBRACK    // Nested balanced []
    ;

// Lexer rules (uppercase)
LBRACE : '{' ;
RBRACE : '}' ;
LPAREN : '(' ;
RPAREN : ')' ;
LBRACK : '[' ;
RBRACK : ']' ;
SEMI : ';' ;
COMMA : ',' ;

BLOCK_COMMENT
    : '!!!' .*? '!!!' -> skip
    ;

LINE_COMMENT
    : '!!' ~[\r\n]*
    ;


// Multi-character operators must come before single-character ones
LT_LT_EQ_GT_GT : '<<=>>' ;  // SFF assignment
LT_EQ_GT : '<=>' ;          // Constanic assignment (SF assignment)
LT_LT : '<<' ;              // SFF marker open
GT_GT : '>>' ;              // SFF marker close
LT : '<' ;                  // SF marker open
GT : '>' ;                  // SF marker close

ASSIGN : '=' ;
PLUS : '+' ;
MINUS : '-' ;
MUL : '*' ;
DIV : '/' ;
CARET : '^';
ESLASH : '\\';
DOLLAR : '$';
QUESTION_QUESTION: '??';
QUESTION: '?';
TILDE_TILDE: '~~';
TILDE: '~';
HASH : '#';

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

// System that accepts _ should convert it to the thinner modifier letter low macro (ux02cd)
// Collation and search systems should accept these as exchangeable.
fragment INTRA_ID_SEPARATOR : '\u202F' | '_' | '\u02CD';
// Skip whitespace
WS : [ \t\r\n]+ -> channel(HIDDEN) ;

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
