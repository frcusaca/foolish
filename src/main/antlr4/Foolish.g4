grammar Foolish;

program : branes EOF ;

characterizable
    : (IDENTIFIER? APOSTROPHE)? (literal | IDENTIFIER | brane)
    ;

brane : LBRACE stmt* RBRACE ;
branes: brane+ ;

stmt
    : (
      branes
    | assignment
    | expr
    ) SEMI LINE_COMMENT?
    ;
assignment : IDENTIFIER ASSIGN expr ;

expr
    : addExpr
    | ifExpr
    ;

addExpr : mulExpr ((PLUS | MINUS) mulExpr)* ;


mulExpr
    : expExpr (( MUL | DIV) expExpr)*
    ;

expExpr
    : unaryExpr <assoc=right> EXP expExpr
    | unaryExpr
    ;

unaryExpr
    : (PLUS|MINUS|MUL)? primary
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
    : ifExprHelperIf (ifExprHelperElif)* (ifExprHelperElse)?
    ;
ifExprHelperIf: IF  expr THEN expr ;
ifExprHelperElif: ELIF expr THEN expr ;
ifExprHelperElse: ELSE expr ;

// Lexer rules (uppercase)
LBRACE : '{' ;
RBRACE : '}' ;
LPAREN : '(' ;
RPAREN : ')' ;
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
EXP : '^' ;

IF  : 'if' ;
THEN : 'then' ;
ELIF : 'elif' ;
ELSE : 'else' ;

UNKNOWN : '???' ; // Unknowns are unknown

INTEGER : DIGIT+ ;

fragment LETTER : [a-zA-Z] ;
fragment DIGIT : [0-9] ;
fragment UNDERSCORE : '_' ;

// Skip whitespace
WS : [ \t\r\n]+ -> skip ;

IDENTIFIER : LETTER (LETTER|DIGIT|UNDERSCORE)* ;

APOSTROPHE : '\'' ;
