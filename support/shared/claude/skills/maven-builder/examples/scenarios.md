# Maven Builder - Example Scenarios

## Scenario 0: Compilation Errors - The Golden Rule

**Situation**: Refactored code, now getting compilation errors

**THE GOLDEN RULE**: When code won't compile, ALWAYS skip tests. Tests can't run anyway.

**Workflow**:
```bash
# ❌ WRONG APPROACH - Don't do this
mvn clean test -T 2C
# Result: Confusing mix of compilation errors + "can't run tests" messages

# ✅ CORRECT APPROACH - Focus on compilation first
# Step 1: Skip tests, see compilation errors clearly
mvn clean compile -T 2C -DskipTests

# Output shows clear compilation errors:
# [ERROR] /path/to/MyClass.java:[42,15] cannot find symbol
# [ERROR]   symbol:   method oldMethodName()
# [ERROR]   location: class com.example.Service

# Step 2: Fix compilation errors iteratively
# Make fixes, then quick recompile (still skipping tests)
mvn compile -T 1C -DskipTests

# Step 3: Repeat step 2 until compilation succeeds
# ...fix more issues...
mvn compile -T 1C -DskipTests
# [INFO] BUILD SUCCESS

# Step 4: ONLY NOW run tests (compilation proven to work)
mvn test -Dparallel=classesAndMethods -DthreadCount=4
```

**Why This Matters**:
- **Faster iteration**: Compilation without tests is 3-5x faster
- **Clear errors**: No test noise obscuring compilation problems
- **One problem at a time**: Fix compilation, THEN fix tests
- **Better workflow**: Know exactly what you're debugging

**Common mistake example**:
```bash
# Mistake: Trying to compile and test in one step when code is broken
mvn clean install -T 2C
# Result: 
#   - Compilation fails on multiple files
#   - Maven tries to run tests anyway (fails)
#   - Error output is 500+ lines mixing compilation + test failures
#   - Hard to see what to fix first

# Better approach:
mvn clean compile -T 2C -DskipTests
# Result:
#   - Clear list of compilation errors
#   - Fix them one by one
#   - Recompile quickly to verify
#   - When BUILD SUCCESS appears, THEN run tests
```

---

## Scenario 1: Modified ANTLR4 Grammar

**Situation**: User modified `MyParser.g4` grammar file

**Workflow**:
```bash
# Step 1: Clean and regenerate sources with parallelism
mvn clean generate-sources -T 2C

# Step 2: Compile everything (Java/Scala + generated code)
mvn compile -T 2C

# Step 3: Run tests to verify grammar changes
mvn test -Dparallel=classesAndMethods -DthreadCount=4
```

**Output Analysis**:
- Check `target/generated-sources/antlr4/` for new parser/lexer classes
- Review console for parser generation warnings
- Scan test XML reports for parsing-related failures

---

## Scenario 2: Multiple Test Failures After Refactoring

**Situation**: Refactored core service class, now 8 tests fail

**Workflow**:
```bash
# Step 1: Full parallel test run to identify all failures
mvn clean test -T 2C -Dparallel=classesAndMethods -DthreadCount=4

# Step 2: Analyze the failures
cd target/surefire-reports/
grep -l "failures=\"[1-9]" TEST-*.xml
# Output shows:
# TEST-com.example.ServiceTest.xml
# TEST-com.example.IntegrationTest.xml
# TEST-com.example.ControllerTest.xml

# Step 3: Look for patterns in failures
grep "failure message" TEST-com.example.*.xml

# Step 4: Group related failures and fix together
# If all failures are NullPointerException in same method,
# likely single root cause in refactored code

# Step 5: Re-run specific test class verbosely
mvn test -Dtest=ServiceTest -DtrimStackTrace=false

# Step 6: Verify fix with full parallel test suite
mvn test -T 1C -Dparallel=classesAndMethods -DthreadCount=4
```

**Key Insight**: The failures were related - all accessing refactored method with changed null-handling

---

## Scenario 3: Flaky Test Investigation

