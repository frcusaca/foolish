grammar Foolish;

program : branes EOF ;

brane : LBRACE stmt* RBRACE ;
branes: brane+ ;

stmt
    : branes SEMI
    | assignment SEMI
    | expr SEMI
    ;

assignment : IDENTIFIER ASSIGN expr ;

expr
    : addExpr
    | ifExpr
    ;

addExpr : mulExpr ((PLUS | MINUS) mulExpr)* ;

mulExpr
    : unaryExpr ((MUL | DIV) unaryExpr)*
    ;

unaryExpr
    : (PLUS|MINUS|MUL)? primary
    ;

primary : INTEGER
        | IDENTIFIER
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
ASSIGN : '=' ;
PLUS : '+' ;
MINUS : '-' ;
MUL : '*' ;
DIV : '/' ;

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
