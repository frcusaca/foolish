package org.foolish.reference.interpreter.ir;
import java.util.List;

public class RuntimeBrane implements RuntimeNode {
    private final List<RuntimeStatement> statements;
    public RuntimeBrane(List<RuntimeStatement> statements) { this.statements = statements; }
    public List<RuntimeStatement> getStatements() { return statements; }
}
