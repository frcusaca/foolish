
package foolishlang;

import java.nio.file.Files;
import java.nio.file.Path;

public class Main {
    public static void main(String[] args) throws Exception {
        String file = args.length>0 ? args[0] : "samples/hello.foo";
        String text = Files.readString(Path.of(file));
        var pr = ParserFacade.parseProgram(text);
        if (!pr.errors().isEmpty()) {
            System.err.println("Parse errors:");
            pr.errors().forEach(System.err::println);
            System.exit(1);
        }
        var ast = (AST.Program) new AstBuilder().visit(pr.tree());
        var sem = SymbolBuilder.build(ast);
        System.out.println("Parsed OK. Globals: " + sem.globals().entries().size() + " types.");
    }
}
