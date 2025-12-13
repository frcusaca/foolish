package org.foolish.intellij;

import com.intellij.lang.ASTNode;
import com.intellij.lang.ParserDefinition;
import com.intellij.lang.PsiParser;
import com.intellij.lexer.Lexer;
import com.intellij.lexer.EmptyLexer;
import com.intellij.openapi.project.Project;
import com.intellij.psi.FileViewProvider;
import com.intellij.psi.PsiElement;
import com.intellij.psi.PsiFile;
import com.intellij.psi.tree.IFileElementType;
import com.intellij.psi.tree.TokenSet;
import org.jetbrains.annotations.NotNull;

public class FoolishParserDefinition implements ParserDefinition {
    public static final IFileElementType FILE = new IFileElementType(FoolishLanguage.INSTANCE);

    @Override
    public @NotNull Lexer createLexer(Project project) {
        return new org.foolish.intellij.highlighting.SimpleFoolishLexer();
    }

    @Override
    public @NotNull PsiParser createParser(Project project) {
        return (root, builder) -> {
            builder.setDebugMode(true);
            var mark = builder.mark();
            while (!builder.eof()) {
                builder.advanceLexer();
            }
            mark.done(root);
            return builder.getTreeBuilt();
        };
    }

    @Override
    public @NotNull IFileElementType getFileNodeType() {
        return FILE;
    }

    @Override
    public @NotNull TokenSet getCommentTokens() {
        return TokenSet.EMPTY;
    }

    @Override
    public @NotNull TokenSet getStringLiteralElements() {
        return TokenSet.EMPTY;
    }

    @Override
    public @NotNull PsiElement createElement(ASTNode node) {
        return new com.intellij.psi.impl.source.tree.LeafPsiElement(node.getElementType(), node.getText());
    }

    @Override
    public @NotNull PsiFile createFile(@NotNull FileViewProvider viewProvider) {
        return new FoolishFile(viewProvider);
    }
}
