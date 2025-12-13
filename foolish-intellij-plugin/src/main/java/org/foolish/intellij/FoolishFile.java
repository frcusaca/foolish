package org.foolish.intellij;

import com.intellij.extapi.psi.PsiFileBase;
import com.intellij.openapi.fileTypes.FileType;
import com.intellij.psi.FileViewProvider;
import org.jetbrains.annotations.NotNull;

public class FoolishFile extends PsiFileBase {
    protected FoolishFile(@NotNull FileViewProvider viewProvider) {
        super(viewProvider, FoolishLanguage.INSTANCE);
    }

    @Override
    public @NotNull FileType getFileType() {
        return FoolishFileType.INSTANCE;
    }
}
