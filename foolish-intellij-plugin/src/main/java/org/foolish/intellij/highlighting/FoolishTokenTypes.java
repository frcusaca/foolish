package org.foolish.intellij.highlighting;

import com.intellij.psi.tree.IElementType;
import org.foolish.intellij.FoolishLanguage;

public interface FoolishTokenTypes {
    IElementType KEYWORD = new IElementType("KEYWORD", FoolishLanguage.INSTANCE);
    IElementType IDENTIFIER = new IElementType("IDENTIFIER", FoolishLanguage.INSTANCE);
    IElementType STRING = new IElementType("STRING", FoolishLanguage.INSTANCE);
    IElementType NUMBER = new IElementType("NUMBER", FoolishLanguage.INSTANCE);
    IElementType COMMENT = new IElementType("COMMENT", FoolishLanguage.INSTANCE);
    IElementType BAD_CHARACTER = new IElementType("BAD_CHARACTER", FoolishLanguage.INSTANCE);
}