**Situation**: `PaymentProcessorTest#testConcurrentPayments` fails intermittently in parallel runs

**Workflow**:
```bash
# Step 1: Confirm it fails in parallel
mvn test -Dtest=PaymentProcessorTest -Dparallel=classesAndMethods -DthreadCount=4
# Sometimes passes, sometimes fails

# Step 2: Run sequentially to isolate parallelism issue
mvn test -Dtest=PaymentProcessorTest -DthreadCount=1
# Always passes

# Step 3: Run single method with verbose output
mvn test -Dtest=PaymentProcessorTest#testConcurrentPayments \
  -DtrimStackTrace=false -X 2>&1 | tee payment-test.log

# Step 4: Review the test for shared state issues
# Discovered: Test uses static PaymentGateway instance
# Fix: Make instance per-test or synchronize access

# Step 5: Create regression test
# Added new test: testPaymentGatewayThreadSafety()
# Verifies concurrent access explicitly
```

---

## Scenario 4: Adding New Feature with TDD

**Situation**: Implementing new validation logic with test-first approach

**Workflow**:
```bash
# Step 1: Write failing test
# Create ValidationTest.java with testEmailValidation()

# Step 2: Run just that test to confirm it fails
mvn test -Dtest=ValidationTest#testEmailValidation
# Expected failure: Method not implemented

# Step 3: Implement minimal code to pass
# Add emailValidation() method to Validator class

# Step 4: Quick compile + single test
mvn compile test -Dtest=ValidationTest#testEmailValidation
# Passes now

# Step 5: Run all validation tests to ensure no regression
mvn test -Dtest=ValidationTest -DthreadCount=2

# Step 6: Full parallel test suite before commit
mvn test -T 1C -Dparallel=classesAndMethods -DthreadCount=4
```

---

## Scenario 5: Debugging Complex Stack Trace

**Situation**: Integration test fails with 200+ line stack trace

**Workflow**:
```bash
# Step 1: Initial failure in parallel run
mvn verify -T 1C -Dparallel=classesAndMethods -DthreadCount=4
# DatabaseIntegrationTest.testDataMigration - FAILED

# Step 2: Get full stack trace to file
mvn test -Dtest=DatabaseIntegrationTest#testDataMigration \
  -DtrimStackTrace=false -X > migration-debug.log 2>&1

# Step 3: Analyze the log file
# Root cause buried in trace: Foreign key constraint violation
# Caused by: SQLIntegrityConstraintViolationException...

# Step 4: Create minimal reproduction test
# New test: testForeignKeyHandling() 
# Isolates just the constraint issue without full migration

# Step 5: Run minimal test while developing fix
mvn test -Dtest=DatabaseIntegrationTest#testForeignKeyHandling
# Fast iteration loop

# Step 6: After fix, verify with full migration test
mvn test -Dtest=DatabaseIntegrationTest#testDataMigration \
  -DtrimStackTrace=false

# Step 7: Keep minimal test as regression test
# Rename to testMigrationForeignKeyRegression()
# Add @Category(RegressionTest.class)
```

---

## Scenario 6: Compilation Errors - Skip Tests, Fix Code First

**Situation**: After major API refactoring, 50+ compilation errors across 15 files

**Workflow**:
```bash
# Step 1: Attempt to compile with tests - fails immediately
mvn clean compile -T 2C
# ERROR: Compilation failure - cannot find symbol

# Step 2: CRITICAL - Skip tests, focus ONLY on compilation
mvn clean compile -T 2C -DskipTests
# Still fails but faster iteration - no time wasted on tests

# Step 3: Identify all compilation errors
mvn compile -T 2C -DskipTests 2>&1 | grep "ERROR" | sort -u
# Shows affected files and error types

# Step 4: Fix compilation errors iteratively
# Edit files to fix import errors, method signatures, etc.

# Step 5: Quick compile check after each batch of fixes
mvn compile -T 1C -DskipTests
# Fast feedback loop - under 30 seconds

# Step 6: Once all compilation errors fixed
mvn compile -T 1C -DskipTests
# BUILD SUCCESS

# Step 7: NOW run tests to verify behavior
mvn test -Dparallel=classesAndMethods -DthreadCount=4
# 12 tests fail - but at least code compiles

# Step 8: Fix test failures
# Update test expectations for new API

# Step 9: Final verification
mvn clean test -T 2C -Dparallel=classesAndMethods -DthreadCount=4
# All tests pass
```

