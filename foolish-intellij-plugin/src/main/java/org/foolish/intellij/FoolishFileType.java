package org.foolish.intellij;

import com.intellij.openapi.fileTypes.LanguageFileType;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import javax.swing.*;

public class FoolishFileType extends LanguageFileType {
    public static final FoolishFileType INSTANCE = new FoolishFileType();

    private FoolishFileType() {
        super(FoolishLanguage.INSTANCE);
    }

    @Override
    public @NotNull String getName() {
        return "Foolish";
    }

    @Override
    public @NotNull String getDescription() {
        return "Foolish language file";
    }

    @Override
    public @NotNull String getDefaultExtension() {
        return "foo";
    }

    @Override
    public @Nullable Icon getIcon() {
        return null; // TODO: Add icon
    }
}
