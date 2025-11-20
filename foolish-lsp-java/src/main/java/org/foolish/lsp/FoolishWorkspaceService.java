package org.foolish.lsp;

import org.eclipse.lsp4j.DidChangeConfigurationParams;
import org.eclipse.lsp4j.DidChangeWatchedFilesParams;
import org.eclipse.lsp4j.services.WorkspaceService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Placeholder workspace service. Provides logging hooks for configuration and file events so we
 * can validate IDE wiring before implementing incremental compilation.
 */
public final class FoolishWorkspaceService implements WorkspaceService {
    private static final Logger LOGGER = LoggerFactory.getLogger(FoolishWorkspaceService.class);

    @Override
    public void didChangeConfiguration(DidChangeConfigurationParams params) {
        LOGGER.info("Configuration changed: {}", params);
    }

    @Override
    public void didChangeWatchedFiles(DidChangeWatchedFilesParams params) {
        LOGGER.debug("Watched files changed: {}", params.getChanges());
    }
}
