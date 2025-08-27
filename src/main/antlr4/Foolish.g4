grammar Foolish;

program : brane EOF ;

brane : LBRACE stmt* RBRACE ;

stmt : (assignment | expr) SEMI ;

assignment : IDENTIFIER ASSIGN expr ;

expr : addExpr ;

addExpr : mulExpr ((PLUS | MINUS) mulExpr)* ;

mulExpr
    : unaryExpr ((MUL | DIV) unaryExpr)*
    ;

unaryOp
    : MUL
    ;

unaryExpr
    : unaryOp unaryExpr
    | primary
    ;

primary : INTEGER
        | IDENTIFIER
        | LPAREN expr RPAREN 
	;

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

INTEGER : SIGN? DIGIT+ ;

fragment LETTER : [a-zA-Z] ;
fragment DIGIT : [0-9] ;
fragment UNDERSCORE : '_' ;
fragment SIGN : [+-] ;

// Skip whitespace
WS : [ \t\r\n]+ -> skip ;

IDENTIFIER : LETTER (LETTER|DIGIT|UNDERSCORE)* ;
