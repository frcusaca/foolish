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
	 * Enter a parse tree produced by {@link FoolishParser#characterizable}.
	 * @param ctx the parse tree
	 */
	void enterCharacterizable(FoolishParser.CharacterizableContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#characterizable}.
	 * @param ctx the parse tree
	 */
	void exitCharacterizable(FoolishParser.CharacterizableContext ctx);
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
	 * Enter a parse tree produced by {@link FoolishParser#branes}.
	 * @param ctx the parse tree
	 */
	void enterBranes(FoolishParser.BranesContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#branes}.
	 * @param ctx the parse tree
	 */
	void exitBranes(FoolishParser.BranesContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#stmt}.
	 * @param ctx the parse tree
	 */
	void enterStmt(FoolishParser.StmtContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#stmt}.
	 * @param ctx the parse tree
	 */
	void exitStmt(FoolishParser.StmtContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#assignment}.
	 * @param ctx the parse tree
	 */
	void enterAssignment(FoolishParser.AssignmentContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#assignment}.
	 * @param ctx the parse tree
	 */
	void exitAssignment(FoolishParser.AssignmentContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#expr}.
	 * @param ctx the parse tree
	 */
	void enterExpr(FoolishParser.ExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#expr}.
	 * @param ctx the parse tree
	 */
	void exitExpr(FoolishParser.ExprContext ctx);
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
	 * Enter a parse tree produced by {@link FoolishParser#unaryExpr}.
	 * @param ctx the parse tree
	 */
	void enterUnaryExpr(FoolishParser.UnaryExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#unaryExpr}.
	 * @param ctx the parse tree
	 */
	void exitUnaryExpr(FoolishParser.UnaryExprContext ctx);
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
	 * Enter a parse tree produced by {@link FoolishParser#primary}.
	 * @param ctx the parse tree
	 */
	void enterPrimary(FoolishParser.PrimaryContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#primary}.
	 * @param ctx the parse tree
	 */
	void exitPrimary(FoolishParser.PrimaryContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#ifExpr}.
	 * @param ctx the parse tree
	 */
	void enterIfExpr(FoolishParser.IfExprContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#ifExpr}.
	 * @param ctx the parse tree
	 */
	void exitIfExpr(FoolishParser.IfExprContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#ifExprHelperIf}.
	 * @param ctx the parse tree
	 */
	void enterIfExprHelperIf(FoolishParser.IfExprHelperIfContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#ifExprHelperIf}.
	 * @param ctx the parse tree
	 */
	void exitIfExprHelperIf(FoolishParser.IfExprHelperIfContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#ifExprHelperElif}.
	 * @param ctx the parse tree
	 */
	void enterIfExprHelperElif(FoolishParser.IfExprHelperElifContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#ifExprHelperElif}.
	 * @param ctx the parse tree
	 */
	void exitIfExprHelperElif(FoolishParser.IfExprHelperElifContext ctx);
	/**
	 * Enter a parse tree produced by {@link FoolishParser#ifExprHelperElse}.
	 * @param ctx the parse tree
	 */
	void enterIfExprHelperElse(FoolishParser.IfExprHelperElseContext ctx);
	/**
	 * Exit a parse tree produced by {@link FoolishParser#ifExprHelperElse}.
	 * @param ctx the parse tree
	 */
	void exitIfExprHelperElse(FoolishParser.IfExprHelperElseContext ctx);
}