**Key Insight**: Separating compilation from testing allows faster iteration when fixing syntax/type errors. Tests are meaningless until code compiles.

**Mistake to Avoid**:
```bash
# DON'T DO THIS when you have compilation errors:
mvn clean install -T 2C
# Wastes time trying to run tests that can't possibly work
```

**Correct Approach**:
```bash
# DO THIS - skip tests during compilation debugging:
mvn clean compile -T 2C -DskipTests
# Then once compilation succeeds:
mvn test -Dparallel=classesAndMethods -DthreadCount=4
```

---

## Scenario 7: Dependency Update Breaking Build

**Situation**: Updated Jackson from 2.13.0 to 2.15.0, compilation fails

**Workflow**:
```bash
# Step 1: Clean build to eliminate stale classes - SKIP TESTS
mvn clean compile -T 2C -DskipTests
# Errors in SerializationUtil.java - deprecated methods

# Step 2: Identify all affected files
mvn compile -T 2C -DskipTests 2>&1 | grep "error:"
# Shows 5 files with compilation errors

# Step 3: Fix compilation errors
# Update deprecated method calls to new API

# Step 4: Incremental compile to verify fixes
mvn compile -T 1C -DskipTests
# BUILD SUCCESS - compilation fixed

# Step 5: NOW run tests to catch runtime issues
mvn test -Dparallel=classesAndMethods -DthreadCount=4
# 3 serialization tests fail

# Step 6: Debug failed serialization tests
mvn test -Dtest=*SerializationTest -DtrimStackTrace=false

# Step 7: Fix test expectations for new Jackson behavior
# Jackson 2.15 changed default date serialization format

# Step 8: Full verification
mvn clean verify -T 2C -Dparallel=classesAndMethods -DthreadCount=4
```

---

## Scenario 8: Parallelizing Approval Tests

**Situation**: Have 200 approval tests in loop-based format, taking 10 minutes to run sequentially

**Current Code** (sequential):
```java
@Test
public void approvalTests() throws Exception {
    List<String> testFiles = loadApprovalTestFiles();
    for (String testFile : testFiles) {
        String input = loadInput(testFile);
        String expected = loadExpected(testFile);
        String actual = processInput(input);
        assertEquals(expected, actual, "Failed: " + testFile);
    }
}
```

**Refactoring Workflow**:
```bash
# Step 1: Record baseline timing
mvn test -Dtest=ApprovalTests
# Takes 600 seconds (10 minutes)

# Step 2: Refactor to JUnit 5 parameterized tests
# New code:
```
```java
@ParameterizedTest
@MethodSource("approvalTestCases")
@Execution(ExecutionMode.CONCURRENT)
void approvalTest(ApprovalTestCase testCase) {
    String actual = processInput(testCase.getInput());
    assertEquals(testCase.getExpected(), actual);
}

static Stream<ApprovalTestCase> approvalTestCases() {
    return loadApprovalTestFiles()
        .map(ApprovalTestCase::fromFile);
}
```
```bash

# Step 3: Add parallel config to junit-platform.properties
# junit.jupiter.execution.parallel.enabled=true
# junit.jupiter.execution.parallel.mode.default=concurrent

# Step 4: Run with parallelization
mvn test -Dtest=ApprovalTests -Dparallel=classesAndMethods -DthreadCount=4
# Takes 180 seconds (3 minutes) - 70% faster!

# Step 5: Verify all tests still pass
mvn test -Dtest=ApprovalTests -Dparallel=classesAndMethods -DthreadCount=4
# All 200 tests pass

# Step 6: Debug specific failing case (if any)
mvn test -Dtest="ApprovalTests#*edge_case_42*" -DtrimStackTrace=false
# Can target individual test case by name

# Step 7: Increase parallelism for even faster execution
mvn test -Dtest=ApprovalTests -Dparallel=classesAndMethods -DthreadCount=8
# Takes 120 seconds (2 minutes) on 8-core machine

# Step 8: Check test timings to find bottlenecks
cd target/surefire-reports/
grep "time=" TEST-ApprovalTests.xml | head -20
# Identify which test cases are slowest
```

