package org.foolish.fvm;

import org.junit.jupiter.api.Test;

import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.*;

class MidoeTest {
    @Test
    void cachesFinalResultWithoutReExecution() {
        AtomicInteger executions = new AtomicInteger();
        Insoe base = env -> {
            executions.incrementAndGet();
            return new IntegerLiteral(5);
        };
        Midoe midoe = new Midoe(base);
        Environment env = new Environment();

        Finer first = Evaluator.eval(midoe, env);
        Finer second = Evaluator.eval(midoe, env);

        assertEquals(1, executions.get(), "Base instruction should only execute once");
        assertSame(first, second);
        assertTrue(midoe.isFinal());
        assertFalse(midoe.isUnknown());
    }

    @Test
    void marksUnknownWhenFinalResultIsUnknown() {
        Midoe midoe = new Midoe(env -> Unknown.INSTANCE);
        Environment env = new Environment();

        Finer result = midoe.evaluate(env);

        assertSame(Unknown.INSTANCE, result);
        assertTrue(midoe.isFinal());
        assertTrue(midoe.isUnknown());
        assertSame(result, midoe.evaluate(env));
    }
}
