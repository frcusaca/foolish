package org.foolish.intellij;

import com.intellij.lang.Language;

public class FoolishLanguage extends Language {
    public static final FoolishLanguage INSTANCE = new FoolishLanguage();

    private FoolishLanguage() {
        super("Foolish");
    }
}
