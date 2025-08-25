// Generated from /home/user/foolish/src/main/antlr4/Foolish.g4 by ANTLR 4.13.1
import org.antlr.v4.runtime.atn.*;
import org.antlr.v4.runtime.dfa.DFA;
import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.misc.*;
import org.antlr.v4.runtime.tree.*;
import java.util.List;
import java.util.Iterator;
import java.util.ArrayList;

@SuppressWarnings({"all", "warnings", "unchecked", "unused", "cast", "CheckReturnValue"})
public class FoolishParser extends Parser {
	static { RuntimeMetaData.checkVersion("4.13.1", RuntimeMetaData.VERSION); }

	protected static final DFA[] _decisionToDFA;
	protected static final PredictionContextCache _sharedContextCache =
		new PredictionContextCache();
	public static final int
		INT_T=1, FLOAT_T=2, STRING_T=3, BRANE_T=4, ARROW=5, ASSIGN=6, ASSIGN_DEREF=7, 
		EQEQ=8, LE=9, GE=10, LT=11, GT=12, PLUS=13, MINUS=14, STAR=15, SLASH=16, 
		HASH=17, CARET=18, DOLLAR=19, OR=20, AND=21, LPAREN=22, RPAREN=23, LBRACE=24, 
		RBRACE=25, LDBLBRACE=26, RDBLBRACE=27, COMMA=28, COLON=29, SEMI=30, DOT=31, 
		DQUOTE=32, PRIM_QUOTE=33, TYPE_KIND=34, TYPE_IDENTIFIER=35, ORD_IDENTIFIER=36, 
		NUMBER=37, LINE_COMMENT=38, BLOCK_COMMENT=39, WS=40, NEWLINE=41;
	public static final int
		RULE_program = 0, RULE_brane = 1, RULE_expression = 2, RULE_logicalOrExpr = 3, 
		RULE_logicalAndExpr = 4, RULE_equalityExpr = 5, RULE_relationalExpr = 6, 
		RULE_addExpr = 7, RULE_mulExpr = 8, RULE_concatExpr = 9, RULE_postfixExpr = 10, 
		RULE_pathOp = 11, RULE_primaryExpr = 12, RULE_funcExpr = 13, RULE_paramList = 14, 
		RULE_param = 15, RULE_braneExpr = 16, RULE_braneStmt = 17, RULE_assignmentExpression = 18, 
		RULE_assignExpr = 19, RULE_typeAssignExpr = 20, RULE_typeExpression = 21, 
		RULE_derefAssignExpr = 22, RULE_braneTypeDef = 23, RULE_fieldDef = 24, 
		RULE_primitiveType = 25, RULE_typeIdentifier = 26, RULE_braneIndex = 27, 
		RULE_literal = 28, RULE_primitiveLiteral = 29, RULE_prefixSign = 30, RULE_intLiteral = 31, 
		RULE_floatLiteral = 32, RULE_stringLiteral = 33, RULE_typeLiteral = 34, 
		RULE_identifier = 35;
	private static String[] makeRuleNames() {
		return new String[] {
			"program", "brane", "expression", "logicalOrExpr", "logicalAndExpr", 
			"equalityExpr", "relationalExpr", "addExpr", "mulExpr", "concatExpr", 
			"postfixExpr", "pathOp", "primaryExpr", "funcExpr", "paramList", "param", 
			"braneExpr", "braneStmt", "assignmentExpression", "assignExpr", "typeAssignExpr", 
			"typeExpression", "derefAssignExpr", "braneTypeDef", "fieldDef", "primitiveType", 
			"typeIdentifier", "braneIndex", "literal", "primitiveLiteral", "prefixSign", 
			"intLiteral", "floatLiteral", "stringLiteral", "typeLiteral", "identifier"
		};
	}
	public static final String[] ruleNames = makeRuleNames();

	private static String[] makeLiteralNames() {
		return new String[] {
			null, "'Int'", "'Float'", "'String'", "'Brane'", "'->'", "'='", "'=$'", 
			"'=='", "'<='", "'>='", "'<'", "'>'", "'+'", "'-'", "'*'", "'/'", "'#'", 
			"'^'", "'$'", "'|'", "'&'", "'('", "')'", "'{'", "'}'", "'{{'", "'}}'", 
			"','", "':'", "';'", "'.'", "'\"'", "'''"
		};
	}
	private static final String[] _LITERAL_NAMES = makeLiteralNames();
	private static String[] makeSymbolicNames() {
		return new String[] {
			null, "INT_T", "FLOAT_T", "STRING_T", "BRANE_T", "ARROW", "ASSIGN", "ASSIGN_DEREF", 
			"EQEQ", "LE", "GE", "LT", "GT", "PLUS", "MINUS", "STAR", "SLASH", "HASH", 
			"CARET", "DOLLAR", "OR", "AND", "LPAREN", "RPAREN", "LBRACE", "RBRACE", 
			"LDBLBRACE", "RDBLBRACE", "COMMA", "COLON", "SEMI", "DOT", "DQUOTE", 
			"PRIM_QUOTE", "TYPE_KIND", "TYPE_IDENTIFIER", "ORD_IDENTIFIER", "NUMBER", 
			"LINE_COMMENT", "BLOCK_COMMENT", "WS", "NEWLINE"
		};
	}
	private static final String[] _SYMBOLIC_NAMES = makeSymbolicNames();
	public static final Vocabulary VOCABULARY = new VocabularyImpl(_LITERAL_NAMES, _SYMBOLIC_NAMES);

	/**
	 * @deprecated Use {@link #VOCABULARY} instead.
	 */
	@Deprecated
	public static final String[] tokenNames;
	static {
		tokenNames = new String[_SYMBOLIC_NAMES.length];
		for (int i = 0; i < tokenNames.length; i++) {
			tokenNames[i] = VOCABULARY.getLiteralName(i);
			if (tokenNames[i] == null) {
				tokenNames[i] = VOCABULARY.getSymbolicName(i);
			}

			if (tokenNames[i] == null) {
				tokenNames[i] = "<INVALID>";
			}
		}
	}

	@Override
	@Deprecated
	public String[] getTokenNames() {
		return tokenNames;
	}

	@Override

	public Vocabulary getVocabulary() {
		return VOCABULARY;
	}

	@Override
	public String getGrammarFileName() { return "Foolish.g4"; }

	@Override
	public String[] getRuleNames() { return ruleNames; }

	@Override
	public String getSerializedATN() { return _serializedATN; }

	@Override
	public ATN getATN() { return _ATN; }

	public FoolishParser(TokenStream input) {
		super(input);
		_interp = new ParserATNSimulator(this,_ATN,_decisionToDFA,_sharedContextCache);
	}

	@SuppressWarnings("CheckReturnValue")
	public static class ProgramContext extends ParserRuleContext {
		public BraneContext brane() {
			return getRuleContext(BraneContext.class,0);
		}
		public TerminalNode EOF() { return getToken(FoolishParser.EOF, 0); }
		public ProgramContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_program; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterProgram(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitProgram(this);
		}
	}

