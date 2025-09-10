// Generated from /home/user/foolish/src/main/antlr4/Foolish.g4 by ANTLR 4.13.1
import org.antlr.v4.runtime.Lexer;
import org.antlr.v4.runtime.CharStream;
import org.antlr.v4.runtime.Token;
import org.antlr.v4.runtime.TokenStream;
import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.atn.*;
import org.antlr.v4.runtime.dfa.DFA;
import org.antlr.v4.runtime.misc.*;

@SuppressWarnings({"all", "warnings", "unchecked", "unused", "cast", "CheckReturnValue", "this-escape"})
public class FoolishLexer extends Lexer {
	static { RuntimeMetaData.checkVersion("4.13.1", RuntimeMetaData.VERSION); }

	protected static final DFA[] _decisionToDFA;
	protected static final PredictionContextCache _sharedContextCache =
		new PredictionContextCache();
	public static final int
		LBRACE=1, RBRACE=2, LPAREN=3, RPAREN=4, SEMI=5, LINE_COMMENT=6, BLOCK_COMMENT=7, 
		ASSIGN=8, PLUS=9, MINUS=10, MUL=11, DIV=12, IF=13, THEN=14, ELIF=15, ELSE=16, 
		UNKNOWN=17, INTEGER=18, WS=19, IDENTIFIER=20, APOSTROPHE=21;
	public static String[] channelNames = {
		"DEFAULT_TOKEN_CHANNEL", "HIDDEN"
	};

	public static String[] modeNames = {
		"DEFAULT_MODE"
	};

	private static String[] makeRuleNames() {
		return new String[] {
			"LBRACE", "RBRACE", "LPAREN", "RPAREN", "SEMI", "LINE_COMMENT", "BLOCK_COMMENT", 
			"ASSIGN", "PLUS", "MINUS", "MUL", "DIV", "IF", "THEN", "ELIF", "ELSE", 
			"UNKNOWN", "INTEGER", "LETTER", "DIGIT", "UNDERSCORE", "WS", "IDENTIFIER", 
			"APOSTROPHE"
		};
	}
	public static final String[] ruleNames = makeRuleNames();

