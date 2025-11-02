package org.foolish.ubc;

import org.foolish.fvm.Env;

/**
 * Demo program to show what the UBC REPL output looks like.
 */
public class ReplOutputDemo {
    public static void main(String[] args) {
        Env env = new Env();

        System.out.println("UBC REPL Output Examples:");
        System.out.println("========================\n");

        // Test 1: Simple integer
        System.out.println("Input: {5;}");
        Object result1 = UbcRepl.eval("{5;}", env);
        System.out.println("Output:");
        System.out.println(result1);
        System.out.println();

        // Test 2: Binary expression
        System.out.println("Input: {10 + 20;}");
        Object result2 = UbcRepl.eval("{10 + 20;}", env);
        System.out.println("Output:");
        System.out.println(result2);
        System.out.println();

        // Test 3: Multiple expressions
        System.out.println("Input: {1; 2; 3 + 4;}");
        Object result3 = UbcRepl.eval("{1; 2; 3 + 4;}", env);
        System.out.println("Output:");
        System.out.println(result3);
        System.out.println();

        // Test 4: Nested expression
        System.out.println("Input: {(5 + 3) * 2;}");
        Object result4 = UbcRepl.eval("{(5 + 3) * 2;}", env);
        System.out.println("Output:");
        System.out.println(result4);
        System.out.println();
    }
}