**Benefits Achieved**:
- Execution time: 10min → 2min (80% reduction)
- Can debug individual test cases easily
- Added 200 new test cases without proportional time increase
- Better isolation - failures don't stop entire suite

**Debugging Individual Approval Test**:
```bash
# Find which test case failed
cd target/surefire-reports/
grep "failure" TEST-ApprovalTests.xml

# Run just that test case
mvn test -Dtest="ApprovalTests#approvalTest*regression_bug_123*" \
  -DtrimStackTrace=false
```

---

## Scenario 9: Performance Regression Hunt

**Situation**: CI reports tests taking 3x longer, need to find slow tests

**Workflow**:
```bash
# Step 1: Run tests with timing
mvn test -Dparallel=classesAndMethods -DthreadCount=4

# Step 2: Analyze test timings from XML reports
cd target/surefire-reports/
for f in TEST-*.xml; do
  echo -n "$f: "
  grep "time=" "$f" | head -1 | sed 's/.*time="\([^"]*\)".*/\1/'
done | sort -t: -k2 -rn
# Shows UserServiceTest taking 45 seconds (was 2 seconds)

# Step 3: Run slow test individually with profiling
mvn test -Dtest=UserServiceTest -DtrimStackTrace=false -X

# Step 4: Review test for performance anti-patterns
# Found: Test doing full database rebuild for each method
# Fix: Move setup to @BeforeClass

# Step 5: Verify performance improvement
mvn test -Dtest=UserServiceTest
# Now completes in 3 seconds

# Step 6: Create performance regression test
# Add testServiceResponseTime() with timeout annotation
# @Test(timeout = 5000) // Fail if takes > 5 seconds
```

---

## Scenario 10: Mixed Java/Scala Build Issues

**Situation**: Scala code can't see newly added Java interface

**Workflow**:
```bash
# Step 1: Clean build with explicit ordering - SKIP TESTS first
mvn clean compile -T 2C -DskipTests

# Problem: Scala plugin compiles before Java compiler runs
# Solution: Check pom.xml execution order

# Step 2: Verify Java compilation first
mvn clean compile -T 1 -DskipTests  # Sequential to see order
# Java compiles -> Scala compiles -> Success

# Step 3: But parallel build still fails
mvn clean compile -T 2C -DskipTests
# Race condition in parallel execution

# Step 4: Fix pom.xml configuration
# Add <phase>compile</phase> explicitly to java compiler plugin
# Add <phase>compile</phase> to scala-maven-plugin
# Ensure Java plugin <execution> listed before Scala

# Step 5: Test parallel compilation
mvn clean compile -T 2C -DskipTests
# Now works correctly

# Step 6: Run tests to verify
mvn test -Dparallel=classesAndMethods -DthreadCount=4
```

---

## Scenario 11: Creating Debug Test for Production Bug

**Situation**: Production bug reports intermittent calculation errors

**Workflow**:
```bash
# Step 1: Create reproduction test
# New file: CalculationRegressionTest.java
# Test method: testRoundingEdgeCase()

# Step 2: Run new test to confirm it reproduces bug
mvn test -Dtest=CalculationRegressionTest#testRoundingEdgeCase
# Test fails as expected - reproduces the bug

# Step 3: Use test for rapid debugging iteration
# Add logging, breakpoints, assertions
mvn test -Dtest=CalculationRegressionTest#testRoundingEdgeCase

# Step 4: Implement fix in Calculator.java
# Changed BigDecimal rounding mode

# Step 5: Verify fix
mvn test -Dtest=CalculationRegressionTest#testRoundingEdgeCase
# Test passes

# Step 6: Keep test permanently in suite
# Move from debug package to main test suite
# Add documentation comment explaining production bug
# Add @Category(RegressionTest.class)

# Step 7: Run full test suite
mvn test -T 1C -Dparallel=classesAndMethods -DthreadCount=4
```

