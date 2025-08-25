// Generated from /home/user/foolish/src/main/antlr4/Foolish.g4 by ANTLR 4.13.1
import org.antlr.v4.runtime.tree.ParseTreeListener;

/**
 * This interface defines a complete listener for a parse tree produced by
 * {@link FoolishParser}.
 */
public interface FoolishListener extends ParseTreeListener {
	/**
	 * Enter a parse tree produced by {@link FoolishParser#program}.
	 * @param ctx the parse tree
	 */
	void enterProgram(FoolishParser.ProgramContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#program}.
	 * @param ctx the parse tree
	 */
	void exitProgram(FoolishParser.ProgramContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#brane}.
	 * @param ctx the parse tree
	 */
	void enterBrane(FoolishParser.BraneContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#brane}.
	 * @param ctx the parse tree
	 */
	void exitBrane(FoolishParser.BraneContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#expression}.
	 * @param ctx the parse tree
	 */
	void enterExpression(FoolishParser.ExpressionContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#expression}.
	 * @param ctx the parse tree
	 */
	void exitExpression(FoolishParser.ExpressionContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#logicalOrExpr}.
	 * @param ctx the parse tree
	 */
	void enterLogicalOrExpr(FoolishParser.LogicalOrExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#logicalOrExpr}.
	 * @param ctx the parse tree
	 */
	void exitLogicalOrExpr(FoolishParser.LogicalOrExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#logicalAndExpr}.
	 * @param ctx the parse tree
	 */
	void enterLogicalAndExpr(FoolishParser.LogicalAndExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#logicalAndExpr}.
	 * @param ctx the parse tree
	 */
	void exitLogicalAndExpr(FoolishParser.LogicalAndExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#equalityExpr}.
	 * @param ctx the parse tree
	 */
	void enterEqualityExpr(FoolishParser.EqualityExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#equalityExpr}.
	 * @param ctx the parse tree
	 */
	void exitEqualityExpr(FoolishParser.EqualityExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#relationalExpr}.
	 * @param ctx the parse tree
	 */
	void enterRelationalExpr(FoolishParser.RelationalExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#relationalExpr}.
	 * @param ctx the parse tree
	 */
	void exitRelationalExpr(FoolishParser.RelationalExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#addExpr}.
	 * @param ctx the parse tree
	 */
	void enterAddExpr(FoolishParser.AddExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#addExpr}.
	 * @param ctx the parse tree
	 */
	void exitAddExpr(FoolishParser.AddExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#mulExpr}.
	 * @param ctx the parse tree
	 */
	void enterMulExpr(FoolishParser.MulExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#mulExpr}.
	 * @param ctx the parse tree
	 */
	void exitMulExpr(FoolishParser.MulExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#concatExpr}.
	 * @param ctx the parse tree
	 */
	void enterConcatExpr(FoolishParser.ConcatExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#concatExpr}.
	 * @param ctx the parse tree
	 */
	void exitConcatExpr(FoolishParser.ConcatExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#postfixExpr}.
	 * @param ctx the parse tree
	 */
	void enterPostfixExpr(FoolishParser.PostfixExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#postfixExpr}.
	 * @param ctx the parse tree
	 */
	void exitPostfixExpr(FoolishParser.PostfixExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#pathOp}.
	 * @param ctx the parse tree
	 */
	void enterPathOp(FoolishParser.PathOpContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#pathOp}.
	 * @param ctx the parse tree
	 */
	void exitPathOp(FoolishParser.PathOpContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#primaryExpr}.
	 * @param ctx the parse tree
	 */
	void enterPrimaryExpr(FoolishParser.PrimaryExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#primaryExpr}.
	 * @param ctx the parse tree
	 */
	void exitPrimaryExpr(FoolishParser.PrimaryExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#funcExpr}.
	 * @param ctx the parse tree
	 */
	void enterFuncExpr(FoolishParser.FuncExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#funcExpr}.
	 * @param ctx the parse tree
	 */
	void exitFuncExpr(FoolishParser.FuncExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#paramList}.
	 * @param ctx the parse tree
	 */
	void enterParamList(FoolishParser.ParamListContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#paramList}.
	 * @param ctx the parse tree
	 */
	void exitParamList(FoolishParser.ParamListContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#param}.
	 * @param ctx the parse tree
	 */
	void enterParam(FoolishParser.ParamContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#param}.
	 * @param ctx the parse tree
	 */
	void exitParam(FoolishParser.ParamContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#braneExpr}.
	 * @param ctx the parse tree
	 */
	void enterBraneExpr(FoolishParser.BraneExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#braneExpr}.
	 * @param ctx the parse tree
	 */
	void exitBraneExpr(FoolishParser.BraneExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#braneStmt}.
	 * @param ctx the parse tree
	 */
	void enterBraneStmt(FoolishParser.BraneStmtContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#braneStmt}.
	 * @param ctx the parse tree
	 */
	void exitBraneStmt(FoolishParser.BraneStmtContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#assignmentExpression}.
	 * @param ctx the parse tree
	 */
	void enterAssignmentExpression(FoolishParser.AssignmentExpressionContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#assignmentExpression}.
	 * @param ctx the parse tree
	 */
	void exitAssignmentExpression(FoolishParser.AssignmentExpressionContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#assignExpr}.
	 * @param ctx the parse tree
	 */
	void enterAssignExpr(FoolishParser.AssignExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#assignExpr}.
	 * @param ctx the parse tree
	 */
	void exitAssignExpr(FoolishParser.AssignExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#typeAssignExpr}.
	 * @param ctx the parse tree
	 */
	void enterTypeAssignExpr(FoolishParser.TypeAssignExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#typeAssignExpr}.
	 * @param ctx the parse tree
	 */
	void exitTypeAssignExpr(FoolishParser.TypeAssignExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#typeExpression}.
	 * @param ctx the parse tree
	 */
	void enterTypeExpression(FoolishParser.TypeExpressionContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#typeExpression}.
	 * @param ctx the parse tree
	 */
	void exitTypeExpression(FoolishParser.TypeExpressionContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#derefAssignExpr}.
	 * @param ctx the parse tree
	 */
	void enterDerefAssignExpr(FoolishParser.DerefAssignExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#derefAssignExpr}.
	 * @param ctx the parse tree
	 */
	void exitDerefAssignExpr(FoolishParser.DerefAssignExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#braneTypeDef}.
	 * @param ctx the parse tree
	 */
	void enterBraneTypeDef(FoolishParser.BraneTypeDefContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#braneTypeDef}.
	 * @param ctx the parse tree
	 */
	void exitBraneTypeDef(FoolishParser.BraneTypeDefContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#fieldDef}.
	 * @param ctx the parse tree
	 */
	void enterFieldDef(FoolishParser.FieldDefContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#fieldDef}.
	 * @param ctx the parse tree
	 */
	void exitFieldDef(FoolishParser.FieldDefContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#primitiveType}.
	 * @param ctx the parse tree
	 */
	void enterPrimitiveType(FoolishParser.PrimitiveTypeContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#primitiveType}.
	 * @param ctx the parse tree
	 */
	void exitPrimitiveType(FoolishParser.PrimitiveTypeContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#typeIdentifier}.
	 * @param ctx the parse tree
	 */
	void enterTypeIdentifier(FoolishParser.TypeIdentifierContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#typeIdentifier}.
	 * @param ctx the parse tree
	 */
	void exitTypeIdentifier(FoolishParser.TypeIdentifierContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#braneIndex}.
	 * @param ctx the parse tree
	 */
	void enterBraneIndex(FoolishParser.BraneIndexContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#braneIndex}.
	 * @param ctx the parse tree
	 */
	void exitBraneIndex(FoolishParser.BraneIndexContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#literal}.
	 * @param ctx the parse tree
	 */
	void enterLiteral(FoolishParser.LiteralContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#literal}.
	 * @param ctx the parse tree
	 */
	void exitLiteral(FoolishParser.LiteralContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#primitiveLiteral}.
	 * @param ctx the parse tree
	 */
	void enterPrimitiveLiteral(FoolishParser.PrimitiveLiteralContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#primitiveLiteral}.
	 * @param ctx the parse tree
	 */
	void exitPrimitiveLiteral(FoolishParser.PrimitiveLiteralContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#prefixSign}.
	 * @param ctx the parse tree
	 */
	void enterPrefixSign(FoolishParser.PrefixSignContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#prefixSign}.
	 * @param ctx the parse tree
	 */
	void exitPrefixSign(FoolishParser.PrefixSignContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#intLiteral}.
	 * @param ctx the parse tree
	 */
	void enterIntLiteral(FoolishParser.IntLiteralContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#intLiteral}.
	 * @param ctx the parse tree
	 */
	void exitIntLiteral(FoolishParser.IntLiteralContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#floatLiteral}.
	 * @param ctx the parse tree
	 */
	void enterFloatLiteral(FoolishParser.FloatLiteralContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#floatLiteral}.
	 * @param ctx the parse tree
	 */
	void exitFloatLiteral(FoolishParser.FloatLiteralContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#stringLiteral}.
	 * @param ctx the parse tree
	 */
	void enterStringLiteral(FoolishParser.StringLiteralContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#stringLiteral}.
	 * @param ctx the parse tree
	 */
	void exitStringLiteral(FoolishParser.StringLiteralContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#typeLiteral}.
	 * @param ctx the parse tree
	 */
	void enterTypeLiteral(FoolishParser.TypeLiteralContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#typeLiteral}.
	 * @param ctx the parse tree
	 */
	void exitTypeLiteral(FoolishParser.TypeLiteralContext ctx);
	/**
	 * Enter a parse tree produced by the {@code TypeIdent}
	 * labeled alternative in {@link FoolishParser#identifier}.
	 * @param ctx the parse tree
	 */
	void enterTypeIdent(FoolishParser.TypeIdentContext ctx);
	/**
	 * Exit a parse tree produced by the {@code TypeIdent}
	 * labeled alternative in {@link FoolishParser#identifier}.
	 * @param ctx the parse tree
	 */
	void exitTypeIdent(FoolishParser.TypeIdentContext ctx);
	/**
	 * Enter a parse tree produced by the {@code OrdIdent}
	 * labeled alternative in {@link FoolishParser#identifier}.
	 * @param ctx the parse tree
	 */
	void enterOrdIdent(FoolishParser.OrdIdentContext ctx);
	/**
	 * Exit a parse tree produced by the {@code OrdIdent}
	 * labeled alternative in {@link FoolishParser#identifier}.
	 * @param ctx the parse tree
	 */
	void exitOrdIdent(FoolishParser.OrdIdentContext ctx);
}