package org.foolish.fvm.ubc;

import org.foolish.fvm.Env;
import org.foolish.fvm.ubc.UbcRepl;
import org.foolish.fvm.ubc.BraneFiroe;
import org.foolish.fvm.ubc.Sequencer4Human;

/**
 * Demo showing the Sequencer4Human output with '➠' tab character.
 */
public class SequencerDemo {
    public static void main(String[] args) {
        Env env = new Env();

        System.out.println("Sequencer4Human Demo");
        System.out.println("===================\n");

        System.out.println("Using default tab character '➠':\n");

        // Test 1: Simple integer
        System.out.println("Input: {5;}");
        BraneFiroe result1 = (BraneFiroe) UbcRepl.eval("{5;}", env);
        System.out.println("Output:");
        System.out.println(result1);
        System.out.println();

        // Test 2: Multiple expressions
        System.out.println("Input: {1; 2; 3 + 4;}");
        BraneFiroe result2 = (BraneFiroe) UbcRepl.eval("{1; 2; 3 + 4;}", env);
        System.out.println("Output:");
        System.out.println(result2);
        System.out.println();

        // Test 3: Nested expression
        System.out.println("Input: {(10 + 5) * 2;}");
        BraneFiroe result3 = (BraneFiroe) UbcRepl.eval("{(10 + 5) * 2;}", env);
        System.out.println("Output:");
        System.out.println(result3);
        System.out.println();

        // Test 4: Custom tab character
        System.out.println("Using custom tab character '  ' (two spaces):");
        System.out.println("Input: {10; 20; 30;}");
        BraneFiroe result4 = (BraneFiroe) UbcRepl.eval("{10; 20; 30;}", env);
        Sequencer4Human customSequencer = new Sequencer4Human("  ");
        System.out.println("Output:");
        System.out.println(customSequencer.sequence(result4));
        System.out.println();

        // Test 5: Different tab character
        System.out.println("Using custom tab character '→' (right arrow):");
        System.out.println("Input: {42;}");
        BraneFiroe result5 = (BraneFiroe) UbcRepl.eval("{42;}", env);
        Sequencer4Human arrowSequencer = new Sequencer4Human("→");
        System.out.println("Output:");
        System.out.println(arrowSequencer.sequence(result5));
    }
}
