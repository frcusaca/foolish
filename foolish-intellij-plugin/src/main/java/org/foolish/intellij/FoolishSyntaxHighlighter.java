package org.foolish.intellij;

import com.intellij.lexer.Lexer;
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors;
import com.intellij.openapi.editor.colors.TextAttributesKey;
import com.intellij.openapi.fileTypes.SyntaxHighlighterBase;
import com.intellij.psi.tree.IElementType;
import org.foolish.intellij.highlighting.FoolishTokenTypes;
import org.foolish.intellij.highlighting.SimpleFoolishLexer;
import org.jetbrains.annotations.NotNull;

public class FoolishSyntaxHighlighter extends SyntaxHighlighterBase {
    public static final TextAttributesKey KEYWORD =
        TextAttributesKey.createTextAttributesKey("FOOLISH_KEYWORD", DefaultLanguageHighlighterColors.KEYWORD);
    public static final TextAttributesKey STRING =
        TextAttributesKey.createTextAttributesKey("FOOLISH_STRING", DefaultLanguageHighlighterColors.STRING);
    public static final TextAttributesKey NUMBER =
        TextAttributesKey.createTextAttributesKey("FOOLISH_NUMBER", DefaultLanguageHighlighterColors.NUMBER);
    public static final TextAttributesKey COMMENT =
        TextAttributesKey.createTextAttributesKey("FOOLISH_COMMENT", DefaultLanguageHighlighterColors.LINE_COMMENT);
    public static final TextAttributesKey IDENTIFIER =
        TextAttributesKey.createTextAttributesKey("FOOLISH_IDENTIFIER", DefaultLanguageHighlighterColors.IDENTIFIER);

    private static final TextAttributesKey[] KEYWORD_KEYS = new TextAttributesKey[]{KEYWORD};
    private static final TextAttributesKey[] STRING_KEYS = new TextAttributesKey[]{STRING};
    private static final TextAttributesKey[] NUMBER_KEYS = new TextAttributesKey[]{NUMBER};
    private static final TextAttributesKey[] COMMENT_KEYS = new TextAttributesKey[]{COMMENT};
    private static final TextAttributesKey[] IDENTIFIER_KEYS = new TextAttributesKey[]{IDENTIFIER};
    private static final TextAttributesKey[] EMPTY_KEYS = new TextAttributesKey[0];

    @Override
    public @NotNull Lexer getHighlightingLexer() {
        return new SimpleFoolishLexer();
    }

    @Override
    public TextAttributesKey @NotNull [] getTokenHighlights(IElementType tokenType) {
        if (tokenType.equals(FoolishTokenTypes.KEYWORD)) {
            return KEYWORD_KEYS;
        }
        if (tokenType.equals(FoolishTokenTypes.STRING)) {
            return STRING_KEYS;
        }
        if (tokenType.equals(FoolishTokenTypes.NUMBER)) {
            return NUMBER_KEYS;
        }
        if (tokenType.equals(FoolishTokenTypes.COMMENT)) {
            return COMMENT_KEYS;
        }
        if (tokenType.equals(FoolishTokenTypes.IDENTIFIER)) {
            return IDENTIFIER_KEYS;
        }
        return EMPTY_KEYS;
    }
}
