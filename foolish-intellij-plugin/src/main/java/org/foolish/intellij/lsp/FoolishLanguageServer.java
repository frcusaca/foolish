package org.foolish.intellij.lsp;

import com.redhat.devtools.lsp4ij.server.ProcessStreamConnectionProvider;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.diagnostic.Logger;
import java.io.File;
import java.util.ArrayList;
import java.util.List;

public class FoolishLanguageServer extends ProcessStreamConnectionProvider {
    private static final Logger LOG = Logger.getInstance(FoolishLanguageServer.class);
    private final Project project;

    public FoolishLanguageServer(Project project) {
        this.project = project;
        List<String> command = new ArrayList<>();
        command.add("java");

        try {
             // Locate the bundled jar
             // The jar should be in 'server/foolish-lsp.jar' relative to the plugin path.

             // First check a system property for dev override
             String customPath = System.getProperty("foolish.lsp.jar.path");
             String jarPath;
             if (customPath != null) {
                 jarPath = customPath;
             } else {
                 File pluginPath = com.intellij.ide.plugins.PluginManagerCore.getPlugin(
                     com.intellij.openapi.extensions.PluginId.getId("org.foolish.intellij")
                 ).getPluginPath().toFile();

                 File serverDir = new File(pluginPath, "server");
                 File jarFile = new File(serverDir, "foolish-lsp.jar");

                 if (!jarFile.exists()) {
                      LOG.warn("Could not find foolish-lsp.jar at " + jarFile.getAbsolutePath());
                      // Fallback: look in 'lib' if structure is different or flattened
                      jarFile = new File(new File(pluginPath, "lib"), "foolish-lsp.jar");
                 }

                 jarPath = jarFile.getAbsolutePath();
             }

             command.add("-jar");
             command.add(jarPath);

        } catch (Exception e) {
            LOG.error("Failed to locate foolish-lsp.jar", e);
        }

        super.setCommands(command);
    }
}
