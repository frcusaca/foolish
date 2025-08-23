
grammar Foolish;

program : brane EOF ;
brane   : braneExpr ;

expression      : logicalOrExpr ;
logicalOrExpr   : logicalAndExpr ( OR OR logicalAndExpr )* ;
logicalAndExpr  : equalityExpr ( AND AND equalityExpr )* ;
equalityExpr    : relationalExpr ( EQEQ relationalExpr )* ;
relationalExpr  : addExpr ( (LT | GT | LE | GE) addExpr )* ;
addExpr         : mulExpr ( (PLUS | MINUS) mulExpr )* ;
mulExpr         : concatExpr ( (STAR | SLASH) concatExpr )* ;

// RPN concatenation by adjacency, left-to-right
concatExpr      : postfixExpr (postfixExpr)* ;

postfixExpr     : primaryExpr pathOp* ;
pathOp          : CARET braneIndex?
                | DOLLAR braneIndex?
                | HASH braneIndex
                ;

primaryExpr     : literal
                | identifier
                | funcExpr
                | braneExpr
                | LPAREN expression RPAREN
                ;

funcExpr        : LPAREN paramList? RPAREN ARROW brane ;
paramList       : param ( COMMA param )* ;
param           : identifier COLON ( type_ | typeIdentifier ) ;

braneExpr       : LBRACE braneStmt* RBRACE ;
braneStmt       : ( expression | assignmentExpression ) ( SEMI | NEWLINE )* ;

assignmentExpression : assignExpr | typeAssignExpr | derefAssignExpr ;
assignExpr      : identifier ASSIGN expression ;
typeAssignExpr  : typeIdentifier ASSIGN typeExpression ;
typeExpression  : type_ | typeIdentifier ;
derefAssignExpr : identifier ASSIGN_DEREF expression ;

braneTypeDef    : LDBLBRACE ( fieldDef ( COMMA fieldDef )* )? RDBLBRACE ;
fieldDef        : identifier COLON ( type_ | typeIdentifier ) ;

primitiveType   : INT_T | FLOAT_T | STRING_T | BRANE_T | braneTypeDef ;
type_           : TYPE_KIND PRIM_QUOTE primitiveType ;

braneIndex      : identifier | intLiteral ;

literal         : primitiveLiteral | typeLiteral ;
primitiveLiteral: intLiteral | floatLiteral | stringLiteral ;
intLiteral      : MINUS? DIGITS ;
floatLiteral    : DIGITS? DOT DIGITS ;
stringLiteral   : DQUOTE (~DQUOTE)* DQUOTE ;
typeLiteral     : TYPE_KIND PRIM_QUOTE primitiveLiteral ;

identifier
    : TYPE_IDENTIFIER     #TypeIdent
    | ORD_IDENTIFIER      #OrdIdent
    ;

// Tokens
INT_T      : 'Int' ;
FLOAT_T    : 'Float' ;
STRING_T   : 'String' ;
BRANE_T    : 'Brane' ;

ARROW      : '->' ;
ASSIGN     : '=' ;
ASSIGN_DEREF : '=$' ;
EQEQ       : '==' ;
LE         : '<=' ;
GE         : '>=' ;
LT         : '<' ;
GT         : '>' ;
PLUS       : '+' ;
MINUS      : '-' ;
STAR       : '*' ;
SLASH      : '/' ;
HASH       : '#' ;
CARET      : '^' ;
DOLLAR     : '$' ;
OR         : '|' ;
AND        : '&' ;
LPAREN     : '(' ;
RPAREN     : ')' ;
LBRACE     : '{' ;
RBRACE     : '}' ;
LDBLBRACE  : '{{' ;
RDBLBRACE  : '}}' ;
COMMA      : ',' ;
COLON      : ':' ;
SEMI       : ';' ;
DOT        : '.' ;
DQUOTE     : '"' ;
PRIM_QUOTE : '\'' ;
TYPE_KIND  : [Tt] ;

// Order matters: TYPE_IDENTIFIER before ORD_IDENTIFIER
TYPE_IDENTIFIER : [Tt] '\'' [A-Za-z_][A-Za-z0-9_]* ;
ORD_IDENTIFIER  : '\''? [A-Za-z_][A-Za-z0-9_]* ;

fragment DIGITS : [0-9]+ ;

// Comments are ignored by the parser
LINE_COMMENT : '--' ~[\r\n]* -> channel(HIDDEN) ;
BLOCK_COMMENT: '---' ( ~'-' | '-' ~'-' | '--' ~'-' )* '---' -> channel(HIDDEN) ;

WS      : [ \t\f\r]+ -> channel(HIDDEN) ;
NEWLINE : '\r'? '\n' ;
