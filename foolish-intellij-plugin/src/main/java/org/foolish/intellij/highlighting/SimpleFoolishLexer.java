package org.foolish.intellij.highlighting;

import com.intellij.lexer.Lexer;
import com.intellij.lexer.LexerBase;
import com.intellij.psi.tree.IElementType;
import com.intellij.util.text.CharArrayUtil;
import org.foolish.intellij.FoolishLanguage;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.util.Arrays;
import java.util.HashSet;
import java.util.Set;

public class SimpleFoolishLexer extends LexerBase {
    private CharSequence myBuffer;
    private int myStartOffset;
    private int myEndOffset;
    private int myTokenStart;
    private int myTokenEnd;
    private IElementType myTokenType;
    private int myState;

    private static final Set<String> KEYWORDS = new HashSet<>(Arrays.asList(
        "if", "else", "return", "while", "for", "fun", "var", "val", "class", "import"
    ));

    @Override
    public void start(@NotNull CharSequence buffer, int startOffset, int endOffset, int initialState) {
        myBuffer = buffer;
        myStartOffset = startOffset;
        myEndOffset = endOffset;
        myState = initialState;
        myTokenEnd = startOffset;
        advance();
    }

    @Override
    public int getState() {
        return myState;
    }

    @Override
    public @Nullable IElementType getTokenType() {
        return myTokenType;
    }

    @Override
    public int getTokenStart() {
        return myTokenStart;
    }

    @Override
    public int getTokenEnd() {
        return myTokenEnd;
    }

    @Override
    public void advance() {
        myTokenStart = myTokenEnd;
        if (myTokenStart >= myEndOffset) {
            myTokenType = null;
            return;
        }

        char c = myBuffer.charAt(myTokenStart);

        if (Character.isWhitespace(c)) {
            myTokenEnd++;
            while (myTokenEnd < myEndOffset && Character.isWhitespace(myBuffer.charAt(myTokenEnd))) {
                myTokenEnd++;
            }
            myTokenType = com.intellij.psi.TokenType.WHITE_SPACE;
            return;
        }

        // Comments
        if (c == '/' && myTokenStart + 1 < myEndOffset && myBuffer.charAt(myTokenStart + 1) == '/') {
             myTokenEnd += 2;
             while (myTokenEnd < myEndOffset && myBuffer.charAt(myTokenEnd) != '\n') {
                 myTokenEnd++;
             }
             myTokenType = FoolishTokenTypes.COMMENT;
             return;
        }

        // Strings
        if (c == '"') {
            myTokenEnd++;
            while (myTokenEnd < myEndOffset) {
                if (myBuffer.charAt(myTokenEnd) == '"' && myBuffer.charAt(myTokenEnd - 1) != '\\') {
                    myTokenEnd++;
                    break;
                }
                myTokenEnd++;
            }
            myTokenType = FoolishTokenTypes.STRING;
            return;
        }

        // Identifiers / Keywords
        if (Character.isJavaIdentifierStart(c)) {
            myTokenEnd++;
            while (myTokenEnd < myEndOffset && Character.isJavaIdentifierPart(myBuffer.charAt(myTokenEnd))) {
                myTokenEnd++;
            }
            String text = myBuffer.subSequence(myTokenStart, myTokenEnd).toString();
            if (KEYWORDS.contains(text)) {
                myTokenType = FoolishTokenTypes.KEYWORD;
            } else {
                myTokenType = FoolishTokenTypes.IDENTIFIER;
            }
            return;
        }

        // Numbers
        if (Character.isDigit(c)) {
            myTokenEnd++;
            while (myTokenEnd < myEndOffset && Character.isDigit(myBuffer.charAt(myTokenEnd))) {
                myTokenEnd++;
            }
            myTokenType = FoolishTokenTypes.NUMBER;
            return;
        }

        // Operators/Symbols
        myTokenEnd++;
        myTokenType = FoolishTokenTypes.BAD_CHARACTER; // Or generic symbol
    }

    @Override
    public @NotNull CharSequence getBufferSequence() {
        return myBuffer;
    }

    @Override
    public int getBufferEnd() {
        return myEndOffset;
    }
}