---

## Scenario 12: Handling Cascading Test Failures

**Situation**: One core utility change causes 50+ test failures

**Workflow**:
```bash
# Step 1: Full parallel test to identify scope
mvn clean test -T 2C -Dparallel=classesAndMethods -DthreadCount=4
# 53 tests failed

# Step 2: Export failure summary
cd target/surefire-reports/
grep -h "testcase.*failure" TEST-*.xml | \
  sed 's/.*classname="\([^"]*\)".*name="\([^"]*\)".*/\1#\2/' | \
  sort | uniq > /tmp/failed-tests.txt

# Step 3: Group by common patterns
cat /tmp/failed-tests.txt | cut -d# -f1 | uniq -c | sort -rn
# Shows:
#  23 com.example.util.DateUtilTest
#  15 com.example.service.OrderServiceTest
#  10 com.example.controller.ApiControllerTest
#   5 others...

# Step 4: Focus on most-failed class first
mvn test -Dtest=DateUtilTest -DtrimStackTrace=false

# Step 5: Identify root cause
# All failures: Expected format "yyyy-MM-dd" but got "yyyy-M-d"
# Changed utility now returns non-zero-padded months

# Step 6: Decide fix strategy
# Option A: Revert utility change
# Option B: Update all test expectations
# Choose B: The new format is correct, tests need updating

# Step 7: Fix tests incrementally
# Update DateUtilTest expectations
mvn test -Dtest=DateUtilTest
# All pass

# Update OrderServiceTest expectations  
mvn test -Dtest=OrderServiceTest
# All pass

# Step 8: Final verification
mvn test -T 1C -Dparallel=classesAndMethods -DthreadCount=4
# All 53 failures now pass
```

---

## Scenario 11: Parallelizing Approval Tests

**Situation**: Have 150 approval tests reading resource files in a loop - takes 10 minutes sequentially

**Current Code** (slow, sequential):
```java
@Test
public void approvalTests() throws IOException {
    List<String> testFiles = loadTestFilesFromResources("approval-tests/");
    for (String testFile : testFiles) {
        String input = readResource(testFile + "_input.txt");
        String expected = readResource(testFile + "_expected.txt");
        String actual = myProcessor.process(input);
        assertEquals(expected, actual, "Failed: " + testFile);
    }
}
```

**Problem**: 150 tests × 4 seconds each = 10 minutes

**Workflow**:

```bash
# Step 1: Measure current performance
mvn test -Dtest=ApprovalTests
# [INFO] Tests run: 1, Time elapsed: 600.123 s

# Step 2: Refactor to JUnit 5 @ParameterizedTest
# New code (see below)

# Step 3: Enable parallel execution
# Add junit-platform.properties:
# junit.jupiter.execution.parallel.enabled=true
# junit.jupiter.execution.parallel.mode.default=concurrent

# Step 4: Run with parallel execution
mvn test -Dtest=ApprovalTests -Dparallel=classesAndMethods -DthreadCount=4
# [INFO] Tests run: 150, Time elapsed: 155.432 s
# Nearly 4x speedup!

# Step 5: Debug single failing approval test
mvn test -Dtest="ApprovalTests#*edge_case_null*" -DtrimStackTrace=false
# Targets just one approval test case

# Step 6: Add more test cases - performance scales
# Add 50 more test files
mvn test -Dtest=ApprovalTests -Dparallel=classesAndMethods -DthreadCount=4
# [INFO] Tests run: 200, Time elapsed: 205.123 s
# Still ~4 seconds per test, but 4 running concurrently
```