	private static String[] makeLiteralNames() {
		return new String[] {
			null, "'{'", "'}'", "'('", "')'", "';'", null, null, "'='", "'+'", "'-'", 
			"'*'", "'/'", "'if'", "'then'", "'elif'", "'else'", "'???'", null, null, 
			null, "'''"
		};
	}
	private static final String[] _LITERAL_NAMES = makeLiteralNames();
	private static String[] makeSymbolicNames() {
		return new String[] {
			null, "LBRACE", "RBRACE", "LPAREN", "RPAREN", "SEMI", "LINE_COMMENT", 
			"BLOCK_COMMENT", "ASSIGN", "PLUS", "MINUS", "MUL", "DIV", "IF", "THEN", 
			"ELIF", "ELSE", "UNKNOWN", "INTEGER", "WS", "IDENTIFIER", "APOSTROPHE"
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


	public FoolishLexer(CharStream input) {
		super(input);
		_interp = new LexerATNSimulator(this,_ATN,_decisionToDFA,_sharedContextCache);
	}

	@Override
	public String getGrammarFileName() { return "Foolish.g4"; }

	@Override
	public String[] getRuleNames() { return ruleNames; }

	@Override
	public String getSerializedATN() { return _serializedATN; }

	@Override
	public String[] getChannelNames() { return channelNames; }

	@Override
	public String[] getModeNames() { return modeNames; }

	@Override
	public ATN getATN() { return _ATN; }

	public static final String _serializedATN =
		"\u0004\u0000\u0015\u0090\u0006\uffff\uffff\u0002\u0000\u0007\u0000\u0002"+
		"\u0001\u0007\u0001\u0002\u0002\u0007\u0002\u0002\u0003\u0007\u0003\u0002"+
		"\u0004\u0007\u0004\u0002\u0005\u0007\u0005\u0002\u0006\u0007\u0006\u0002"+
		"\u0007\u0007\u0007\u0002\b\u0007\b\u0002\t\u0007\t\u0002\n\u0007\n\u0002"+
		"\u000b\u0007\u000b\u0002\f\u0007\f\u0002\r\u0007\r\u0002\u000e\u0007\u000e"+
		"\u0002\u000f\u0007\u000f\u0002\u0010\u0007\u0010\u0002\u0011\u0007\u0011"+
		"\u0002\u0012\u0007\u0012\u0002\u0013\u0007\u0013\u0002\u0014\u0007\u0014"+
		"\u0002\u0015\u0007\u0015\u0002\u0016\u0007\u0016\u0002\u0017\u0007\u0017"+
		"\u0001\u0000\u0001\u0000\u0001\u0001\u0001\u0001\u0001\u0002\u0001\u0002"+
		"\u0001\u0003\u0001\u0003\u0001\u0004\u0001\u0004\u0001\u0005\u0001\u0005"+
		"\u0005\u0005>\b\u0005\n\u0005\f\u0005A\t\u0005\u0001\u0005\u0001\u0005"+
		"\u0001\u0006\u0001\u0006\u0001\u0006\u0001\u0006\u0001\u0006\u0005\u0006"+
		"J\b\u0006\n\u0006\f\u0006M\t\u0006\u0001\u0006\u0001\u0006\u0001\u0006"+
		"\u0001\u0006\u0001\u0006\u0001\u0007\u0001\u0007\u0001\b\u0001\b\u0001"+
		"\t\u0001\t\u0001\n\u0001\n\u0001\u000b\u0001\u000b\u0001\f\u0001\f\u0001"+
		"\f\u0001\r\u0001\r\u0001\r\u0001\r\u0001\r\u0001\u000e\u0001\u000e\u0001"+
		"\u000e\u0001\u000e\u0001\u000e\u0001\u000f\u0001\u000f\u0001\u000f\u0001"+
		"\u000f\u0001\u000f\u0001\u0010\u0001\u0010\u0001\u0010\u0001\u0010\u0001"+
		"\u0011\u0004\u0011u\b\u0011\u000b\u0011\f\u0011v\u0001\u0012\u0001\u0012"+
		"\u0001\u0013\u0001\u0013\u0001\u0014\u0001\u0014\u0001\u0015\u0004\u0015"+
		"\u0080\b\u0015\u000b\u0015\f\u0015\u0081\u0001\u0015\u0001\u0015\u0001"+
		"\u0016\u0001\u0016\u0001\u0016\u0001\u0016\u0005\u0016\u008a\b\u0016\n"+
		"\u0016\f\u0016\u008d\t\u0016\u0001\u0017\u0001\u0017\u0001K\u0000\u0018"+
		"\u0001\u0001\u0003\u0002\u0005\u0003\u0007\u0004\t\u0005\u000b\u0006\r"+
		"\u0007\u000f\b\u0011\t\u0013\n\u0015\u000b\u0017\f\u0019\r\u001b\u000e"+
		"\u001d\u000f\u001f\u0010!\u0011#\u0012%\u0000\'\u0000)\u0000+\u0013-\u0014"+
		"/\u0015\u0001\u0000\u0004\u0002\u0000\n\n\r\r\u0002\u0000AZaz\u0001\u0000"+
		"09\u0003\u0000\t\n\r\r  \u0094\u0000\u0001\u0001\u0000\u0000\u0000\u0000"+
		"\u0003\u0001\u0000\u0000\u0000\u0000\u0005\u0001\u0000\u0000\u0000\u0000"+
		"\u0007\u0001\u0000\u0000\u0000\u0000\t\u0001\u0000\u0000\u0000\u0000\u000b"+
		"\u0001\u0000\u0000\u0000\u0000\r\u0001\u0000\u0000\u0000\u0000\u000f\u0001"+
		"\u0000\u0000\u0000\u0000\u0011\u0001\u0000\u0000\u0000\u0000\u0013\u0001"+
		"\u0000\u0000\u0000\u0000\u0015\u0001\u0000\u0000\u0000\u0000\u0017\u0001"+
		"\u0000\u0000\u0000\u0000\u0019\u0001\u0000\u0000\u0000\u0000\u001b\u0001"+
		"\u0000\u0000\u0000\u0000\u001d\u0001\u0000\u0000\u0000\u0000\u001f\u0001"+
		"\u0000\u0000\u0000\u0000!\u0001\u0000\u0000\u0000\u0000#\u0001\u0000\u0000"+
		"\u0000\u0000+\u0001\u0000\u0000\u0000\u0000-\u0001\u0000\u0000\u0000\u0000"+
		"/\u0001\u0000\u0000\u0000\u00011\u0001\u0000\u0000\u0000\u00033\u0001"+
		"\u0000\u0000\u0000\u00055\u0001\u0000\u0000\u0000\u00077\u0001\u0000\u0000"+
		"\u0000\t9\u0001\u0000\u0000\u0000\u000b;\u0001\u0000\u0000\u0000\rD\u0001"+
		"\u0000\u0000\u0000\u000fS\u0001\u0000\u0000\u0000\u0011U\u0001\u0000\u0000"+
		"\u0000\u0013W\u0001\u0000\u0000\u0000\u0015Y\u0001\u0000\u0000\u0000\u0017"+
		"[\u0001\u0000\u0000\u0000\u0019]\u0001\u0000\u0000\u0000\u001b`\u0001"+
		"\u0000\u0000\u0000\u001de\u0001\u0000\u0000\u0000\u001fj\u0001\u0000\u0000"+
		"\u0000!o\u0001\u0000\u0000\u0000#t\u0001\u0000\u0000\u0000%x\u0001\u0000"+
		"\u0000\u0000\'z\u0001\u0000\u0000\u0000)|\u0001\u0000\u0000\u0000+\u007f"+
		"\u0001\u0000\u0000\u0000-\u0085\u0001\u0000\u0000\u0000/\u008e\u0001\u0000"+
		"\u0000\u000012\u0005{\u0000\u00002\u0002\u0001\u0000\u0000\u000034\u0005"+
		"}\u0000\u00004\u0004\u0001\u0000\u0000\u000056\u0005(\u0000\u00006\u0006"+
		"\u0001\u0000\u0000\u000078\u0005)\u0000\u00008\b\u0001\u0000\u0000\u0000"+
		"9:\u0005;\u0000\u0000:\n\u0001\u0000\u0000\u0000;?\u0005!\u0000\u0000"+
		"<>\b\u0000\u0000\u0000=<\u0001\u0000\u0000\u0000>A\u0001\u0000\u0000\u0000"+
		"?=\u0001\u0000\u0000\u0000?@\u0001\u0000\u0000\u0000@B\u0001\u0000\u0000"+
		"\u0000A?\u0001\u0000\u0000\u0000BC\u0006\u0005\u0000\u0000C\f\u0001\u0000"+
		"\u0000\u0000DE\u0005!\u0000\u0000EF\u0005!\u0000\u0000FK\u0001\u0000\u0000"+
		"\u0000GJ\t\u0000\u0000\u0000HJ\u0007\u0000\u0000\u0000IG\u0001\u0000\u0000"+
		"\u0000IH\u0001\u0000\u0000\u0000JM\u0001\u0000\u0000\u0000KL\u0001\u0000"+
		"\u0000\u0000KI\u0001\u0000\u0000\u0000LN\u0001\u0000\u0000\u0000MK\u0001"+
		"\u0000\u0000\u0000NO\u0005!\u0000\u0000OP\u0005!\u0000\u0000PQ\u0001\u0000"+
		"\u0000\u0000QR\u0006\u0006\u0000\u0000R\u000e\u0001\u0000\u0000\u0000"+
		"ST\u0005=\u0000\u0000T\u0010\u0001\u0000\u0000\u0000UV\u0005+\u0000\u0000"+
		"V\u0012\u0001\u0000\u0000\u0000WX\u0005-\u0000\u0000X\u0014\u0001\u0000"+
		"\u0000\u0000YZ\u0005*\u0000\u0000Z\u0016\u0001\u0000\u0000\u0000[\\\u0005"+
		"/\u0000\u0000\\\u0018\u0001\u0000\u0000\u0000]^\u0005i\u0000\u0000^_\u0005"+
		"f\u0000\u0000_\u001a\u0001\u0000\u0000\u0000`a\u0005t\u0000\u0000ab\u0005"+
		"h\u0000\u0000bc\u0005e\u0000\u0000cd\u0005n\u0000\u0000d\u001c\u0001\u0000"+
		"\u0000\u0000ef\u0005e\u0000\u0000fg\u0005l\u0000\u0000gh\u0005i\u0000"+
		"\u0000hi\u0005f\u0000\u0000i\u001e\u0001\u0000\u0000\u0000jk\u0005e\u0000"+
		"\u0000kl\u0005l\u0000\u0000lm\u0005s\u0000\u0000mn\u0005e\u0000\u0000"+
		"n \u0001\u0000\u0000\u0000op\u0005?\u0000\u0000pq\u0005?\u0000\u0000q"+
		"r\u0005?\u0000\u0000r\"\u0001\u0000\u0000\u0000su\u0003\'\u0013\u0000"+
		"ts\u0001\u0000\u0000\u0000uv\u0001\u0000\u0000\u0000vt\u0001\u0000\u0000"+
		"\u0000vw\u0001\u0000\u0000\u0000w$\u0001\u0000\u0000\u0000xy\u0007\u0001"+
		"\u0000\u0000y&\u0001\u0000\u0000\u0000z{\u0007\u0002\u0000\u0000{(\u0001"+
		"\u0000\u0000\u0000|}\u0005_\u0000\u0000}*\u0001\u0000\u0000\u0000~\u0080"+
		"\u0007\u0003\u0000\u0000\u007f~\u0001\u0000\u0000\u0000\u0080\u0081\u0001"+
		"\u0000\u0000\u0000\u0081\u007f\u0001\u0000\u0000\u0000\u0081\u0082\u0001"+
		"\u0000\u0000\u0000\u0082\u0083\u0001\u0000\u0000\u0000\u0083\u0084\u0006"+
		"\u0015\u0000\u0000\u0084,\u0001\u0000\u0000\u0000\u0085\u008b\u0003%\u0012"+
		"\u0000\u0086\u008a\u0003%\u0012\u0000\u0087\u008a\u0003\'\u0013\u0000"+
		"\u0088\u008a\u0003)\u0014\u0000\u0089\u0086\u0001\u0000\u0000\u0000\u0089"+
		"\u0087\u0001\u0000\u0000\u0000\u0089\u0088\u0001\u0000\u0000\u0000\u008a"+
		"\u008d\u0001\u0000\u0000\u0000\u008b\u0089\u0001\u0000\u0000\u0000\u008b"+
		"\u008c\u0001\u0000\u0000\u0000\u008c.\u0001\u0000\u0000\u0000\u008d\u008b"+
		"\u0001\u0000\u0000\u0000\u008e\u008f\u0005\'\u0000\u0000\u008f0\u0001"+
		"\u0000\u0000\u0000\b\u0000?IKv\u0081\u0089\u008b\u0001\u0006\u0000\u0000";
	public static final ATN _ATN =
		new ATNDeserializer().deserialize(_serializedATN.toCharArray());
	static {
		_decisionToDFA = new DFA[_ATN.getNumberOfDecisions()];
		for (int i = 0; i < _ATN.getNumberOfDecisions(); i++) {
			_decisionToDFA[i] = new DFA(_ATN.getDecisionState(i), i);
		}
	}
}