	public final ProgramContext program() throws RecognitionException {
		ProgramContext _localctx = new ProgramContext(_ctx, getState());
		enterRule(_localctx, 0, RULE_program);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(72);
			brane();
			setState(73);
			match(EOF);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class BraneContext extends ParserRuleContext {
		public BraneExprContext braneExpr() {
			return getRuleContext(BraneExprContext.class,0);
		}
		public BraneContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_brane; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterBrane(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitBrane(this);
		}
	}

	public final BraneContext brane() throws RecognitionException {
		BraneContext _localctx = new BraneContext(_ctx, getState());
		enterRule(_localctx, 2, RULE_brane);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(75);
			braneExpr();
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class ExpressionContext extends ParserRuleContext {
		public LogicalOrExprContext logicalOrExpr() {
			return getRuleContext(LogicalOrExprContext.class,0);
		}
		public ExpressionContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_expression; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterExpression(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitExpression(this);
		}
	}

	public final ExpressionContext expression() throws RecognitionException {
		ExpressionContext _localctx = new ExpressionContext(_ctx, getState());
		enterRule(_localctx, 4, RULE_expression);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(77);
			logicalOrExpr();
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class LogicalOrExprContext extends ParserRuleContext {
		public List<LogicalAndExprContext> logicalAndExpr() {
			return getRuleContexts(LogicalAndExprContext.class);
		}
		public LogicalAndExprContext logicalAndExpr(int i) {
			return getRuleContext(LogicalAndExprContext.class,i);
		}
		public List<TerminalNode> OR() { return getTokens(FoolishParser.OR); }
		public TerminalNode OR(int i) {
			return getToken(FoolishParser.OR, i);
		}
		public LogicalOrExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_logicalOrExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterLogicalOrExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitLogicalOrExpr(this);
		}
	}

	public final LogicalOrExprContext logicalOrExpr() throws RecognitionException {
		LogicalOrExprContext _localctx = new LogicalOrExprContext(_ctx, getState());
		enterRule(_localctx, 6, RULE_logicalOrExpr);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(79);
			logicalAndExpr();
			setState(85);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (_la==OR) {
				{
				{
				setState(80);
				match(OR);
				setState(81);
				match(OR);
				setState(82);
				logicalAndExpr();
				}
				}
				setState(87);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class LogicalAndExprContext extends ParserRuleContext {
		public List<EqualityExprContext> equalityExpr() {
			return getRuleContexts(EqualityExprContext.class);
		}
		public EqualityExprContext equalityExpr(int i) {
			return getRuleContext(EqualityExprContext.class,i);
		}
		public List<TerminalNode> AND() { return getTokens(FoolishParser.AND); }
		public TerminalNode AND(int i) {
			return getToken(FoolishParser.AND, i);
		}
		public LogicalAndExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_logicalAndExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterLogicalAndExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitLogicalAndExpr(this);
		}
	}

	public final LogicalAndExprContext logicalAndExpr() throws RecognitionException {
		LogicalAndExprContext _localctx = new LogicalAndExprContext(_ctx, getState());
		enterRule(_localctx, 8, RULE_logicalAndExpr);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(88);
			equalityExpr();
			setState(94);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (_la==AND) {
				{
				{
				setState(89);
				match(AND);
				setState(90);
				match(AND);
				setState(91);
				equalityExpr();
				}
				}
				setState(96);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class EqualityExprContext extends ParserRuleContext {
		public List<RelationalExprContext> relationalExpr() {
			return getRuleContexts(RelationalExprContext.class);
		}
		public RelationalExprContext relationalExpr(int i) {
			return getRuleContext(RelationalExprContext.class,i);
		}
		public List<TerminalNode> EQEQ() { return getTokens(FoolishParser.EQEQ); }
		public TerminalNode EQEQ(int i) {
			return getToken(FoolishParser.EQEQ, i);
		}
		public EqualityExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_equalityExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterEqualityExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitEqualityExpr(this);
		}
	}

	public final EqualityExprContext equalityExpr() throws RecognitionException {
		EqualityExprContext _localctx = new EqualityExprContext(_ctx, getState());
		enterRule(_localctx, 10, RULE_equalityExpr);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(97);
			relationalExpr();
			setState(102);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (_la==EQEQ) {
				{
				{
				setState(98);
				match(EQEQ);
				setState(99);
				relationalExpr();
				}
				}
				setState(104);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class RelationalExprContext extends ParserRuleContext {
		public List<AddExprContext> addExpr() {
			return getRuleContexts(AddExprContext.class);
		}
		public AddExprContext addExpr(int i) {
			return getRuleContext(AddExprContext.class,i);
		}
		public List<TerminalNode> LT() { return getTokens(FoolishParser.LT); }
		public TerminalNode LT(int i) {
			return getToken(FoolishParser.LT, i);
		}
		public List<TerminalNode> GT() { return getTokens(FoolishParser.GT); }
		public TerminalNode GT(int i) {
			return getToken(FoolishParser.GT, i);
		}
		public List<TerminalNode> LE() { return getTokens(FoolishParser.LE); }
		public TerminalNode LE(int i) {
			return getToken(FoolishParser.LE, i);
		}
		public List<TerminalNode> GE() { return getTokens(FoolishParser.GE); }
		public TerminalNode GE(int i) {
			return getToken(FoolishParser.GE, i);
		}
		public RelationalExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_relationalExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterRelationalExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitRelationalExpr(this);
		}
	}

	public final RelationalExprContext relationalExpr() throws RecognitionException {
		RelationalExprContext _localctx = new RelationalExprContext(_ctx, getState());
		enterRule(_localctx, 12, RULE_relationalExpr);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(105);
			addExpr();
			setState(110);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while ((((_la) & ~0x3f) == 0 && ((1L << _la) & 7680L) != 0)) {
				{
				{
				setState(106);
				_la = _input.LA(1);
				if ( !((((_la) & ~0x3f) == 0 && ((1L << _la) & 7680L) != 0)) ) {
				_errHandler.recoverInline(this);
				}
				else {
					if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
					_errHandler.reportMatch(this);
					consume();
				}
				setState(107);
				addExpr();
				}
				}
				setState(112);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class AddExprContext extends ParserRuleContext {
		public List<MulExprContext> mulExpr() {
			return getRuleContexts(MulExprContext.class);
		}
		public MulExprContext mulExpr(int i) {
			return getRuleContext(MulExprContext.class,i);
		}
		public List<TerminalNode> PLUS() { return getTokens(FoolishParser.PLUS); }
		public TerminalNode PLUS(int i) {
			return getToken(FoolishParser.PLUS, i);
		}
		public List<TerminalNode> MINUS() { return getTokens(FoolishParser.MINUS); }
		public TerminalNode MINUS(int i) {
			return getToken(FoolishParser.MINUS, i);
		}
		public AddExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_addExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterAddExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitAddExpr(this);
		}
	}

	public final AddExprContext addExpr() throws RecognitionException {
		AddExprContext _localctx = new AddExprContext(_ctx, getState());
		enterRule(_localctx, 14, RULE_addExpr);
		int _la;
		try {
			int _alt;
			enterOuterAlt(_localctx, 1);
			{
			setState(113);
			mulExpr();
			setState(118);
			_errHandler.sync(this);
			_alt = getInterpreter().adaptivePredict(_input,4,_ctx);
			while ( _alt!=2 && _alt!=org.antlr.v4.runtime.atn.ATN.INVALID_ALT_NUMBER ) {
				if ( _alt==1 ) {
					{
					{
					setState(114);
					_la = _input.LA(1);
					if ( !(_la==PLUS || _la==MINUS) ) {
					_errHandler.recoverInline(this);
					}
					else {
						if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
						_errHandler.reportMatch(this);
						consume();
					}
					setState(115);
					mulExpr();
					}
					} 
				}
				setState(120);
				_errHandler.sync(this);
				_alt = getInterpreter().adaptivePredict(_input,4,_ctx);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class MulExprContext extends ParserRuleContext {
		public List<ConcatExprContext> concatExpr() {
			return getRuleContexts(ConcatExprContext.class);
		}
		public ConcatExprContext concatExpr(int i) {
			return getRuleContext(ConcatExprContext.class,i);
		}
		public List<TerminalNode> STAR() { return getTokens(FoolishParser.STAR); }
		public TerminalNode STAR(int i) {
			return getToken(FoolishParser.STAR, i);
		}
		public List<TerminalNode> SLASH() { return getTokens(FoolishParser.SLASH); }
		public TerminalNode SLASH(int i) {
			return getToken(FoolishParser.SLASH, i);
		}
		public MulExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_mulExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterMulExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitMulExpr(this);
		}
	}

	public final MulExprContext mulExpr() throws RecognitionException {
		MulExprContext _localctx = new MulExprContext(_ctx, getState());
		enterRule(_localctx, 16, RULE_mulExpr);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(121);
			concatExpr();
			setState(126);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (_la==STAR || _la==SLASH) {
				{
				{
				setState(122);
				_la = _input.LA(1);
				if ( !(_la==STAR || _la==SLASH) ) {
				_errHandler.recoverInline(this);
				}
				else {
					if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
					_errHandler.reportMatch(this);
					consume();
				}
				setState(123);
				concatExpr();
				}
				}
				setState(128);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class ConcatExprContext extends ParserRuleContext {
		public List<PostfixExprContext> postfixExpr() {
			return getRuleContexts(PostfixExprContext.class);
		}
		public PostfixExprContext postfixExpr(int i) {
			return getRuleContext(PostfixExprContext.class,i);
		}
		public ConcatExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_concatExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterConcatExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitConcatExpr(this);
		}
	}

	public final ConcatExprContext concatExpr() throws RecognitionException {
		ConcatExprContext _localctx = new ConcatExprContext(_ctx, getState());
		enterRule(_localctx, 18, RULE_concatExpr);
		try {
			int _alt;
			enterOuterAlt(_localctx, 1);
			{
			setState(129);
			postfixExpr();
			setState(133);
			_errHandler.sync(this);
			_alt = getInterpreter().adaptivePredict(_input,6,_ctx);
			while ( _alt!=2 && _alt!=org.antlr.v4.runtime.atn.ATN.INVALID_ALT_NUMBER ) {
				if ( _alt==1 ) {
					{
					{
					setState(130);
					postfixExpr();
					}
					} 
				}
				setState(135);
				_errHandler.sync(this);
				_alt = getInterpreter().adaptivePredict(_input,6,_ctx);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class PostfixExprContext extends ParserRuleContext {
		public PrimaryExprContext primaryExpr() {
			return getRuleContext(PrimaryExprContext.class,0);
		}
		public List<PathOpContext> pathOp() {
			return getRuleContexts(PathOpContext.class);
		}
		public PathOpContext pathOp(int i) {
			return getRuleContext(PathOpContext.class,i);
		}
		public PostfixExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_postfixExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterPostfixExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitPostfixExpr(this);
		}
	}

	public final PostfixExprContext postfixExpr() throws RecognitionException {
		PostfixExprContext _localctx = new PostfixExprContext(_ctx, getState());
		enterRule(_localctx, 20, RULE_postfixExpr);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(136);
			primaryExpr();
			setState(140);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while ((((_la) & ~0x3f) == 0 && ((1L << _la) & 917504L) != 0)) {
				{
				{
				setState(137);
				pathOp();
				}
				}
				setState(142);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class PathOpContext extends ParserRuleContext {
		public TerminalNode CARET() { return getToken(FoolishParser.CARET, 0); }
		public BraneIndexContext braneIndex() {
			return getRuleContext(BraneIndexContext.class,0);
		}
		public TerminalNode DOLLAR() { return getToken(FoolishParser.DOLLAR, 0); }
		public TerminalNode HASH() { return getToken(FoolishParser.HASH, 0); }
		public PathOpContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_pathOp; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterPathOp(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitPathOp(this);
		}
	}

	public final PathOpContext pathOp() throws RecognitionException {
		PathOpContext _localctx = new PathOpContext(_ctx, getState());
		enterRule(_localctx, 22, RULE_pathOp);
		try {
			setState(153);
			_errHandler.sync(this);
			switch (_input.LA(1)) {
			case CARET:
				enterOuterAlt(_localctx, 1);
				{
				setState(143);
				match(CARET);
				setState(145);
				_errHandler.sync(this);
				switch ( getInterpreter().adaptivePredict(_input,8,_ctx) ) {
				case 1:
					{
					setState(144);
					braneIndex();
					}
					break;
				}
				}
				break;
			case DOLLAR:
				enterOuterAlt(_localctx, 2);
				{
				setState(147);
				match(DOLLAR);
				setState(149);
				_errHandler.sync(this);
				switch ( getInterpreter().adaptivePredict(_input,9,_ctx) ) {
				case 1:
					{
					setState(148);
					braneIndex();
					}
					break;
				}
				}
				break;
			case HASH:
				enterOuterAlt(_localctx, 3);
				{
				setState(151);
				match(HASH);
				setState(152);
				braneIndex();
				}
				break;
			default:
				throw new NoViableAltException(this);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class PrimaryExprContext extends ParserRuleContext {
		public LiteralContext literal() {
			return getRuleContext(LiteralContext.class,0);
		}
		public IdentifierContext identifier() {
			return getRuleContext(IdentifierContext.class,0);
		}
		public FuncExprContext funcExpr() {
			return getRuleContext(FuncExprContext.class,0);
		}
		public BraneExprContext braneExpr() {
			return getRuleContext(BraneExprContext.class,0);
		}
		public TerminalNode LPAREN() { return getToken(FoolishParser.LPAREN, 0); }
		public ExpressionContext expression() {
			return getRuleContext(ExpressionContext.class,0);
		}
		public TerminalNode RPAREN() { return getToken(FoolishParser.RPAREN, 0); }
		public PrimaryExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_primaryExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterPrimaryExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitPrimaryExpr(this);
		}
	}

	public final PrimaryExprContext primaryExpr() throws RecognitionException {
		PrimaryExprContext _localctx = new PrimaryExprContext(_ctx, getState());
		enterRule(_localctx, 24, RULE_primaryExpr);
		try {
			setState(163);
			_errHandler.sync(this);
			switch ( getInterpreter().adaptivePredict(_input,11,_ctx) ) {
			case 1:
				enterOuterAlt(_localctx, 1);
				{
				setState(155);
				literal();
				}
				break;
			case 2:
				enterOuterAlt(_localctx, 2);
				{
				setState(156);
				identifier();
				}
				break;
			case 3:
				enterOuterAlt(_localctx, 3);
				{
				setState(157);
				funcExpr();
				}
				break;
			case 4:
				enterOuterAlt(_localctx, 4);
				{
				setState(158);
				braneExpr();
				}
				break;
			case 5:
				enterOuterAlt(_localctx, 5);
				{
				setState(159);
				match(LPAREN);
				setState(160);
				expression();
				setState(161);
				match(RPAREN);
				}
				break;
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class FuncExprContext extends ParserRuleContext {
		public TerminalNode LPAREN() { return getToken(FoolishParser.LPAREN, 0); }
		public TerminalNode RPAREN() { return getToken(FoolishParser.RPAREN, 0); }
		public TerminalNode ARROW() { return getToken(FoolishParser.ARROW, 0); }
		public BraneContext brane() {
			return getRuleContext(BraneContext.class,0);
		}
		public ParamListContext paramList() {
			return getRuleContext(ParamListContext.class,0);
		}
		public FuncExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_funcExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterFuncExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitFuncExpr(this);
		}
	}

	public final FuncExprContext funcExpr() throws RecognitionException {
		FuncExprContext _localctx = new FuncExprContext(_ctx, getState());
		enterRule(_localctx, 26, RULE_funcExpr);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(165);
			match(LPAREN);
			setState(167);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==TYPE_IDENTIFIER || _la==ORD_IDENTIFIER) {
				{
				setState(166);
				paramList();
				}
			}

			setState(169);
			match(RPAREN);
			setState(170);
			match(ARROW);
			setState(171);
			brane();
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class ParamListContext extends ParserRuleContext {
		public List<ParamContext> param() {
			return getRuleContexts(ParamContext.class);
		}
		public ParamContext param(int i) {
			return getRuleContext(ParamContext.class,i);
		}
		public List<TerminalNode> COMMA() { return getTokens(FoolishParser.COMMA); }
		public TerminalNode COMMA(int i) {
			return getToken(FoolishParser.COMMA, i);
		}
		public ParamListContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_paramList; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterParamList(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitParamList(this);
		}
	}

	public final ParamListContext paramList() throws RecognitionException {
		ParamListContext _localctx = new ParamListContext(_ctx, getState());
		enterRule(_localctx, 28, RULE_paramList);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(173);
			param();
			setState(178);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (_la==COMMA) {
				{
				{
				setState(174);
				match(COMMA);
				setState(175);
				param();
				}
				}
				setState(180);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class ParamContext extends ParserRuleContext {
		public IdentifierContext identifier() {
			return getRuleContext(IdentifierContext.class,0);
		}
		public TerminalNode COLON() { return getToken(FoolishParser.COLON, 0); }
		public PrimitiveTypeContext primitiveType() {
			return getRuleContext(PrimitiveTypeContext.class,0);
		}
		public TypeIdentifierContext typeIdentifier() {
			return getRuleContext(TypeIdentifierContext.class,0);
		}
		public ParamContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_param; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterParam(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitParam(this);
		}
	}

	public final ParamContext param() throws RecognitionException {
		ParamContext _localctx = new ParamContext(_ctx, getState());
		enterRule(_localctx, 30, RULE_param);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(181);
			identifier();
			setState(182);
			match(COLON);
			setState(185);
			_errHandler.sync(this);
			switch (_input.LA(1)) {
			case INT_T:
			case FLOAT_T:
			case STRING_T:
			case BRANE_T:
			case LDBLBRACE:
				{
				setState(183);
				primitiveType();
				}
				break;
			case TYPE_KIND:
				{
				setState(184);
				typeIdentifier();
				}
				break;
			default:
				throw new NoViableAltException(this);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class BraneExprContext extends ParserRuleContext {
		public TerminalNode LBRACE() { return getToken(FoolishParser.LBRACE, 0); }
		public TerminalNode RBRACE() { return getToken(FoolishParser.RBRACE, 0); }
		public List<BraneStmtContext> braneStmt() {
			return getRuleContexts(BraneStmtContext.class);
		}
		public BraneStmtContext braneStmt(int i) {
			return getRuleContext(BraneStmtContext.class,i);
		}
		public BraneExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_braneExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterBraneExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitBraneExpr(this);
		}
	}

	public final BraneExprContext braneExpr() throws RecognitionException {
		BraneExprContext _localctx = new BraneExprContext(_ctx, getState());
		enterRule(_localctx, 32, RULE_braneExpr);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(187);
			match(LBRACE);
			setState(191);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while ((((_la) & ~0x3f) == 0 && ((1L << _la) & 264161484800L) != 0)) {
				{
				{
				setState(188);
				braneStmt();
				}
				}
				setState(193);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			setState(194);
			match(RBRACE);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class BraneStmtContext extends ParserRuleContext {
		public ExpressionContext expression() {
			return getRuleContext(ExpressionContext.class,0);
		}
		public AssignmentExpressionContext assignmentExpression() {
			return getRuleContext(AssignmentExpressionContext.class,0);
		}
		public List<TerminalNode> SEMI() { return getTokens(FoolishParser.SEMI); }
		public TerminalNode SEMI(int i) {
			return getToken(FoolishParser.SEMI, i);
		}
		public List<TerminalNode> NEWLINE() { return getTokens(FoolishParser.NEWLINE); }
		public TerminalNode NEWLINE(int i) {
			return getToken(FoolishParser.NEWLINE, i);
		}
		public BraneStmtContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_braneStmt; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterBraneStmt(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitBraneStmt(this);
		}
	}

	public final BraneStmtContext braneStmt() throws RecognitionException {
		BraneStmtContext _localctx = new BraneStmtContext(_ctx, getState());
		enterRule(_localctx, 34, RULE_braneStmt);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(198);
			_errHandler.sync(this);
			switch ( getInterpreter().adaptivePredict(_input,16,_ctx) ) {
			case 1:
				{
				setState(196);
				expression();
				}
				break;
			case 2:
				{
				setState(197);
				assignmentExpression();
				}
				break;
			}
			setState(203);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (_la==SEMI || _la==NEWLINE) {
				{
				{
				setState(200);
				_la = _input.LA(1);
				if ( !(_la==SEMI || _la==NEWLINE) ) {
				_errHandler.recoverInline(this);
				}
				else {
					if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
					_errHandler.reportMatch(this);
					consume();
				}
				}
				}
				setState(205);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class AssignmentExpressionContext extends ParserRuleContext {
		public AssignExprContext assignExpr() {
			return getRuleContext(AssignExprContext.class,0);
		}
		public TypeAssignExprContext typeAssignExpr() {
			return getRuleContext(TypeAssignExprContext.class,0);
		}
		public DerefAssignExprContext derefAssignExpr() {
			return getRuleContext(DerefAssignExprContext.class,0);
		}
		public AssignmentExpressionContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_assignmentExpression; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterAssignmentExpression(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitAssignmentExpression(this);
		}
	}

	public final AssignmentExpressionContext assignmentExpression() throws RecognitionException {
		AssignmentExpressionContext _localctx = new AssignmentExpressionContext(_ctx, getState());
		enterRule(_localctx, 36, RULE_assignmentExpression);
		try {
			setState(209);
			_errHandler.sync(this);
			switch ( getInterpreter().adaptivePredict(_input,18,_ctx) ) {
			case 1:
				enterOuterAlt(_localctx, 1);
				{
				setState(206);
				assignExpr();
				}
				break;
			case 2:
				enterOuterAlt(_localctx, 2);
				{
				setState(207);
				typeAssignExpr();
				}
				break;
			case 3:
				enterOuterAlt(_localctx, 3);
				{
				setState(208);
				derefAssignExpr();
				}
				break;
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class AssignExprContext extends ParserRuleContext {
		public IdentifierContext identifier() {
			return getRuleContext(IdentifierContext.class,0);
		}
		public TerminalNode ASSIGN() { return getToken(FoolishParser.ASSIGN, 0); }
		public ExpressionContext expression() {
			return getRuleContext(ExpressionContext.class,0);
		}
		public AssignExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_assignExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterAssignExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitAssignExpr(this);
		}
	}

	public final AssignExprContext assignExpr() throws RecognitionException {
		AssignExprContext _localctx = new AssignExprContext(_ctx, getState());
		enterRule(_localctx, 38, RULE_assignExpr);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(211);
			identifier();
			setState(212);
			match(ASSIGN);
			setState(213);
			expression();
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class TypeAssignExprContext extends ParserRuleContext {
		public TypeIdentifierContext typeIdentifier() {
			return getRuleContext(TypeIdentifierContext.class,0);
		}
		public TerminalNode ASSIGN() { return getToken(FoolishParser.ASSIGN, 0); }
		public TypeExpressionContext typeExpression() {
			return getRuleContext(TypeExpressionContext.class,0);
		}
		public TypeAssignExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_typeAssignExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterTypeAssignExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitTypeAssignExpr(this);
		}
	}

	public final TypeAssignExprContext typeAssignExpr() throws RecognitionException {
		TypeAssignExprContext _localctx = new TypeAssignExprContext(_ctx, getState());
		enterRule(_localctx, 40, RULE_typeAssignExpr);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(215);
			typeIdentifier();
			setState(216);
			match(ASSIGN);
			setState(217);
			typeExpression();
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class TypeExpressionContext extends ParserRuleContext {
		public PrimitiveTypeContext primitiveType() {
			return getRuleContext(PrimitiveTypeContext.class,0);
		}
		public TypeIdentifierContext typeIdentifier() {
			return getRuleContext(TypeIdentifierContext.class,0);
		}
		public TypeExpressionContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_typeExpression; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterTypeExpression(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitTypeExpression(this);
		}
	}

	public final TypeExpressionContext typeExpression() throws RecognitionException {
		TypeExpressionContext _localctx = new TypeExpressionContext(_ctx, getState());
		enterRule(_localctx, 42, RULE_typeExpression);
		try {
			setState(221);
			_errHandler.sync(this);
			switch (_input.LA(1)) {
			case INT_T:
			case FLOAT_T:
			case STRING_T:
			case BRANE_T:
			case LDBLBRACE:
				enterOuterAlt(_localctx, 1);
				{
				setState(219);
				primitiveType();
				}
				break;
			case TYPE_KIND:
				enterOuterAlt(_localctx, 2);
				{
				setState(220);
				typeIdentifier();
				}
				break;
			default:
				throw new NoViableAltException(this);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class DerefAssignExprContext extends ParserRuleContext {
		public IdentifierContext identifier() {
			return getRuleContext(IdentifierContext.class,0);
		}
		public TerminalNode ASSIGN_DEREF() { return getToken(FoolishParser.ASSIGN_DEREF, 0); }
		public ExpressionContext expression() {
			return getRuleContext(ExpressionContext.class,0);
		}
		public DerefAssignExprContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_derefAssignExpr; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterDerefAssignExpr(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitDerefAssignExpr(this);
		}
	}

	public final DerefAssignExprContext derefAssignExpr() throws RecognitionException {
		DerefAssignExprContext _localctx = new DerefAssignExprContext(_ctx, getState());
		enterRule(_localctx, 44, RULE_derefAssignExpr);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(223);
			identifier();
			setState(224);
			match(ASSIGN_DEREF);
			setState(225);
			expression();
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class BraneTypeDefContext extends ParserRuleContext {
		public TerminalNode LDBLBRACE() { return getToken(FoolishParser.LDBLBRACE, 0); }
		public TerminalNode RDBLBRACE() { return getToken(FoolishParser.RDBLBRACE, 0); }
		public List<FieldDefContext> fieldDef() {
			return getRuleContexts(FieldDefContext.class);
		}
		public FieldDefContext fieldDef(int i) {
			return getRuleContext(FieldDefContext.class,i);
		}
		public List<TerminalNode> COMMA() { return getTokens(FoolishParser.COMMA); }
		public TerminalNode COMMA(int i) {
			return getToken(FoolishParser.COMMA, i);
		}
		public BraneTypeDefContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_braneTypeDef; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterBraneTypeDef(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitBraneTypeDef(this);
		}
	}

	public final BraneTypeDefContext braneTypeDef() throws RecognitionException {
		BraneTypeDefContext _localctx = new BraneTypeDefContext(_ctx, getState());
		enterRule(_localctx, 46, RULE_braneTypeDef);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(227);
			match(LDBLBRACE);
			setState(236);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==TYPE_IDENTIFIER || _la==ORD_IDENTIFIER) {
				{
				setState(228);
				fieldDef();
				setState(233);
				_errHandler.sync(this);
				_la = _input.LA(1);
				while (_la==COMMA) {
					{
					{
					setState(229);
					match(COMMA);
					setState(230);
					fieldDef();
					}
					}
					setState(235);
					_errHandler.sync(this);
					_la = _input.LA(1);
				}
				}
			}

			setState(238);
			match(RDBLBRACE);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class FieldDefContext extends ParserRuleContext {
		public IdentifierContext identifier() {
			return getRuleContext(IdentifierContext.class,0);
		}
		public TerminalNode COLON() { return getToken(FoolishParser.COLON, 0); }
		public PrimitiveTypeContext primitiveType() {
			return getRuleContext(PrimitiveTypeContext.class,0);
		}
		public TypeIdentifierContext typeIdentifier() {
			return getRuleContext(TypeIdentifierContext.class,0);
		}
		public FieldDefContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_fieldDef; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterFieldDef(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitFieldDef(this);
		}
	}

	public final FieldDefContext fieldDef() throws RecognitionException {
		FieldDefContext _localctx = new FieldDefContext(_ctx, getState());
		enterRule(_localctx, 48, RULE_fieldDef);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(240);
			identifier();
			setState(241);
			match(COLON);
			setState(244);
			_errHandler.sync(this);
			switch (_input.LA(1)) {
			case INT_T:
			case FLOAT_T:
			case STRING_T:
			case BRANE_T:
			case LDBLBRACE:
				{
				setState(242);
				primitiveType();
				}
				break;
			case TYPE_KIND:
				{
				setState(243);
				typeIdentifier();
				}
				break;
			default:
				throw new NoViableAltException(this);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class PrimitiveTypeContext extends ParserRuleContext {
		public TerminalNode INT_T() { return getToken(FoolishParser.INT_T, 0); }
		public TerminalNode FLOAT_T() { return getToken(FoolishParser.FLOAT_T, 0); }
		public TerminalNode STRING_T() { return getToken(FoolishParser.STRING_T, 0); }
		public TerminalNode BRANE_T() { return getToken(FoolishParser.BRANE_T, 0); }
		public BraneTypeDefContext braneTypeDef() {
			return getRuleContext(BraneTypeDefContext.class,0);
		}
		public PrimitiveTypeContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_primitiveType; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterPrimitiveType(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitPrimitiveType(this);
		}
	}

	public final PrimitiveTypeContext primitiveType() throws RecognitionException {
		PrimitiveTypeContext _localctx = new PrimitiveTypeContext(_ctx, getState());
		enterRule(_localctx, 50, RULE_primitiveType);
		try {
			setState(251);
			_errHandler.sync(this);
			switch (_input.LA(1)) {
			case INT_T:
				enterOuterAlt(_localctx, 1);
				{
				setState(246);
				match(INT_T);
				}
				break;
			case FLOAT_T:
				enterOuterAlt(_localctx, 2);
				{
				setState(247);
				match(FLOAT_T);
				}
				break;
			case STRING_T:
				enterOuterAlt(_localctx, 3);
				{
				setState(248);
				match(STRING_T);
				}
				break;
			case BRANE_T:
				enterOuterAlt(_localctx, 4);
				{
				setState(249);
				match(BRANE_T);
				}
				break;
			case LDBLBRACE:
				enterOuterAlt(_localctx, 5);
				{
				setState(250);
				braneTypeDef();
				}
				break;
			default:
				throw new NoViableAltException(this);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class TypeIdentifierContext extends ParserRuleContext {
		public TerminalNode TYPE_KIND() { return getToken(FoolishParser.TYPE_KIND, 0); }
		public TerminalNode PRIM_QUOTE() { return getToken(FoolishParser.PRIM_QUOTE, 0); }
		public TerminalNode TYPE_IDENTIFIER() { return getToken(FoolishParser.TYPE_IDENTIFIER, 0); }
		public TypeIdentifierContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_typeIdentifier; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterTypeIdentifier(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitTypeIdentifier(this);
		}
	}

	public final TypeIdentifierContext typeIdentifier() throws RecognitionException {
		TypeIdentifierContext _localctx = new TypeIdentifierContext(_ctx, getState());
		enterRule(_localctx, 52, RULE_typeIdentifier);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(253);
			match(TYPE_KIND);
			setState(254);
			match(PRIM_QUOTE);
			setState(255);
			match(TYPE_IDENTIFIER);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class BraneIndexContext extends ParserRuleContext {
		public IdentifierContext identifier() {
			return getRuleContext(IdentifierContext.class,0);
		}
		public IntLiteralContext intLiteral() {
			return getRuleContext(IntLiteralContext.class,0);
		}
		public BraneIndexContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_braneIndex; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterBraneIndex(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitBraneIndex(this);
		}
	}

	public final BraneIndexContext braneIndex() throws RecognitionException {
		BraneIndexContext _localctx = new BraneIndexContext(_ctx, getState());
		enterRule(_localctx, 54, RULE_braneIndex);
		try {
			setState(259);
			_errHandler.sync(this);
			switch (_input.LA(1)) {
			case TYPE_IDENTIFIER:
			case ORD_IDENTIFIER:
				enterOuterAlt(_localctx, 1);
				{
				setState(257);
				identifier();
				}
				break;
			case PLUS:
			case MINUS:
			case NUMBER:
				enterOuterAlt(_localctx, 2);
				{
				setState(258);
				intLiteral();
				}
				break;
			default:
				throw new NoViableAltException(this);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class LiteralContext extends ParserRuleContext {
		public PrimitiveLiteralContext primitiveLiteral() {
			return getRuleContext(PrimitiveLiteralContext.class,0);
		}
		public TypeLiteralContext typeLiteral() {
			return getRuleContext(TypeLiteralContext.class,0);
		}
		public LiteralContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_literal; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterLiteral(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitLiteral(this);
		}
	}

	public final LiteralContext literal() throws RecognitionException {
		LiteralContext _localctx = new LiteralContext(_ctx, getState());
		enterRule(_localctx, 56, RULE_literal);
		try {
			setState(263);
			_errHandler.sync(this);
			switch (_input.LA(1)) {
			case PLUS:
			case MINUS:
			case DOT:
			case DQUOTE:
			case NUMBER:
				enterOuterAlt(_localctx, 1);
				{
				setState(261);
				primitiveLiteral();
				}
				break;
			case TYPE_KIND:
				enterOuterAlt(_localctx, 2);
				{
				setState(262);
				typeLiteral();
				}
				break;
			default:
				throw new NoViableAltException(this);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class PrimitiveLiteralContext extends ParserRuleContext {
		public IntLiteralContext intLiteral() {
			return getRuleContext(IntLiteralContext.class,0);
		}
		public FloatLiteralContext floatLiteral() {
			return getRuleContext(FloatLiteralContext.class,0);
		}
		public StringLiteralContext stringLiteral() {
			return getRuleContext(StringLiteralContext.class,0);
		}
		public PrimitiveLiteralContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_primitiveLiteral; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterPrimitiveLiteral(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitPrimitiveLiteral(this);
		}
	}

	public final PrimitiveLiteralContext primitiveLiteral() throws RecognitionException {
		PrimitiveLiteralContext _localctx = new PrimitiveLiteralContext(_ctx, getState());
		enterRule(_localctx, 58, RULE_primitiveLiteral);
		try {
			setState(268);
			_errHandler.sync(this);
			switch ( getInterpreter().adaptivePredict(_input,26,_ctx) ) {
			case 1:
				enterOuterAlt(_localctx, 1);
				{
				setState(265);
				intLiteral();
				}
				break;
			case 2:
				enterOuterAlt(_localctx, 2);
				{
				setState(266);
				floatLiteral();
				}
				break;
			case 3:
				enterOuterAlt(_localctx, 3);
				{
				setState(267);
				stringLiteral();
				}
				break;
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class PrefixSignContext extends ParserRuleContext {
		public TerminalNode MINUS() { return getToken(FoolishParser.MINUS, 0); }
		public TerminalNode PLUS() { return getToken(FoolishParser.PLUS, 0); }
		public PrefixSignContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_prefixSign; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterPrefixSign(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitPrefixSign(this);
		}
	}

	public final PrefixSignContext prefixSign() throws RecognitionException {
		PrefixSignContext _localctx = new PrefixSignContext(_ctx, getState());
		enterRule(_localctx, 60, RULE_prefixSign);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(270);
			_la = _input.LA(1);
			if ( !(_la==PLUS || _la==MINUS) ) {
			_errHandler.recoverInline(this);
			}
			else {
				if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
				_errHandler.reportMatch(this);
				consume();
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class IntLiteralContext extends ParserRuleContext {
		public TerminalNode NUMBER() { return getToken(FoolishParser.NUMBER, 0); }
		public PrefixSignContext prefixSign() {
			return getRuleContext(PrefixSignContext.class,0);
		}
		public IntLiteralContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_intLiteral; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterIntLiteral(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitIntLiteral(this);
		}
	}

	public final IntLiteralContext intLiteral() throws RecognitionException {
		IntLiteralContext _localctx = new IntLiteralContext(_ctx, getState());
		enterRule(_localctx, 62, RULE_intLiteral);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(273);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==PLUS || _la==MINUS) {
				{
				setState(272);
				prefixSign();
				}
			}

			setState(275);
			match(NUMBER);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class FloatLiteralContext extends ParserRuleContext {
		public TerminalNode DOT() { return getToken(FoolishParser.DOT, 0); }
		public List<TerminalNode> NUMBER() { return getTokens(FoolishParser.NUMBER); }
		public TerminalNode NUMBER(int i) {
			return getToken(FoolishParser.NUMBER, i);
		}
		public PrefixSignContext prefixSign() {
			return getRuleContext(PrefixSignContext.class,0);
		}
		public FloatLiteralContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_floatLiteral; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterFloatLiteral(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitFloatLiteral(this);
		}
	}

	public final FloatLiteralContext floatLiteral() throws RecognitionException {
		FloatLiteralContext _localctx = new FloatLiteralContext(_ctx, getState());
		enterRule(_localctx, 64, RULE_floatLiteral);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(278);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==PLUS || _la==MINUS) {
				{
				setState(277);
				prefixSign();
				}
			}

			setState(281);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==NUMBER) {
				{
				setState(280);
				match(NUMBER);
				}
			}

			setState(283);
			match(DOT);
			setState(284);
			match(NUMBER);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class StringLiteralContext extends ParserRuleContext {
		public List<TerminalNode> DQUOTE() { return getTokens(FoolishParser.DQUOTE); }
		public TerminalNode DQUOTE(int i) {
			return getToken(FoolishParser.DQUOTE, i);
		}
		public StringLiteralContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_stringLiteral; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterStringLiteral(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitStringLiteral(this);
		}
	}

	public final StringLiteralContext stringLiteral() throws RecognitionException {
		StringLiteralContext _localctx = new StringLiteralContext(_ctx, getState());
		enterRule(_localctx, 66, RULE_stringLiteral);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(286);
			match(DQUOTE);
			setState(290);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while ((((_la) & ~0x3f) == 0 && ((1L << _la) & 4393751543806L) != 0)) {
				{
				{
				setState(287);
				_la = _input.LA(1);
				if ( _la <= 0 || (_la==DQUOTE) ) {
				_errHandler.recoverInline(this);
				}
				else {
					if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
					_errHandler.reportMatch(this);
					consume();
				}
				}
				}
				setState(292);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			setState(293);
			match(DQUOTE);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class TypeLiteralContext extends ParserRuleContext {
		public TerminalNode TYPE_KIND() { return getToken(FoolishParser.TYPE_KIND, 0); }
		public TerminalNode PRIM_QUOTE() { return getToken(FoolishParser.PRIM_QUOTE, 0); }
		public PrimitiveLiteralContext primitiveLiteral() {
			return getRuleContext(PrimitiveLiteralContext.class,0);
		}
		public TypeLiteralContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_typeLiteral; }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterTypeLiteral(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitTypeLiteral(this);
		}
	}

	public final TypeLiteralContext typeLiteral() throws RecognitionException {
		TypeLiteralContext _localctx = new TypeLiteralContext(_ctx, getState());
		enterRule(_localctx, 68, RULE_typeLiteral);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(295);
			match(TYPE_KIND);
			setState(296);
			match(PRIM_QUOTE);
			setState(297);
			primitiveLiteral();
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class IdentifierContext extends ParserRuleContext {
		public IdentifierContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_identifier; }
	 
		public IdentifierContext() { }
		public void copyFrom(IdentifierContext ctx) {
			super.copyFrom(ctx);
		}
	}
	@SuppressWarnings("CheckReturnValue")
	public static class OrdIdentContext extends IdentifierContext {
		public TerminalNode ORD_IDENTIFIER() { return getToken(FoolishParser.ORD_IDENTIFIER, 0); }
		public OrdIdentContext(IdentifierContext ctx) { copyFrom(ctx); }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterOrdIdent(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitOrdIdent(this);
		}
	}
	@SuppressWarnings("CheckReturnValue")
	public static class TypeIdentContext extends IdentifierContext {
		public TerminalNode TYPE_IDENTIFIER() { return getToken(FoolishParser.TYPE_IDENTIFIER, 0); }
		public TypeIdentContext(IdentifierContext ctx) { copyFrom(ctx); }
		@Override
		public void enterRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).enterTypeIdent(this);
		}
		@Override
		public void exitRule(ParseTreeListener listener) {
			if ( listener instanceof FoolishListener ) ((FoolishListener)listener).exitTypeIdent(this);
		}
	}

	public final IdentifierContext identifier() throws RecognitionException {
		IdentifierContext _localctx = new IdentifierContext(_ctx, getState());
		enterRule(_localctx, 70, RULE_identifier);
		try {
			setState(301);
			_errHandler.sync(this);
			switch (_input.LA(1)) {
			case TYPE_IDENTIFIER:
				_localctx = new TypeIdentContext(_localctx);
				enterOuterAlt(_localctx, 1);
				{
				setState(299);
				match(TYPE_IDENTIFIER);
				}
				break;
			case ORD_IDENTIFIER:
				_localctx = new OrdIdentContext(_localctx);
				enterOuterAlt(_localctx, 2);
				{
				setState(300);
				match(ORD_IDENTIFIER);
				}
				break;
			default:
				throw new NoViableAltException(this);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	public static final String _serializedATN =
		"\u0004\u0001)\u0130\u0002\u0000\u0007\u0000\u0002\u0001\u0007\u0001\u0002"+
		"\u0002\u0007\u0002\u0002\u0003\u0007\u0003\u0002\u0004\u0007\u0004\u0002"+
		"\u0005\u0007\u0005\u0002\u0006\u0007\u0006\u0002\u0007\u0007\u0007\u0002"+
		"\b\u0007\b\u0002\t\u0007\t\u0002\n\u0007\n\u0002\u000b\u0007\u000b\u0002"+
		"\f\u0007\f\u0002\r\u0007\r\u0002\u000e\u0007\u000e\u0002\u000f\u0007\u000f"+
		"\u0002\u0010\u0007\u0010\u0002\u0011\u0007\u0011\u0002\u0012\u0007\u0012"+
		"\u0002\u0013\u0007\u0013\u0002\u0014\u0007\u0014\u0002\u0015\u0007\u0015"+
		"\u0002\u0016\u0007\u0016\u0002\u0017\u0007\u0017\u0002\u0018\u0007\u0018"+
		"\u0002\u0019\u0007\u0019\u0002\u001a\u0007\u001a\u0002\u001b\u0007\u001b"+
		"\u0002\u001c\u0007\u001c\u0002\u001d\u0007\u001d\u0002\u001e\u0007\u001e"+
		"\u0002\u001f\u0007\u001f\u0002 \u0007 \u0002!\u0007!\u0002\"\u0007\"\u0002"+
		"#\u0007#\u0001\u0000\u0001\u0000\u0001\u0000\u0001\u0001\u0001\u0001\u0001"+
		"\u0002\u0001\u0002\u0001\u0003\u0001\u0003\u0001\u0003\u0001\u0003\u0005"+
		"\u0003T\b\u0003\n\u0003\f\u0003W\t\u0003\u0001\u0004\u0001\u0004\u0001"+
		"\u0004\u0001\u0004\u0005\u0004]\b\u0004\n\u0004\f\u0004`\t\u0004\u0001"+
		"\u0005\u0001\u0005\u0001\u0005\u0005\u0005e\b\u0005\n\u0005\f\u0005h\t"+
		"\u0005\u0001\u0006\u0001\u0006\u0001\u0006\u0005\u0006m\b\u0006\n\u0006"+
		"\f\u0006p\t\u0006\u0001\u0007\u0001\u0007\u0001\u0007\u0005\u0007u\b\u0007"+
		"\n\u0007\f\u0007x\t\u0007\u0001\b\u0001\b\u0001\b\u0005\b}\b\b\n\b\f\b"+
		"\u0080\t\b\u0001\t\u0001\t\u0005\t\u0084\b\t\n\t\f\t\u0087\t\t\u0001\n"+
		"\u0001\n\u0005\n\u008b\b\n\n\n\f\n\u008e\t\n\u0001\u000b\u0001\u000b\u0003"+
		"\u000b\u0092\b\u000b\u0001\u000b\u0001\u000b\u0003\u000b\u0096\b\u000b"+
		"\u0001\u000b\u0001\u000b\u0003\u000b\u009a\b\u000b\u0001\f\u0001\f\u0001"+
		"\f\u0001\f\u0001\f\u0001\f\u0001\f\u0001\f\u0003\f\u00a4\b\f\u0001\r\u0001"+
		"\r\u0003\r\u00a8\b\r\u0001\r\u0001\r\u0001\r\u0001\r\u0001\u000e\u0001"+
		"\u000e\u0001\u000e\u0005\u000e\u00b1\b\u000e\n\u000e\f\u000e\u00b4\t\u000e"+
		"\u0001\u000f\u0001\u000f\u0001\u000f\u0001\u000f\u0003\u000f\u00ba\b\u000f"+
		"\u0001\u0010\u0001\u0010\u0005\u0010\u00be\b\u0010\n\u0010\f\u0010\u00c1"+
		"\t\u0010\u0001\u0010\u0001\u0010\u0001\u0011\u0001\u0011\u0003\u0011\u00c7"+
		"\b\u0011\u0001\u0011\u0005\u0011\u00ca\b\u0011\n\u0011\f\u0011\u00cd\t"+
		"\u0011\u0001\u0012\u0001\u0012\u0001\u0012\u0003\u0012\u00d2\b\u0012\u0001"+
		"\u0013\u0001\u0013\u0001\u0013\u0001\u0013\u0001\u0014\u0001\u0014\u0001"+
		"\u0014\u0001\u0014\u0001\u0015\u0001\u0015\u0003\u0015\u00de\b\u0015\u0001"+
		"\u0016\u0001\u0016\u0001\u0016\u0001\u0016\u0001\u0017\u0001\u0017\u0001"+
		"\u0017\u0001\u0017\u0005\u0017\u00e8\b\u0017\n\u0017\f\u0017\u00eb\t\u0017"+
		"\u0003\u0017\u00ed\b\u0017\u0001\u0017\u0001\u0017\u0001\u0018\u0001\u0018"+
		"\u0001\u0018\u0001\u0018\u0003\u0018\u00f5\b\u0018\u0001\u0019\u0001\u0019"+
		"\u0001\u0019\u0001\u0019\u0001\u0019\u0003\u0019\u00fc\b\u0019\u0001\u001a"+
		"\u0001\u001a\u0001\u001a\u0001\u001a\u0001\u001b\u0001\u001b\u0003\u001b"+
		"\u0104\b\u001b\u0001\u001c\u0001\u001c\u0003\u001c\u0108\b\u001c\u0001"+
		"\u001d\u0001\u001d\u0001\u001d\u0003\u001d\u010d\b\u001d\u0001\u001e\u0001"+
		"\u001e\u0001\u001f\u0003\u001f\u0112\b\u001f\u0001\u001f\u0001\u001f\u0001"+
		" \u0003 \u0117\b \u0001 \u0003 \u011a\b \u0001 \u0001 \u0001 \u0001!\u0001"+
		"!\u0005!\u0121\b!\n!\f!\u0124\t!\u0001!\u0001!\u0001\"\u0001\"\u0001\""+
		"\u0001\"\u0001#\u0001#\u0003#\u012e\b#\u0001#\u0000\u0000$\u0000\u0002"+
		"\u0004\u0006\b\n\f\u000e\u0010\u0012\u0014\u0016\u0018\u001a\u001c\u001e"+
		" \"$&(*,.02468:<>@BDF\u0000\u0005\u0001\u0000\t\f\u0001\u0000\r\u000e"+
		"\u0001\u0000\u000f\u0010\u0002\u0000\u001e\u001e))\u0001\u0000  \u0134"+
		"\u0000H\u0001\u0000\u0000\u0000\u0002K\u0001\u0000\u0000\u0000\u0004M"+
		"\u0001\u0000\u0000\u0000\u0006O\u0001\u0000\u0000\u0000\bX\u0001\u0000"+
		"\u0000\u0000\na\u0001\u0000\u0000\u0000\fi\u0001\u0000\u0000\u0000\u000e"+
		"q\u0001\u0000\u0000\u0000\u0010y\u0001\u0000\u0000\u0000\u0012\u0081\u0001"+
		"\u0000\u0000\u0000\u0014\u0088\u0001\u0000\u0000\u0000\u0016\u0099\u0001"+
		"\u0000\u0000\u0000\u0018\u00a3\u0001\u0000\u0000\u0000\u001a\u00a5\u0001"+
		"\u0000\u0000\u0000\u001c\u00ad\u0001\u0000\u0000\u0000\u001e\u00b5\u0001"+
		"\u0000\u0000\u0000 \u00bb\u0001\u0000\u0000\u0000\"\u00c6\u0001\u0000"+
		"\u0000\u0000$\u00d1\u0001\u0000\u0000\u0000&\u00d3\u0001\u0000\u0000\u0000"+
		"(\u00d7\u0001\u0000\u0000\u0000*\u00dd\u0001\u0000\u0000\u0000,\u00df"+
		"\u0001\u0000\u0000\u0000.\u00e3\u0001\u0000\u0000\u00000\u00f0\u0001\u0000"+
		"\u0000\u00002\u00fb\u0001\u0000\u0000\u00004\u00fd\u0001\u0000\u0000\u0000"+
		"6\u0103\u0001\u0000\u0000\u00008\u0107\u0001\u0000\u0000\u0000:\u010c"+
		"\u0001\u0000\u0000\u0000<\u010e\u0001\u0000\u0000\u0000>\u0111\u0001\u0000"+
		"\u0000\u0000@\u0116\u0001\u0000\u0000\u0000B\u011e\u0001\u0000\u0000\u0000"+
		"D\u0127\u0001\u0000\u0000\u0000F\u012d\u0001\u0000\u0000\u0000HI\u0003"+
		"\u0002\u0001\u0000IJ\u0005\u0000\u0000\u0001J\u0001\u0001\u0000\u0000"+
		"\u0000KL\u0003 \u0010\u0000L\u0003\u0001\u0000\u0000\u0000MN\u0003\u0006"+
		"\u0003\u0000N\u0005\u0001\u0000\u0000\u0000OU\u0003\b\u0004\u0000PQ\u0005"+
		"\u0014\u0000\u0000QR\u0005\u0014\u0000\u0000RT\u0003\b\u0004\u0000SP\u0001"+
		"\u0000\u0000\u0000TW\u0001\u0000\u0000\u0000US\u0001\u0000\u0000\u0000"+
		"UV\u0001\u0000\u0000\u0000V\u0007\u0001\u0000\u0000\u0000WU\u0001\u0000"+
		"\u0000\u0000X^\u0003\n\u0005\u0000YZ\u0005\u0015\u0000\u0000Z[\u0005\u0015"+
		"\u0000\u0000[]\u0003\n\u0005\u0000\\Y\u0001\u0000\u0000\u0000]`\u0001"+
		"\u0000\u0000\u0000^\\\u0001\u0000\u0000\u0000^_\u0001\u0000\u0000\u0000"+
		"_\t\u0001\u0000\u0000\u0000`^\u0001\u0000\u0000\u0000af\u0003\f\u0006"+
		"\u0000bc\u0005\b\u0000\u0000ce\u0003\f\u0006\u0000db\u0001\u0000\u0000"+
		"\u0000eh\u0001\u0000\u0000\u0000fd\u0001\u0000\u0000\u0000fg\u0001\u0000"+
		"\u0000\u0000g\u000b\u0001\u0000\u0000\u0000hf\u0001\u0000\u0000\u0000"+
		"in\u0003\u000e\u0007\u0000jk\u0007\u0000\u0000\u0000km\u0003\u000e\u0007"+
		"\u0000lj\u0001\u0000\u0000\u0000mp\u0001\u0000\u0000\u0000nl\u0001\u0000"+
		"\u0000\u0000no\u0001\u0000\u0000\u0000o\r\u0001\u0000\u0000\u0000pn\u0001"+
		"\u0000\u0000\u0000qv\u0003\u0010\b\u0000rs\u0007\u0001\u0000\u0000su\u0003"+
		"\u0010\b\u0000tr\u0001\u0000\u0000\u0000ux\u0001\u0000\u0000\u0000vt\u0001"+
		"\u0000\u0000\u0000vw\u0001\u0000\u0000\u0000w\u000f\u0001\u0000\u0000"+
		"\u0000xv\u0001\u0000\u0000\u0000y~\u0003\u0012\t\u0000z{\u0007\u0002\u0000"+
		"\u0000{}\u0003\u0012\t\u0000|z\u0001\u0000\u0000\u0000}\u0080\u0001\u0000"+
		"\u0000\u0000~|\u0001\u0000\u0000\u0000~\u007f\u0001\u0000\u0000\u0000"+
		"\u007f\u0011\u0001\u0000\u0000\u0000\u0080~\u0001\u0000\u0000\u0000\u0081"+
		"\u0085\u0003\u0014\n\u0000\u0082\u0084\u0003\u0014\n\u0000\u0083\u0082"+
		"\u0001\u0000\u0000\u0000\u0084\u0087\u0001\u0000\u0000\u0000\u0085\u0083"+
		"\u0001\u0000\u0000\u0000\u0085\u0086\u0001\u0000\u0000\u0000\u0086\u0013"+
		"\u0001\u0000\u0000\u0000\u0087\u0085\u0001\u0000\u0000\u0000\u0088\u008c"+
		"\u0003\u0018\f\u0000\u0089\u008b\u0003\u0016\u000b\u0000\u008a\u0089\u0001"+
		"\u0000\u0000\u0000\u008b\u008e\u0001\u0000\u0000\u0000\u008c\u008a\u0001"+
		"\u0000\u0000\u0000\u008c\u008d\u0001\u0000\u0000\u0000\u008d\u0015\u0001"+
		"\u0000\u0000\u0000\u008e\u008c\u0001\u0000\u0000\u0000\u008f\u0091\u0005"+
		"\u0012\u0000\u0000\u0090\u0092\u00036\u001b\u0000\u0091\u0090\u0001\u0000"+
		"\u0000\u0000\u0091\u0092\u0001\u0000\u0000\u0000\u0092\u009a\u0001\u0000"+
		"\u0000\u0000\u0093\u0095\u0005\u0013\u0000\u0000\u0094\u0096\u00036\u001b"+
		"\u0000\u0095\u0094\u0001\u0000\u0000\u0000\u0095\u0096\u0001\u0000\u0000"+
		"\u0000\u0096\u009a\u0001\u0000\u0000\u0000\u0097\u0098\u0005\u0011\u0000"+
		"\u0000\u0098\u009a\u00036\u001b\u0000\u0099\u008f\u0001\u0000\u0000\u0000"+
		"\u0099\u0093\u0001\u0000\u0000\u0000\u0099\u0097\u0001\u0000\u0000\u0000"+
		"\u009a\u0017\u0001\u0000\u0000\u0000\u009b\u00a4\u00038\u001c\u0000\u009c"+
		"\u00a4\u0003F#\u0000\u009d\u00a4\u0003\u001a\r\u0000\u009e\u00a4\u0003"+
		" \u0010\u0000\u009f\u00a0\u0005\u0016\u0000\u0000\u00a0\u00a1\u0003\u0004"+
		"\u0002\u0000\u00a1\u00a2\u0005\u0017\u0000\u0000\u00a2\u00a4\u0001\u0000"+
		"\u0000\u0000\u00a3\u009b\u0001\u0000\u0000\u0000\u00a3\u009c\u0001\u0000"+
		"\u0000\u0000\u00a3\u009d\u0001\u0000\u0000\u0000\u00a3\u009e\u0001\u0000"+
		"\u0000\u0000\u00a3\u009f\u0001\u0000\u0000\u0000\u00a4\u0019\u0001\u0000"+
		"\u0000\u0000\u00a5\u00a7\u0005\u0016\u0000\u0000\u00a6\u00a8\u0003\u001c"+
		"\u000e\u0000\u00a7\u00a6\u0001\u0000\u0000\u0000\u00a7\u00a8\u0001\u0000"+
		"\u0000\u0000\u00a8\u00a9\u0001\u0000\u0000\u0000\u00a9\u00aa\u0005\u0017"+
		"\u0000\u0000\u00aa\u00ab\u0005\u0005\u0000\u0000\u00ab\u00ac\u0003\u0002"+
		"\u0001\u0000\u00ac\u001b\u0001\u0000\u0000\u0000\u00ad\u00b2\u0003\u001e"+
		"\u000f\u0000\u00ae\u00af\u0005\u001c\u0000\u0000\u00af\u00b1\u0003\u001e"+
		"\u000f\u0000\u00b0\u00ae\u0001\u0000\u0000\u0000\u00b1\u00b4\u0001\u0000"+
		"\u0000\u0000\u00b2\u00b0\u0001\u0000\u0000\u0000\u00b2\u00b3\u0001\u0000"+
		"\u0000\u0000\u00b3\u001d\u0001\u0000\u0000\u0000\u00b4\u00b2\u0001\u0000"+
		"\u0000\u0000\u00b5\u00b6\u0003F#\u0000\u00b6\u00b9\u0005\u001d\u0000\u0000"+
		"\u00b7\u00ba\u00032\u0019\u0000\u00b8\u00ba\u00034\u001a\u0000\u00b9\u00b7"+
		"\u0001\u0000\u0000\u0000\u00b9\u00b8\u0001\u0000\u0000\u0000\u00ba\u001f"+
		"\u0001\u0000\u0000\u0000\u00bb\u00bf\u0005\u0018\u0000\u0000\u00bc\u00be"+
		"\u0003\"\u0011\u0000\u00bd\u00bc\u0001\u0000\u0000\u0000\u00be\u00c1\u0001"+
		"\u0000\u0000\u0000\u00bf\u00bd\u0001\u0000\u0000\u0000\u00bf\u00c0\u0001"+
		"\u0000\u0000\u0000\u00c0\u00c2\u0001\u0000\u0000\u0000\u00c1\u00bf\u0001"+
		"\u0000\u0000\u0000\u00c2\u00c3\u0005\u0019\u0000\u0000\u00c3!\u0001\u0000"+
		"\u0000\u0000\u00c4\u00c7\u0003\u0004\u0002\u0000\u00c5\u00c7\u0003$\u0012"+
		"\u0000\u00c6\u00c4\u0001\u0000\u0000\u0000\u00c6\u00c5\u0001\u0000\u0000"+
		"\u0000\u00c7\u00cb\u0001\u0000\u0000\u0000\u00c8\u00ca\u0007\u0003\u0000"+
		"\u0000\u00c9\u00c8\u0001\u0000\u0000\u0000\u00ca\u00cd\u0001\u0000\u0000"+
		"\u0000\u00cb\u00c9\u0001\u0000\u0000\u0000\u00cb\u00cc\u0001\u0000\u0000"+
		"\u0000\u00cc#\u0001\u0000\u0000\u0000\u00cd\u00cb\u0001\u0000\u0000\u0000"+
		"\u00ce\u00d2\u0003&\u0013\u0000\u00cf\u00d2\u0003(\u0014\u0000\u00d0\u00d2"+
		"\u0003,\u0016\u0000\u00d1\u00ce\u0001\u0000\u0000\u0000\u00d1\u00cf\u0001"+
		"\u0000\u0000\u0000\u00d1\u00d0\u0001\u0000\u0000\u0000\u00d2%\u0001\u0000"+
		"\u0000\u0000\u00d3\u00d4\u0003F#\u0000\u00d4\u00d5\u0005\u0006\u0000\u0000"+
		"\u00d5\u00d6\u0003\u0004\u0002\u0000\u00d6\'\u0001\u0000\u0000\u0000\u00d7"+
		"\u00d8\u00034\u001a\u0000\u00d8\u00d9\u0005\u0006\u0000\u0000\u00d9\u00da"+
		"\u0003*\u0015\u0000\u00da)\u0001\u0000\u0000\u0000\u00db\u00de\u00032"+
		"\u0019\u0000\u00dc\u00de\u00034\u001a\u0000\u00dd\u00db\u0001\u0000\u0000"+
		"\u0000\u00dd\u00dc\u0001\u0000\u0000\u0000\u00de+\u0001\u0000\u0000\u0000"+
		"\u00df\u00e0\u0003F#\u0000\u00e0\u00e1\u0005\u0007\u0000\u0000\u00e1\u00e2"+
		"\u0003\u0004\u0002\u0000\u00e2-\u0001\u0000\u0000\u0000\u00e3\u00ec\u0005"+
		"\u001a\u0000\u0000\u00e4\u00e9\u00030\u0018\u0000\u00e5\u00e6\u0005\u001c"+
		"\u0000\u0000\u00e6\u00e8\u00030\u0018\u0000\u00e7\u00e5\u0001\u0000\u0000"+
		"\u0000\u00e8\u00eb\u0001\u0000\u0000\u0000\u00e9\u00e7\u0001\u0000\u0000"+
		"\u0000\u00e9\u00ea\u0001\u0000\u0000\u0000\u00ea\u00ed\u0001\u0000\u0000"+
		"\u0000\u00eb\u00e9\u0001\u0000\u0000\u0000\u00ec\u00e4\u0001\u0000\u0000"+
		"\u0000\u00ec\u00ed\u0001\u0000\u0000\u0000\u00ed\u00ee\u0001\u0000\u0000"+
		"\u0000\u00ee\u00ef\u0005\u001b\u0000\u0000\u00ef/\u0001\u0000\u0000\u0000"+
		"\u00f0\u00f1\u0003F#\u0000\u00f1\u00f4\u0005\u001d\u0000\u0000\u00f2\u00f5"+
		"\u00032\u0019\u0000\u00f3\u00f5\u00034\u001a\u0000\u00f4\u00f2\u0001\u0000"+
		"\u0000\u0000\u00f4\u00f3\u0001\u0000\u0000\u0000\u00f51\u0001\u0000\u0000"+
		"\u0000\u00f6\u00fc\u0005\u0001\u0000\u0000\u00f7\u00fc\u0005\u0002\u0000"+
		"\u0000\u00f8\u00fc\u0005\u0003\u0000\u0000\u00f9\u00fc\u0005\u0004\u0000"+
		"\u0000\u00fa\u00fc\u0003.\u0017\u0000\u00fb\u00f6\u0001\u0000\u0000\u0000"+
		"\u00fb\u00f7\u0001\u0000\u0000\u0000\u00fb\u00f8\u0001\u0000\u0000\u0000"+
		"\u00fb\u00f9\u0001\u0000\u0000\u0000\u00fb\u00fa\u0001\u0000\u0000\u0000"+
		"\u00fc3\u0001\u0000\u0000\u0000\u00fd\u00fe\u0005\"\u0000\u0000\u00fe"+
		"\u00ff\u0005!\u0000\u0000\u00ff\u0100\u0005#\u0000\u0000\u01005\u0001"+
		"\u0000\u0000\u0000\u0101\u0104\u0003F#\u0000\u0102\u0104\u0003>\u001f"+
		"\u0000\u0103\u0101\u0001\u0000\u0000\u0000\u0103\u0102\u0001\u0000\u0000"+
		"\u0000\u01047\u0001\u0000\u0000\u0000\u0105\u0108\u0003:\u001d\u0000\u0106"+
		"\u0108\u0003D\"\u0000\u0107\u0105\u0001\u0000\u0000\u0000\u0107\u0106"+
		"\u0001\u0000\u0000\u0000\u01089\u0001\u0000\u0000\u0000\u0109\u010d\u0003"+
		">\u001f\u0000\u010a\u010d\u0003@ \u0000\u010b\u010d\u0003B!\u0000\u010c"+
		"\u0109\u0001\u0000\u0000\u0000\u010c\u010a\u0001\u0000\u0000\u0000\u010c"+
		"\u010b\u0001\u0000\u0000\u0000\u010d;\u0001\u0000\u0000\u0000\u010e\u010f"+
		"\u0007\u0001\u0000\u0000\u010f=\u0001\u0000\u0000\u0000\u0110\u0112\u0003"+
		"<\u001e\u0000\u0111\u0110\u0001\u0000\u0000\u0000\u0111\u0112\u0001\u0000"+
		"\u0000\u0000\u0112\u0113\u0001\u0000\u0000\u0000\u0113\u0114\u0005%\u0000"+
		"\u0000\u0114?\u0001\u0000\u0000\u0000\u0115\u0117\u0003<\u001e\u0000\u0116"+
		"\u0115\u0001\u0000\u0000\u0000\u0116\u0117\u0001\u0000\u0000\u0000\u0117"+
		"\u0119\u0001\u0000\u0000\u0000\u0118\u011a\u0005%\u0000\u0000\u0119\u0118"+
		"\u0001\u0000\u0000\u0000\u0119\u011a\u0001\u0000\u0000\u0000\u011a\u011b"+
		"\u0001\u0000\u0000\u0000\u011b\u011c\u0005\u001f\u0000\u0000\u011c\u011d"+
		"\u0005%\u0000\u0000\u011dA\u0001\u0000\u0000\u0000\u011e\u0122\u0005 "+
		"\u0000\u0000\u011f\u0121\b\u0004\u0000\u0000\u0120\u011f\u0001\u0000\u0000"+
		"\u0000\u0121\u0124\u0001\u0000\u0000\u0000\u0122\u0120\u0001\u0000\u0000"+
		"\u0000\u0122\u0123\u0001\u0000\u0000\u0000\u0123\u0125\u0001\u0000\u0000"+
		"\u0000\u0124\u0122\u0001\u0000\u0000\u0000\u0125\u0126\u0005 \u0000\u0000"+
		"\u0126C\u0001\u0000\u0000\u0000\u0127\u0128\u0005\"\u0000\u0000\u0128"+
		"\u0129\u0005!\u0000\u0000\u0129\u012a\u0003:\u001d\u0000\u012aE\u0001"+
		"\u0000\u0000\u0000\u012b\u012e\u0005#\u0000\u0000\u012c\u012e\u0005$\u0000"+
		"\u0000\u012d\u012b\u0001\u0000\u0000\u0000\u012d\u012c\u0001\u0000\u0000"+
		"\u0000\u012eG\u0001\u0000\u0000\u0000 U^fnv~\u0085\u008c\u0091\u0095\u0099"+
		"\u00a3\u00a7\u00b2\u00b9\u00bf\u00c6\u00cb\u00d1\u00dd\u00e9\u00ec\u00f4"+
		"\u00fb\u0103\u0107\u010c\u0111\u0116\u0119\u0122\u012d";
	public static final ATN _ATN =
		new ATNDeserializer().deserialize(_serializedATN.toCharArray());
	static {
		_decisionToDFA = new DFA[_ATN.getNumberOfDecisions()];
		for (int i = 0; i < _ATN.getNumberOfDecisions(); i++) {
			_decisionToDFA[i] = new DFA(_ATN.getDecisionState(i), i);
		}
	}
}