**Refactored Code** (fast, parallel):

```java
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.MethodSource;
import org.junit.jupiter.api.parallel.Execution;
import org.junit.jupiter.api.parallel.ExecutionMode;

public class ApprovalTests {
    
    @ParameterizedTest(name = "Approval test: {0}")
    @MethodSource("approvalTestCases")
    @Execution(ExecutionMode.CONCURRENT)  // KEY: Enable parallel execution
    void approvalTest(ApprovalTestCase testCase) throws IOException {
        String input = testCase.loadInput();
        String expected = testCase.loadExpected();
        String actual = myProcessor.process(input);
        
        assertEquals(expected, actual, 
            "Approval test failed for: " + testCase.getName());
    }
    
    static Stream<ApprovalTestCase> approvalTestCases() throws IOException {
        // Load all test files from resources directory
        List<String> testFiles = loadTestFilesFromResources("approval-tests/");
        
        return testFiles.stream()
            .map(filename -> new ApprovalTestCase(
                filename,
                "approval-tests/" + filename + "_input.txt",
                "approval-tests/" + filename + "_expected.txt"
            ));
    }
    
    record ApprovalTestCase(String name, String inputPath, String expectedPath) {
        String loadInput() throws IOException {
            return readResource(inputPath);
        }
        
        String loadExpected() throws IOException {
            return readResource(expectedPath);
        }
        
        String getName() {
            return name;
        }
    }
}
```

**Alternative: Using @TestFactory for Dynamic Tests**:

```java
@TestFactory
Stream<DynamicTest> approvalTests() throws IOException {
    List<String> testFiles = loadTestFilesFromResources("approval-tests/");
    
    return testFiles.stream()
        .map(testFile -> DynamicTest.dynamicTest(
            "Approval: " + testFile,
            () -> {
                String input = readResource("approval-tests/" + testFile + "_input.txt");
                String expected = readResource("approval-tests/" + testFile + "_expected.txt");
                String actual = myProcessor.process(input);
                assertEquals(expected, actual);
            }
        ));
}
```

**Configuration**: `src/test/resources/junit-platform.properties`
```properties
junit.jupiter.execution.parallel.enabled=true
junit.jupiter.execution.parallel.mode.default=concurrent
junit.jupiter.execution.parallel.mode.classes.default=concurrent
junit.jupiter.execution.parallel.config.strategy=dynamic
```

**Debugging Specific Approval Test**:
```bash
# Run just tests matching pattern
mvn test -Dtest="ApprovalTests#*edge_case*" -DtrimStackTrace=false

# Or use display name filtering
mvn test -Dtest="ApprovalTests" \
  -Djunit.platform.filters.displayname=".*null_handling.*"
```

**Benefits**:
- **Speed**: 150 tests in ~2.5 minutes instead of 10 minutes (4x speedup on 4 cores)
- **Scalability**: Adding tests doesn't linearly increase runtime
- **Debuggability**: Can target single test case easily
- **CI/CD**: Much faster feedback in continuous integration
- **Isolation**: Each test case is independent, reducing flakiness

**Migration Steps**:
1. Keep old loop-based test initially (comment it out)
2. Add parameterized version alongside
3. Verify both produce same results
4. Enable parallelization
5. Remove old loop-based test
6. Enjoy 4x+ speedup

---

## Scenario 11: Parallelizing Approval Tests

**Situation**: Have 150 approval tests running sequentially in a loop, taking 5 minutes

**Current Code**:
```java
@Test
public void allApprovalTests() throws Exception {
    List<File> testFiles = loadApprovalTestFiles();
    for (File testFile : testFiles) {
        String input = readFile(testFile.getName() + "_input.txt");
        String expected = readFile(testFile.getName() + "_expected.txt");
        String actual = processor.process(input);
        assertEquals(expected, actual, "Failed: " + testFile.getName());
    }
}
```

**Problem**: Sequential execution is slow; one failure stops all remaining tests

