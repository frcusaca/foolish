
package foolishlang;

import com.foolishlang.grammar.*;
import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.tree.ParseTree;
import java.util.*;

public class ParserFacade {
    public record ParseResult(ParseTree tree, List<String> errors) {}

    public static ParseResult parseProgram(String text) {
        FoolishLexer lexer = new FoolishLexer(CharStreams.fromString(text));
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);
        var errors = new ArrayList<String>();
        BaseErrorListener el = new BaseErrorListener() {
            @Override public void syntaxError(Recognizer<?, ?> recognizer, Object offendingSymbol, int line, int charPositionInLine, String msg, RecognitionException e) {
                errors.add("line "+line+":"+charPositionInLine+" "+msg);
            }
        };
        parser.removeErrorListeners();
        lexer.removeErrorListeners();
        parser.addErrorListener(el);
        lexer.addErrorListener(el);
        ParseTree tree = parser.program();
        return new ParseResult(tree, errors);
    }

    // Simple statement-level incremental parsing inside outermost brane
    public static class Incremental {
        private final List<Segment> segments = new ArrayList<>();

        public static final class Segment {
            public String text;
            public int startOffset;
            public int endOffset;
            public ParseResult parse;
        }

        public Incremental(String text) {
            setText(text);
        }

        public void setText(String text) {
            segments.clear();
            int open = text.indexOf('{');
            int close = text.lastIndexOf('}');
            if (open < 0 || close <= open) {
                Segment s = new Segment();
                s.text = text; s.startOffset=0; s.endOffset=text.length();
                s.parse = parseProgram(text);
                segments.add(s);
                return;
            }
            String inner = text.substring(open+1, close);
            int segStart = open+1;
            StringBuilder buf = new StringBuilder();
            for (int i=0;i<inner.length();i++) {
                char c = inner.charAt(i);
                buf.append(c);
                if (c=='\n' || c==';') {
                    addSegment(segStart, open+1+i+1, buf.toString());
                    segStart = open+1+i+1;
                    buf.setLength(0);
                }
            }
            if (buf.length()>0) addSegment(segStart, close, buf.toString());
        }

        private void addSegment(int start, int end, String text) {
            Segment s = new Segment();
            s.text = text; s.startOffset=start; s.endOffset=end;
            s.parse = parseProgram("{ "+text+" }");
            segments.add(s);
        }

        public void edit(int start, int end, String replacement) {
            for (Segment s : segments) {
                if (end <= s.startOffset || start >= s.endOffset) continue;
                int localStart = Math.max(0, start - s.startOffset);
                int localEnd   = Math.min(s.text.length(), end - s.startOffset);
                String before = s.text.substring(0, localStart);
                String after  = s.text.substring(localEnd);
                s.text = before + replacement + after;
                int newEnd = s.startOffset + s.text.length();
                s.endOffset = newEnd;
                s.parse = parseProgram("{ "+s.text+" }");
            }
        }

        public List<Segment> segments(){ return segments; }
    }
}
