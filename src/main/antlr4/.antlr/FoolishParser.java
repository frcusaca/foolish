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
    public static final int
            LBRACE = 1, RBRACE = 2, LPAREN = 3, RPAREN = 4, SEMI = 5, LINE_COMMENT = 6, BLOCK_COMMENT = 7,
            ASSIGN = 8, PLUS = 9, MINUS = 10, MUL = 11, DIV = 12, IF = 13, THEN = 14, ELIF = 15, ELSE = 16,
            UNKNOWN = 17, INTEGER = 18, WS = 19, IDENTIFIER = 20, APOSTROPHE = 21;
    public static final int
            RULE_program = 0, RULE_characterizable = 1, RULE_brane = 2, RULE_branes = 3,
            RULE_stmt = 4, RULE_assignment = 5, RULE_expr = 6, RULE_addExpr = 7, RULE_mulExpr = 8,
            RULE_unaryExpr = 9, RULE_literal = 10, RULE_primary = 11, RULE_ifExpr = 12,
            RULE_ifExprHelperIf = 13, RULE_ifExprHelperElif = 14, RULE_ifExprHelperElse = 15;
    public static final String[] ruleNames = makeRuleNames();
    /**
     * @deprecated Use {@link #VOCABULARY} instead.
     */
    @Deprecated
    public static final String[] tokenNames;
    public static final String _serializedATN =
            "\u0004\u0001\u0015\u008b\u0002\u0000\u0007\u0000\u0002\u0001\u0007\u0001" +
                    "\u0002\u0002\u0007\u0002\u0002\u0003\u0007\u0003\u0002\u0004\u0007\u0004" +
                    "\u0002\u0005\u0007\u0005\u0002\u0006\u0007\u0006\u0002\u0007\u0007\u0007" +
                    "\u0002\b\u0007\b\u0002\t\u0007\t\u0002\n\u0007\n\u0002\u000b\u0007\u000b" +
                    "\u0002\f\u0007\f\u0002\r\u0007\r\u0002\u000e\u0007\u000e\u0002\u000f\u0007" +
                    "\u000f\u0001\u0000\u0001\u0000\u0001\u0000\u0001\u0001\u0003\u0001%\b" +
                    "\u0001\u0001\u0001\u0003\u0001(\b\u0001\u0001\u0001\u0001\u0001\u0001" +
                    "\u0001\u0003\u0001-\b\u0001\u0001\u0002\u0001\u0002\u0005\u00021\b\u0002" +
                    "\n\u0002\f\u00024\t\u0002\u0001\u0002\u0001\u0002\u0001\u0003\u0004\u0003" +
                    "9\b\u0003\u000b\u0003\f\u0003:\u0001\u0004\u0001\u0004\u0003\u0004?\b" +
                    "\u0004\u0001\u0004\u0001\u0004\u0001\u0004\u0003\u0004D\b\u0004\u0001" +
                    "\u0004\u0001\u0004\u0001\u0004\u0003\u0004I\b\u0004\u0003\u0004K\b\u0004" +
                    "\u0001\u0005\u0001\u0005\u0001\u0005\u0001\u0005\u0001\u0006\u0001\u0006" +
                    "\u0003\u0006S\b\u0006\u0001\u0007\u0001\u0007\u0001\u0007\u0005\u0007" +
                    "X\b\u0007\n\u0007\f\u0007[\t\u0007\u0001\b\u0001\b\u0001\b\u0005\b`\b" +
                    "\b\n\b\f\bc\t\b\u0001\t\u0003\tf\b\t\u0001\t\u0001\t\u0001\n\u0001\n\u0001" +
                    "\u000b\u0001\u000b\u0001\u000b\u0001\u000b\u0001\u000b\u0001\u000b\u0003" +
                    "\u000br\b\u000b\u0001\f\u0001\f\u0005\fv\b\f\n\f\f\fy\t\f\u0001\f\u0003" +
                    "\f|\b\f\u0001\r\u0001\r\u0001\r\u0001\r\u0001\r\u0001\u000e\u0001\u000e" +
                    "\u0001\u000e\u0001\u000e\u0001\u000e\u0001\u000f\u0001\u000f\u0001\u000f" +
                    "\u0001\u000f\u0000\u0000\u0010\u0000\u0002\u0004\u0006\b\n\f\u000e\u0010" +
                    "\u0012\u0014\u0016\u0018\u001a\u001c\u001e\u0000\u0003\u0001\u0000\t\n" +
                    "\u0001\u0000\u000b\f\u0001\u0000\t\u000b\u008d\u0000 \u0001\u0000\u0000" +
                    "\u0000\u0002\'\u0001\u0000\u0000\u0000\u0004.\u0001\u0000\u0000\u0000" +
                    "\u00068\u0001\u0000\u0000\u0000\bJ\u0001\u0000\u0000\u0000\nL\u0001\u0000" +
                    "\u0000\u0000\fR\u0001\u0000\u0000\u0000\u000eT\u0001\u0000\u0000\u0000" +
                    "\u0010\\\u0001\u0000\u0000\u0000\u0012e\u0001\u0000\u0000\u0000\u0014" +
                    "i\u0001\u0000\u0000\u0000\u0016q\u0001\u0000\u0000\u0000\u0018s\u0001" +
                    "\u0000\u0000\u0000\u001a}\u0001\u0000\u0000\u0000\u001c\u0082\u0001\u0000" +
                    "\u0000\u0000\u001e\u0087\u0001\u0000\u0000\u0000 !\u0003\u0006\u0003\u0000" +
                    "!\"\u0005\u0000\u0000\u0001\"\u0001\u0001\u0000\u0000\u0000#%\u0005\u0014" +
                    "\u0000\u0000$#\u0001\u0000\u0000\u0000$%\u0001\u0000\u0000\u0000%&\u0001" +
                    "\u0000\u0000\u0000&(\u0005\u0015\u0000\u0000\'$\u0001\u0000\u0000\u0000" +
                    "\'(\u0001\u0000\u0000\u0000(,\u0001\u0000\u0000\u0000)-\u0003\u0014\n" +
                    "\u0000*-\u0005\u0014\u0000\u0000+-\u0003\u0004\u0002\u0000,)\u0001\u0000" +
                    "\u0000\u0000,*\u0001\u0000\u0000\u0000,+\u0001\u0000\u0000\u0000-\u0003" +
                    "\u0001\u0000\u0000\u0000.2\u0005\u0001\u0000\u0000/1\u0003\b\u0004\u0000" +
                    "0/\u0001\u0000\u0000\u000014\u0001\u0000\u0000\u000020\u0001\u0000\u0000" +
                    "\u000023\u0001\u0000\u0000\u000035\u0001\u0000\u0000\u000042\u0001\u0000" +
                    "\u0000\u000056\u0005\u0002\u0000\u00006\u0005\u0001\u0000\u0000\u0000" +
                    "79\u0003\u0004\u0002\u000087\u0001\u0000\u0000\u00009:\u0001\u0000\u0000" +
                    "\u0000:8\u0001\u0000\u0000\u0000:;\u0001\u0000\u0000\u0000;\u0007\u0001" +
                    "\u0000\u0000\u0000<>\u0003\u0004\u0002\u0000=?\u0005\u0005\u0000\u0000" +
                    ">=\u0001\u0000\u0000\u0000>?\u0001\u0000\u0000\u0000?K\u0001\u0000\u0000" +
                    "\u0000@D\u0003\u0006\u0003\u0000AD\u0003\n\u0005\u0000BD\u0003\f\u0006" +
                    "\u0000C@\u0001\u0000\u0000\u0000CA\u0001\u0000\u0000\u0000CB\u0001\u0000" +
                    "\u0000\u0000DE\u0001\u0000\u0000\u0000EF\u0005\u0005\u0000\u0000FH\u0001" +
                    "\u0000\u0000\u0000GI\u0005\u0006\u0000\u0000HG\u0001\u0000\u0000\u0000" +
                    "HI\u0001\u0000\u0000\u0000IK\u0001\u0000\u0000\u0000J<\u0001\u0000\u0000" +
                    "\u0000JC\u0001\u0000\u0000\u0000K\t\u0001\u0000\u0000\u0000LM\u0005\u0014" +
                    "\u0000\u0000MN\u0005\b\u0000\u0000NO\u0003\f\u0006\u0000O\u000b\u0001" +
                    "\u0000\u0000\u0000PS\u0003\u000e\u0007\u0000QS\u0003\u0018\f\u0000RP\u0001" +
                    "\u0000\u0000\u0000RQ\u0001\u0000\u0000\u0000S\r\u0001\u0000\u0000\u0000" +
                    "TY\u0003\u0010\b\u0000UV\u0007\u0000\u0000\u0000VX\u0003\u0010\b\u0000" +
                    "WU\u0001\u0000\u0000\u0000X[\u0001\u0000\u0000\u0000YW\u0001\u0000\u0000" +
                    "\u0000YZ\u0001\u0000\u0000\u0000Z\u000f\u0001\u0000\u0000\u0000[Y\u0001" +
                    "\u0000\u0000\u0000\\a\u0003\u0012\t\u0000]^\u0007\u0001\u0000\u0000^`" +
                    "\u0003\u0012\t\u0000_]\u0001\u0000\u0000\u0000`c\u0001\u0000\u0000\u0000" +
                    "a_\u0001\u0000\u0000\u0000ab\u0001\u0000\u0000\u0000b\u0011\u0001\u0000" +
                    "\u0000\u0000ca\u0001\u0000\u0000\u0000df\u0007\u0002\u0000\u0000ed\u0001" +
                    "\u0000\u0000\u0000ef\u0001\u0000\u0000\u0000fg\u0001\u0000\u0000\u0000" +
                    "gh\u0003\u0016\u000b\u0000h\u0013\u0001\u0000\u0000\u0000ij\u0005\u0012" +
                    "\u0000\u0000j\u0015\u0001\u0000\u0000\u0000kr\u0003\u0002\u0001\u0000" +
                    "lm\u0005\u0003\u0000\u0000mn\u0003\f\u0006\u0000no\u0005\u0004\u0000\u0000" +
                    "or\u0001\u0000\u0000\u0000pr\u0005\u0011\u0000\u0000qk\u0001\u0000\u0000" +
                    "\u0000ql\u0001\u0000\u0000\u0000qp\u0001\u0000\u0000\u0000r\u0017\u0001" +
                    "\u0000\u0000\u0000sw\u0003\u001a\r\u0000tv\u0003\u001c\u000e\u0000ut\u0001" +
                    "\u0000\u0000\u0000vy\u0001\u0000\u0000\u0000wu\u0001\u0000\u0000\u0000" +
                    "wx\u0001\u0000\u0000\u0000x{\u0001\u0000\u0000\u0000yw\u0001\u0000\u0000" +
                    "\u0000z|\u0003\u001e\u000f\u0000{z\u0001\u0000\u0000\u0000{|\u0001\u0000" +
                    "\u0000\u0000|\u0019\u0001\u0000\u0000\u0000}~\u0005\r\u0000\u0000~\u007f" +
                    "\u0003\f\u0006\u0000\u007f\u0080\u0005\u000e\u0000\u0000\u0080\u0081\u0003" +
                    "\f\u0006\u0000\u0081\u001b\u0001\u0000\u0000\u0000\u0082\u0083\u0005\u000f" +
                    "\u0000\u0000\u0083\u0084\u0003\f\u0006\u0000\u0084\u0085\u0005\u000e\u0000" +
                    "\u0000\u0085\u0086\u0003\f\u0006\u0000\u0086\u001d\u0001\u0000\u0000\u0000" +
                    "\u0087\u0088\u0005\u0010\u0000\u0000\u0088\u0089\u0003\f\u0006\u0000\u0089" +
                    "\u001f\u0001\u0000\u0000\u0000\u0010$\',2:>CHJRYaeqw{";
    public static final ATN _ATN =
            new ATNDeserializer().deserialize(_serializedATN.toCharArray());
    protected static final DFA[] _decisionToDFA;
    protected static final PredictionContextCache _sharedContextCache =
            new PredictionContextCache();
    private static final String[] _LITERAL_NAMES = makeLiteralNames();
    private static final String[] _SYMBOLIC_NAMES = makeSymbolicNames();
    public static final Vocabulary VOCABULARY = new VocabularyImpl(_LITERAL_NAMES, _SYMBOLIC_NAMES);

    static {
        RuntimeMetaData.checkVersion("4.13.1", RuntimeMetaData.VERSION);
    }

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

    static {
        _decisionToDFA = new DFA[_ATN.getNumberOfDecisions()];
        for (int i = 0; i < _ATN.getNumberOfDecisions(); i++) {
            _decisionToDFA[i] = new DFA(_ATN.getDecisionState(i), i);
        }
    }

    public FoolishParser(TokenStream input) {
        super(input);
        _interp = new ParserATNSimulator(this, _ATN, _decisionToDFA, _sharedContextCache);
    }

    private static String[] makeRuleNames() {
        return new String[]{
                "program", "characterizable", "brane", "branes", "stmt", "assignment",
                "expr", "addExpr", "mulExpr", "unaryExpr", "literal", "primary", "ifExpr",
                "ifExprHelperIf", "ifExprHelperElif", "ifExprHelperElse"
        };
    }

    private static String[] makeLiteralNames() {
        return new String[]{
                null, "'{'", "'}'", "'('", "')'", "';'", null, null, "'='", "'+'", "'-'",
                "'*'", "'/'", "'if'", "'then'", "'elif'", "'else'", "'???'", null, null,
                null, "'''"
        };
    }

    private static String[] makeSymbolicNames() {
        return new String[]{
                null, "LBRACE", "RBRACE", "LPAREN", "RPAREN", "SEMI", "LINE_COMMENT",
                "BLOCK_COMMENT", "ASSIGN", "PLUS", "MINUS", "MUL", "DIV", "IF", "THEN",
                "ELIF", "ELSE", "UNKNOWN", "INTEGER", "WS", "IDENTIFIER", "APOSTROPHE"
        };
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
    public String getGrammarFileName() {
        return "Foolish.g4";
    }

    @Override
    public String[] getRuleNames() {
        return ruleNames;
    }

    @Override
    public String getSerializedATN() {
        return _serializedATN;
    }

    @Override
    public ATN getATN() {
        return _ATN;
    }

    public final ProgramContext program() throws RecognitionException {
        ProgramContext _localctx = new ProgramContext(_ctx, getState());
        enterRule(_localctx, 0, RULE_program);
        try {
            enterOuterAlt(_localctx, 1);
            {
                setState(32);
                branes();
                setState(33);
                match(EOF);
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final CharacterizableContext characterizable() throws RecognitionException {
        CharacterizableContext _localctx = new CharacterizableContext(_ctx, getState());
        enterRule(_localctx, 2, RULE_characterizable);
        int _la;
        try {
            enterOuterAlt(_localctx, 1);
            {
                setState(39);
                _errHandler.sync(this);
                switch (getInterpreter().adaptivePredict(_input, 1, _ctx)) {
                    case 1: {
                        setState(36);
                        _errHandler.sync(this);
                        _la = _input.LA(1);
                        if (_la == IDENTIFIER) {
                            {
                                setState(35);
                                match(IDENTIFIER);
                            }
                        }

                        setState(38);
                        match(APOSTROPHE);
                    }
                    break;
                }
                setState(44);
                _errHandler.sync(this);
                switch (_input.LA(1)) {
                    case INTEGER: {
                        setState(41);
                        literal();
                    }
                    break;
                    case IDENTIFIER: {
                        setState(42);
                        match(IDENTIFIER);
                    }
                    break;
                    case LBRACE: {
                        setState(43);
                        brane();
                    }
                    break;
                    default:
                        throw new NoViableAltException(this);
                }
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final BraneContext brane() throws RecognitionException {
        BraneContext _localctx = new BraneContext(_ctx, getState());
        enterRule(_localctx, 4, RULE_brane);
        int _la;
        try {
            enterOuterAlt(_localctx, 1);
            {
                setState(46);
                match(LBRACE);
                setState(50);
                _errHandler.sync(this);
                _la = _input.LA(1);
                while ((((_la) & ~0x3f) == 0 && ((1L << _la) & 3550730L) != 0)) {
                    {
                        {
                            setState(47);
                            stmt();
                        }
                    }
                    setState(52);
                    _errHandler.sync(this);
                    _la = _input.LA(1);
                }
                setState(53);
                match(RBRACE);
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final BranesContext branes() throws RecognitionException {
        BranesContext _localctx = new BranesContext(_ctx, getState());
        enterRule(_localctx, 6, RULE_branes);
        int _la;
        try {
            enterOuterAlt(_localctx, 1);
            {
                setState(56);
                _errHandler.sync(this);
                _la = _input.LA(1);
                do {
                    {
                        {
                            setState(55);
                            brane();
                        }
                    }
                    setState(58);
                    _errHandler.sync(this);
                    _la = _input.LA(1);
                } while (_la == LBRACE);
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final StmtContext stmt() throws RecognitionException {
        StmtContext _localctx = new StmtContext(_ctx, getState());
        enterRule(_localctx, 8, RULE_stmt);
        int _la;
        try {
            setState(74);
            _errHandler.sync(this);
            switch (getInterpreter().adaptivePredict(_input, 8, _ctx)) {
                case 1:
                    enterOuterAlt(_localctx, 1);
                {
                    {
                        setState(60);
                        brane();
                        setState(62);
                        _errHandler.sync(this);
                        _la = _input.LA(1);
                        if (_la == SEMI) {
                            {
                                setState(61);
                                match(SEMI);
                            }
                        }

                    }
                }
                break;
                case 2:
                    enterOuterAlt(_localctx, 2);
                {
                    {
                        setState(67);
                        _errHandler.sync(this);
                        switch (getInterpreter().adaptivePredict(_input, 6, _ctx)) {
                            case 1: {
                                setState(64);
                                branes();
                            }
                            break;
                            case 2: {
                                setState(65);
                                assignment();
                            }
                            break;
                            case 3: {
                                setState(66);
                                expr();
                            }
                            break;
                        }
                        setState(69);
                        match(SEMI);
                    }
                    setState(72);
                    _errHandler.sync(this);
                    _la = _input.LA(1);
                    if (_la == LINE_COMMENT) {
                        {
                            setState(71);
                            match(LINE_COMMENT);
                        }
                    }

                }
                break;
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final AssignmentContext assignment() throws RecognitionException {
        AssignmentContext _localctx = new AssignmentContext(_ctx, getState());
        enterRule(_localctx, 10, RULE_assignment);
        try {
            enterOuterAlt(_localctx, 1);
            {
                setState(76);
                match(IDENTIFIER);
                setState(77);
                match(ASSIGN);
                setState(78);
                expr();
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final ExprContext expr() throws RecognitionException {
        ExprContext _localctx = new ExprContext(_ctx, getState());
        enterRule(_localctx, 12, RULE_expr);
        try {
            setState(82);
            _errHandler.sync(this);
            switch (_input.LA(1)) {
                case LBRACE:
                case LPAREN:
                case PLUS:
                case MINUS:
                case MUL:
                case UNKNOWN:
                case INTEGER:
                case IDENTIFIER:
                case APOSTROPHE:
                    enterOuterAlt(_localctx, 1);
                {
                    setState(80);
                    addExpr();
                }
                break;
                case IF:
                    enterOuterAlt(_localctx, 2);
                {
                    setState(81);
                    ifExpr();
                }
                break;
                default:
                    throw new NoViableAltException(this);
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final AddExprContext addExpr() throws RecognitionException {
        AddExprContext _localctx = new AddExprContext(_ctx, getState());
        enterRule(_localctx, 14, RULE_addExpr);
        int _la;
        try {
            enterOuterAlt(_localctx, 1);
            {
                setState(84);
                mulExpr();
                setState(89);
                _errHandler.sync(this);
                _la = _input.LA(1);
                while (_la == PLUS || _la == MINUS) {
                    {
                        {
                            setState(85);
                            _la = _input.LA(1);
                            if (!(_la == PLUS || _la == MINUS)) {
                                _errHandler.recoverInline(this);
                            } else {
                                if (_input.LA(1) == Token.EOF) matchedEOF = true;
                                _errHandler.reportMatch(this);
                                consume();
                            }
                            setState(86);
                            mulExpr();
                        }
                    }
                    setState(91);
                    _errHandler.sync(this);
                    _la = _input.LA(1);
                }
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final MulExprContext mulExpr() throws RecognitionException {
        MulExprContext _localctx = new MulExprContext(_ctx, getState());
        enterRule(_localctx, 16, RULE_mulExpr);
        int _la;
        try {
            enterOuterAlt(_localctx, 1);
            {
                setState(92);
                unaryExpr();
                setState(97);
                _errHandler.sync(this);
                _la = _input.LA(1);
                while (_la == MUL || _la == DIV) {
                    {
                        {
                            setState(93);
                            _la = _input.LA(1);
                            if (!(_la == MUL || _la == DIV)) {
                                _errHandler.recoverInline(this);
                            } else {
                                if (_input.LA(1) == Token.EOF) matchedEOF = true;
                                _errHandler.reportMatch(this);
                                consume();
                            }
                            setState(94);
                            unaryExpr();
                        }
                    }
                    setState(99);
                    _errHandler.sync(this);
                    _la = _input.LA(1);
                }
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final UnaryExprContext unaryExpr() throws RecognitionException {
        UnaryExprContext _localctx = new UnaryExprContext(_ctx, getState());
        enterRule(_localctx, 18, RULE_unaryExpr);
        int _la;
        try {
            enterOuterAlt(_localctx, 1);
            {
                setState(101);
                _errHandler.sync(this);
                _la = _input.LA(1);
                if ((((_la) & ~0x3f) == 0 && ((1L << _la) & 3584L) != 0)) {
                    {
                        setState(100);
                        _la = _input.LA(1);
                        if (!((((_la) & ~0x3f) == 0 && ((1L << _la) & 3584L) != 0))) {
                            _errHandler.recoverInline(this);
                        } else {
                            if (_input.LA(1) == Token.EOF) matchedEOF = true;
                            _errHandler.reportMatch(this);
                            consume();
                        }
                    }
                }

                setState(103);
                primary();
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final LiteralContext literal() throws RecognitionException {
        LiteralContext _localctx = new LiteralContext(_ctx, getState());
        enterRule(_localctx, 20, RULE_literal);
        try {
            enterOuterAlt(_localctx, 1);
            {
                setState(105);
                match(INTEGER);
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final PrimaryContext primary() throws RecognitionException {
        PrimaryContext _localctx = new PrimaryContext(_ctx, getState());
        enterRule(_localctx, 22, RULE_primary);
        try {
            setState(113);
            _errHandler.sync(this);
            switch (_input.LA(1)) {
                case LBRACE:
                case INTEGER:
                case IDENTIFIER:
                case APOSTROPHE:
                    enterOuterAlt(_localctx, 1);
                {
                    setState(107);
                    characterizable();
                }
                break;
                case LPAREN:
                    enterOuterAlt(_localctx, 2);
                {
                    setState(108);
                    match(LPAREN);
                    setState(109);
                    expr();
                    setState(110);
                    match(RPAREN);
                }
                break;
                case UNKNOWN:
                    enterOuterAlt(_localctx, 3);
                {
                    setState(112);
                    match(UNKNOWN);
                }
                break;
                default:
                    throw new NoViableAltException(this);
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final IfExprContext ifExpr() throws RecognitionException {
        IfExprContext _localctx = new IfExprContext(_ctx, getState());
        enterRule(_localctx, 24, RULE_ifExpr);
        try {
            int _alt;
            enterOuterAlt(_localctx, 1);
            {
                setState(115);
                ifExprHelperIf();
                setState(119);
                _errHandler.sync(this);
                _alt = getInterpreter().adaptivePredict(_input, 14, _ctx);
                while (_alt != 2 && _alt != org.antlr.v4.runtime.atn.ATN.INVALID_ALT_NUMBER) {
                    if (_alt == 1) {
                        {
                            {
                                setState(116);
                                ifExprHelperElif();
                            }
                        }
                    }
                    setState(121);
                    _errHandler.sync(this);
                    _alt = getInterpreter().adaptivePredict(_input, 14, _ctx);
                }
                setState(123);
                _errHandler.sync(this);
                switch (getInterpreter().adaptivePredict(_input, 15, _ctx)) {
                    case 1: {
                        setState(122);
                        ifExprHelperElse();
                    }
                    break;
                }
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final IfExprHelperIfContext ifExprHelperIf() throws RecognitionException {
        IfExprHelperIfContext _localctx = new IfExprHelperIfContext(_ctx, getState());
        enterRule(_localctx, 26, RULE_ifExprHelperIf);
        try {
            enterOuterAlt(_localctx, 1);
            {
                setState(125);
                match(IF);
                setState(126);
                expr();
                setState(127);
                match(THEN);
                setState(128);
                expr();
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final IfExprHelperElifContext ifExprHelperElif() throws RecognitionException {
        IfExprHelperElifContext _localctx = new IfExprHelperElifContext(_ctx, getState());
        enterRule(_localctx, 28, RULE_ifExprHelperElif);
        try {
            enterOuterAlt(_localctx, 1);
            {
                setState(130);
                match(ELIF);
                setState(131);
                expr();
                setState(132);
                match(THEN);
                setState(133);
                expr();
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    public final IfExprHelperElseContext ifExprHelperElse() throws RecognitionException {
        IfExprHelperElseContext _localctx = new IfExprHelperElseContext(_ctx, getState());
        enterRule(_localctx, 30, RULE_ifExprHelperElse);
        try {
            enterOuterAlt(_localctx, 1);
            {
                setState(135);
                match(ELSE);
                setState(136);
                expr();
            }
        } catch (RecognitionException re) {
            _localctx.exception = re;
            _errHandler.reportError(this, re);
            _errHandler.recover(this, re);
        } finally {
            exitRule();
        }
        return _localctx;
    }

    @SuppressWarnings("CheckReturnValue")
    public static class ProgramContext extends ParserRuleContext {
        public ProgramContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public BranesContext branes() {
            return getRuleContext(BranesContext.class, 0);
        }

        public TerminalNode EOF() {
            return getToken(FoolishParser.EOF, 0);
        }

        @Override
        public int getRuleIndex() {
            return RULE_program;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class CharacterizableContext extends ParserRuleContext {
        public CharacterizableContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public LiteralContext literal() {
            return getRuleContext(LiteralContext.class, 0);
        }

        public List<TerminalNode> IDENTIFIER() {
            return getTokens(FoolishParser.IDENTIFIER);
        }

        public TerminalNode IDENTIFIER(int i) {
            return getToken(FoolishParser.IDENTIFIER, i);
        }

        public BraneContext brane() {
            return getRuleContext(BraneContext.class, 0);
        }

        public TerminalNode APOSTROPHE() {
            return getToken(FoolishParser.APOSTROPHE, 0);
        }

        @Override
        public int getRuleIndex() {
            return RULE_characterizable;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class BraneContext extends ParserRuleContext {
        public BraneContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public TerminalNode LBRACE() {
            return getToken(FoolishParser.LBRACE, 0);
        }

        public TerminalNode RBRACE() {
            return getToken(FoolishParser.RBRACE, 0);
        }

        public List<StmtContext> stmt() {
            return getRuleContexts(StmtContext.class);
        }

        public StmtContext stmt(int i) {
            return getRuleContext(StmtContext.class, i);
        }

        @Override
        public int getRuleIndex() {
            return RULE_brane;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class BranesContext extends ParserRuleContext {
        public BranesContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public List<BraneContext> brane() {
            return getRuleContexts(BraneContext.class);
        }

        public BraneContext brane(int i) {
            return getRuleContext(BraneContext.class, i);
        }

        @Override
        public int getRuleIndex() {
            return RULE_branes;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class StmtContext extends ParserRuleContext {
        public StmtContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public BraneContext brane() {
            return getRuleContext(BraneContext.class, 0);
        }

        public TerminalNode SEMI() {
            return getToken(FoolishParser.SEMI, 0);
        }

        public TerminalNode LINE_COMMENT() {
            return getToken(FoolishParser.LINE_COMMENT, 0);
        }

        public BranesContext branes() {
            return getRuleContext(BranesContext.class, 0);
        }

        public AssignmentContext assignment() {
            return getRuleContext(AssignmentContext.class, 0);
        }

        public ExprContext expr() {
            return getRuleContext(ExprContext.class, 0);
        }

        @Override
        public int getRuleIndex() {
            return RULE_stmt;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class AssignmentContext extends ParserRuleContext {
        public AssignmentContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public TerminalNode IDENTIFIER() {
            return getToken(FoolishParser.IDENTIFIER, 0);
        }

        public TerminalNode ASSIGN() {
            return getToken(FoolishParser.ASSIGN, 0);
        }

        public ExprContext expr() {
            return getRuleContext(ExprContext.class, 0);
        }

        @Override
        public int getRuleIndex() {
            return RULE_assignment;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class ExprContext extends ParserRuleContext {
        public ExprContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public AddExprContext addExpr() {
            return getRuleContext(AddExprContext.class, 0);
        }

        public IfExprContext ifExpr() {
            return getRuleContext(IfExprContext.class, 0);
        }

        @Override
        public int getRuleIndex() {
            return RULE_expr;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class AddExprContext extends ParserRuleContext {
        public AddExprContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public List<MulExprContext> mulExpr() {
            return getRuleContexts(MulExprContext.class);
        }

        public MulExprContext mulExpr(int i) {
            return getRuleContext(MulExprContext.class, i);
        }

        public List<TerminalNode> PLUS() {
            return getTokens(FoolishParser.PLUS);
        }

        public TerminalNode PLUS(int i) {
            return getToken(FoolishParser.PLUS, i);
        }

        public List<TerminalNode> MINUS() {
            return getTokens(FoolishParser.MINUS);
        }

        public TerminalNode MINUS(int i) {
            return getToken(FoolishParser.MINUS, i);
        }

        @Override
        public int getRuleIndex() {
            return RULE_addExpr;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class MulExprContext extends ParserRuleContext {
        public MulExprContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public List<UnaryExprContext> unaryExpr() {
            return getRuleContexts(UnaryExprContext.class);
        }

        public UnaryExprContext unaryExpr(int i) {
            return getRuleContext(UnaryExprContext.class, i);
        }

        public List<TerminalNode> MUL() {
            return getTokens(FoolishParser.MUL);
        }

        public TerminalNode MUL(int i) {
            return getToken(FoolishParser.MUL, i);
        }

        public List<TerminalNode> DIV() {
            return getTokens(FoolishParser.DIV);
        }

        public TerminalNode DIV(int i) {
            return getToken(FoolishParser.DIV, i);
        }

        @Override
        public int getRuleIndex() {
            return RULE_mulExpr;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class UnaryExprContext extends ParserRuleContext {
        public UnaryExprContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public PrimaryContext primary() {
            return getRuleContext(PrimaryContext.class, 0);
        }

        public TerminalNode PLUS() {
            return getToken(FoolishParser.PLUS, 0);
        }

        public TerminalNode MINUS() {
            return getToken(FoolishParser.MINUS, 0);
        }

        public TerminalNode MUL() {
            return getToken(FoolishParser.MUL, 0);
        }

        @Override
        public int getRuleIndex() {
            return RULE_unaryExpr;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class LiteralContext extends ParserRuleContext {
        public LiteralContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public TerminalNode INTEGER() {
            return getToken(FoolishParser.INTEGER, 0);
        }

        @Override
        public int getRuleIndex() {
            return RULE_literal;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class PrimaryContext extends ParserRuleContext {
        public PrimaryContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public CharacterizableContext characterizable() {
            return getRuleContext(CharacterizableContext.class, 0);
        }

        public TerminalNode LPAREN() {
            return getToken(FoolishParser.LPAREN, 0);
        }

        public ExprContext expr() {
            return getRuleContext(ExprContext.class, 0);
        }

        public TerminalNode RPAREN() {
            return getToken(FoolishParser.RPAREN, 0);
        }

        public TerminalNode UNKNOWN() {
            return getToken(FoolishParser.UNKNOWN, 0);
        }

        @Override
        public int getRuleIndex() {
            return RULE_primary;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class IfExprContext extends ParserRuleContext {
        public IfExprContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public IfExprHelperIfContext ifExprHelperIf() {
            return getRuleContext(IfExprHelperIfContext.class, 0);
        }

        public List<IfExprHelperElifContext> ifExprHelperElif() {
            return getRuleContexts(IfExprHelperElifContext.class);
        }

        public IfExprHelperElifContext ifExprHelperElif(int i) {
            return getRuleContext(IfExprHelperElifContext.class, i);
        }

        public IfExprHelperElseContext ifExprHelperElse() {
            return getRuleContext(IfExprHelperElseContext.class, 0);
        }

        @Override
        public int getRuleIndex() {
            return RULE_ifExpr;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class IfExprHelperIfContext extends ParserRuleContext {
        public IfExprHelperIfContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public TerminalNode IF() {
            return getToken(FoolishParser.IF, 0);
        }

        public List<ExprContext> expr() {
            return getRuleContexts(ExprContext.class);
        }

        public ExprContext expr(int i) {
            return getRuleContext(ExprContext.class, i);
        }

        public TerminalNode THEN() {
            return getToken(FoolishParser.THEN, 0);
        }

        @Override
        public int getRuleIndex() {
            return RULE_ifExprHelperIf;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class IfExprHelperElifContext extends ParserRuleContext {
        public IfExprHelperElifContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public TerminalNode ELIF() {
            return getToken(FoolishParser.ELIF, 0);
        }

        public List<ExprContext> expr() {
            return getRuleContexts(ExprContext.class);
        }

        public ExprContext expr(int i) {
            return getRuleContext(ExprContext.class, i);
        }

        public TerminalNode THEN() {
            return getToken(FoolishParser.THEN, 0);
        }

        @Override
        public int getRuleIndex() {
            return RULE_ifExprHelperElif;
        }
    }

    @SuppressWarnings("CheckReturnValue")
    public static class IfExprHelperElseContext extends ParserRuleContext {
        public IfExprHelperElseContext(ParserRuleContext parent, int invokingState) {
            super(parent, invokingState);
        }

        public TerminalNode ELSE() {
            return getToken(FoolishParser.ELSE, 0);
        }

        public ExprContext expr() {
            return getRuleContext(ExprContext.class, 0);
        }

        @Override
        public int getRuleIndex() {
            return RULE_ifExprHelperElse;
        }
    }
}