**Workflow**:
```bash
# Step 1: Refactor to JUnit 5 @ParameterizedTest
# New code structure:
# @ParameterizedTest
# @MethodSource("approvalTestCases")
# @Execution(ExecutionMode.CONCURRENT)
# void approvalTest(ApprovalTestCase testCase) { ... }

# Step 2: Add junit-platform.properties
cat > src/test/resources/junit-platform.properties << EOF
junit.jupiter.execution.parallel.enabled=true
junit.jupiter.execution.parallel.mode.default=concurrent
junit.jupiter.execution.parallel.mode.classes.default=concurrent
EOF

# Step 3: Run with parallel execution
mvn test -Dtest=ApprovalTests -Dparallel=classesAndMethods -DthreadCount=4

# Step 4: Verify all tests still pass
# Review XML output for any failures
grep "failures=" target/surefire-reports/TEST-ApprovalTests.xml

# Step 5: Check performance improvement
grep "time=" target/surefire-reports/TEST-ApprovalTests.xml
# Before: time="300.45" (5 minutes)
# After: time="82.12" (82 seconds) - ~3.7x speedup!

# Step 6: Debug single failing approval test
mvn test -Dtest="ApprovalTests#*edge_case_null*" -DtrimStackTrace=false

# Step 7: Add new approval test case
# Just add new files to resources:
# - src/test/resources/approval-tests/new_test_input.txt
# - src/test/resources/approval-tests/new_test_expected.txt

# Step 8: Re-run to verify new test
mvn test -Dtest=ApprovalTests -Dparallel=classesAndMethods -DthreadCount=4
```

**Key Benefits**:
- 3-4x faster execution with 4 threads
- Each test case isolated and independently debuggable
- Can target single test case by name pattern
- Adding new test cases requires no code changes
- Failures don't block remaining tests

**Refactored Code**:
```java
public class ApprovalTests {
    
    @ParameterizedTest(name = "Approval test: {0}")
    @MethodSource("approvalTestCases")
    @Execution(ExecutionMode.CONCURRENT)  // Enable parallelization
    void approvalTest(ApprovalTestCase testCase) throws Exception {
        String input = testCase.loadInput();
        String expected = testCase.loadExpected();
        String actual = processor.process(input);
        
        assertEquals(expected, actual, 
            "Approval test failed for: " + testCase.getName());
    }
    
    static Stream<ApprovalTestCase> approvalTestCases() throws Exception {
        Path testsDir = Paths.get("src/test/resources/approval-tests");
        return Files.list(testsDir)
            .filter(p -> p.toString().endsWith("_input.txt"))
            .map(ApprovalTestCase::fromInputFile);
    }
}

class ApprovalTestCase {
    private final String name;
    private final Path inputPath;
    private final Path expectedPath;
    
    static ApprovalTestCase fromInputFile(Path inputPath) {
        String name = inputPath.getFileName().toString()
            .replace("_input.txt", "");
        Path expectedPath = inputPath.getParent()
            .resolve(name + "_expected.txt");
        return new ApprovalTestCase(name, inputPath, expectedPath);
    }
    
    String loadInput() throws IOException {
        return Files.readString(inputPath);
    }
    
    String loadExpected() throws IOException {
        return Files.readString(expectedPath);
    }
    
    String getName() { return name; }
    
    @Override
    public String toString() { return name; }
}
```

---

## Best Practices Demonstrated

1. **Separate compilation from testing**: Use `-DskipTests` when debugging compilation errors
2. **Start broad, narrow down**: Full test run → specific test class → single test method
3. **Use parallelism by default**: Only go sequential when debugging specific issues
4. **Preserve debug work**: Convert debugging tests into regression tests
5. **Analyze before fixing**: Use XML reports to understand failure patterns
6. **Verify fixes comprehensively**: Single test pass → test class pass → full suite pass
7. **Document learnings**: Add comments explaining non-obvious test cases
8. **Parallelize approval tests**: Use JUnit 5 parameterized tests for massive speedup
