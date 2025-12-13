# Maven Builder Skill for Claude Code

A comprehensive Maven build and test management skill for mixed Java/Scala projects with emphasis on parallel execution, intelligent debugging, and test regression preservation.

## Overview

This skill provides Claude Code with structured strategies for:
- **Compilation debugging**: Skip tests to focus purely on fixing syntax/type errors first
- **Parallel builds**: Optimized multi-threaded compilation for faster development cycles
- **Intelligent testing**: Parallel test execution (classes AND methods) with fallback to targeted debugging
- **Source generation**: Efficient handling of ANTLR4 and other code generators
- **Failure analysis**: XML-based test result analysis and pattern recognition
- **Approval tests**: Parallelization strategies for resource-file-based approval/golden master tests
- **Regression preservation**: Converting debugging code into permanent regression tests

## Contents

- **SKILL.md**: Complete Maven build reference with modes, strategies, and best practices
- **examples/scenarios.md**: 10 real-world scenarios demonstrating the skill in action
- **examples/quick-reference.md**: Cheat sheet for common commands and patterns

## Key Features

### 1. Build Modes
- **Compilation debugging** (`-DskipTests`) - Fix syntax errors first, test later
- Clean parallel builds (`-T 2C`) for fresh starts
- Incremental builds (`-T 1C`) for routine development  
- Source generation handling for ANTLR4 and similar tools
- Mixed Java/Scala compilation support

### 2. Test Execution Strategies
- **Standard**: Parallel test execution (`-Dparallel=classes,methods -DthreadCount=4`)
- **Approval tests**: Parallelized resource-file-based testing with JUnit 5 parameterized tests
- **Debugging**: Targeted verbose execution for failed tests
- **Regression**: Single test runs for focused debugging
- **Comprehensive**: Full suite validation before commits

### 3. Output Analysis
- XML report scanning for failure patterns
- Grouping related failures for efficient fixing
- Test timing analysis for performance regression detection
- Stack trace extraction and root cause identification

### 4. Workflow Intelligence
- Decision trees for selecting appropriate build commands
- Progressive debugging: broad → specific → single test
- Automatic regression test suggestion from debug code
- CI/CD optimization strategies

## Quick Start

### Compilation Debugging
```bash
# When you have compilation errors - skip tests!
mvn clean compile -T 2C -DskipTests
```

### Standard Development Flow
```bash
# 1. Make code changes
# 2. Incremental build + tests
mvn compile -T 1C test -Dparallel=classes,methods -DthreadCount=4
```

### After ANTLR4 Changes
```bash
mvn clean generate-sources -T 2C && mvn compile -T 2C -DskipTests
# Then after verifying compilation:
mvn test -Dparallel=classes,methods -DthreadCount=4
```

### Debugging Test Failures
```bash
# 1. Identify failures
mvn test -T 1C -Dparallel=classes,methods -DthreadCount=4

# 2. Debug specific test
mvn test -Dtest=FailedTest -DtrimStackTrace=false

# 3. Deep dive single method
mvn test -Dtest=FailedTest#method -DtrimStackTrace=false -X
```

### Parallelized Approval Tests
```bash
# Run all approval tests in parallel
mvn test -Dtest=ApprovalTests -Dparallel=classes,methods -DthreadCount=4

# Debug single approval test case
mvn test -Dtest="ApprovalTests#*testCase001*" -DtrimStackTrace=false
```
mvn test -Dtest=FailedTest#method -DtrimStackTrace=false -X
```

## Use Cases

This skill is designed for:
- ✅ Mixed Java/Scala projects
- ✅ Projects using ANTLR4 or other code generators
- ✅ Large test suites requiring parallel execution
- ✅ Teams practicing test-driven development
- ✅ CI/CD pipelines requiring fast feedback
- ✅ Projects with complex dependency graphs

## Integration with Claude Code

Claude Code will use this skill to:
1. Automatically select appropriate build commands based on file changes
2. Execute parallel builds and tests for fast feedback
3. Intelligently debug test failures by narrowing focus
4. Suggest converting debug code into regression tests
5. Analyze test output patterns to identify root causes

## Examples

See `examples/scenarios.md` for detailed walkthroughs including:
- ANTLR4 grammar modifications
- Refactoring with multiple test failures
- Flaky test investigation
- TDD workflows
- Complex stack trace debugging
- **Compilation error debugging (skip tests!)**
- Dependency update impacts
- **Parallelizing approval/golden master tests**
- Performance regression hunting
- Mixed Java/Scala build issues
- Production bug reproduction
- Cascading test failure handling

## Best Practices Encoded

1. **Separate compilation from testing**: Use `-DskipTests` when debugging compilation errors
2. **Parallel by default**: Use `-T 1C` or `-T 2C` unless debugging
3. **Parallelize classes AND methods**: `-Dparallel=classes,methods` for maximum test speed
4. **XML-first analysis**: Test reports are more reliable than console
5. **Incremental debugging**: Start broad, narrow to specific failures
6. **Preserve learning**: Convert debug code to regression tests
7. **Clean when uncertain**: `mvn clean` prevents stale build artifacts
8. **Group related fixes**: Fix related failures together
9. **Monitor resources**: Watch CPU/memory during parallel execution
10. **Parallelize approval tests**: Use JUnit 5 @ParameterizedTest for resource-based tests

## Requirements

- Maven 3.3+ (for parallel build support)
- JDK 8+ for Java projects
- Scala 2.12+ for Scala projects (if applicable)
- Sufficient system resources (4+ cores recommended for parallel builds)

## Output Locations

After running commands, results are in:
- `target/classes/` - Compiled Java/Scala code
- `target/test-classes/` - Compiled test code
- `target/generated-sources/` - Generated code (ANTLR4, etc.)
- `target/surefire-reports/` - Test reports (XML + TXT)
- `target/failsafe-reports/` - Integration test reports

## Troubleshooting

Common issues and solutions are documented in:
- SKILL.md section 8: "Common Issues and Solutions"
- quick-reference.md: "Troubleshooting" section

## Contributing Debug Tests to Regression Suites

When Claude Code creates temporary test code for debugging, this skill encourages:
1. Preserving valuable debug tests
2. Adding them to the permanent test suite
3. Labeling with `@Category(RegressionTest.class)` or similar
4. Documenting what production bug they prevent
5. Ensuring they run in standard test execution

## Performance Considerations

- Parallel builds use 1-2 threads per core (`-T 1C` or `-T 2C`)
- Test execution uses separate thread pool (`-DthreadCount=4`)
- Memory requirements scale with parallelism (set `MAVEN_OPTS` as needed)
- XML report parsing is efficient even for large test suites

## License

This skill is part of the Claude Code ecosystem and follows Anthropic's usage terms.

## Version

Version 1.0.0 - Initial release with comprehensive Maven support

---

For detailed command reference, see `examples/quick-reference.md`  
For real-world scenarios, see `examples/scenarios.md`  
For complete documentation, see `SKILL.md`
