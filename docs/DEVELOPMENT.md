# Development

## Running Tests

### Running All Tests

To run all tests in the project, use:

```bash
mvn test
```

### Running Specific Approval Tests

We use a custom system property `foolish.test.filter` to filter approval tests by name. This property is respected by both `UbcApprovalTest` (Java) and `ScUbcApprovalTest` (Scala).

To run a specific approval test (or a set of tests matching a substring), use:

```bash
mvn test -Dfoolish.test.filter=simpleInteger
```

This will run any test whose input filename contains `simpleInteger`.

Examples:

**Run all tests containing "simple" in Java Core:**
```bash
cd foolish-core-java
mvn test -Dtest=UbcApprovalTest -Dfoolish.test.filter=simple
```

**Run a specific test in Scala Core:**
```bash
cd foolish-core-scala
mvn test -Dtest=ScUbcApprovalTest -Dfoolish.test.filter=complexArithmetic
```

**Run all approval tests in parallel (default behavior):**
The approval tests are configured to run concurrently. No extra flags are needed.
