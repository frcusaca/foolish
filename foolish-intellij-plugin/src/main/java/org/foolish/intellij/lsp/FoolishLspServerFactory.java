package org.foolish.intellij.lsp;

import com.redhat.devtools.lsp4ij.server.StreamConnectionProvider;
import com.redhat.devtools.lsp4ij.LanguageServerFactory;
import com.intellij.openapi.project.Project;
import org.jetbrains.annotations.NotNull;

public class FoolishLspServerFactory implements LanguageServerFactory {
    @Override
    public @NotNull StreamConnectionProvider createConnectionProvider(@NotNull Project project) {
        return new FoolishLanguageServer(project);
    }
}
