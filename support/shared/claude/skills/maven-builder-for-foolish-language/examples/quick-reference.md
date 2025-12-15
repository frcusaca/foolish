# Maven Builder - Quick Reference

## Common Commands

### Build Commands

| Task | Command | When to Use |
|------|---------|-------------|
| Clean parallel build | `mvn clean compile -T 2C` | Fresh start, major changes |
| **Compilation debugging** | `mvn clean compile -T 2C -DskipTests` | **Fixing compilation errors - skip tests!** |
| Incremental build | `mvn compile -T 1C` | Routine development |
| Generate sources | `mvn generate-sources -T 2C` | ANTLR4, protobuf changes |
| Skip tests | `mvn compile -T 2C -DskipTests` | Quick build verification |

### Test Commands

| Task | Command | When to Use |
|------|---------|-------------|
| Parallel tests | `mvn test -T 1C -Dparallel=classes,methods -DthreadCount=4` | Standard test run |
| Single test class | `mvn test -Dtest=ClassName` | Focus on one class |
| Single test method | `mvn test -Dtest=ClassName#method` | Debug specific test |
| Verbose test | `mvn test -Dtest=ClassName -DtrimStackTrace=false` | Need full stack traces |
| Re-run failures | `mvn surefire:test` | After fixing failures |
| **Approval tests** | `mvn test -Dtest=ApprovalTests -Dparallel=classes,methods` | **Parallel approval tests** |
| **Debug approval** | `mvn test -Dtest="ApprovalTests#*case001*"` | **Single approval case** |

### Combined Workflows

| Task | Command | When to Use |
|------|---------|-------------|
| Full clean + test | `mvn clean test -T 2C -Dparallel=classes,methods -DthreadCount=4` | Before commits |
| Verify (with IT) | `mvn clean verify -T 2C -Dparallel=classes,methods -DthreadCount=4` | Complete validation |
| Debug build | `mvn clean compile test -Dtest=MyTest -X > debug.log 2>&1` | Deep debugging |
| **Compile then test** | `mvn compile -T 2C -DskipTests && mvn test -Dparallel=classes,methods` | **After fixing compilation** |

## Flag Reference

| Flag | Meaning | Example |
|------|---------|---------|
| `-T 1C` | 1 thread per CPU core | `-T 1C` |
| `-T 2C` | 2 threads per CPU core | `-T 2C` |
| `-T 4` | Exactly 4 threads | `-T 4` |
| `-Dparallel=classes,methods` | **Parallel test execution by class AND method** | `-Dparallel=classes,methods` |
| `-DthreadCount=4` | Use 4 threads for tests | `-DthreadCount=4` |
| `-Dtest=Pattern` | Run specific test(s) | `-Dtest=*IntegrationTest` |
| `-DtrimStackTrace=false` | Full stack traces | `-DtrimStackTrace=false` |
| `-DskipTests` | **Skip test execution (compilation debugging)** | `-DskipTests` |
| `-X` | Debug output | `-X` |

## Output Locations

| Content | Location |
|---------|----------|
| Compiled classes | `target/classes/` |
| Test classes | `target/test-classes/` |
| Generated sources | `target/generated-sources/` |
| Test XML reports | `target/surefire-reports/TEST-*.xml` |
| Test text reports | `target/surefire-reports/*.txt` |
| Integration tests | `target/failsafe-reports/` |

## Analyzing Test Results

### Find Failed Tests
```bash
cd target/surefire-reports/
grep -l "failures=\"[1-9]" TEST-*.xml
```

### Extract Failure Messages
```bash
for f in $(grep -l "failures=\"[1-9]" TEST-*.xml); do
  echo "=== $f ==="
  grep -A 5 "failure message" "$f"
done
```

### Count Test Results
```bash
# Total tests
grep "tests=" TEST-*.xml | head -1

# Failed tests  
grep -c "failures=\"[1-9]" TEST-*.xml
```

### Get Test Timing
```bash
for f in TEST-*.xml; do
  echo -n "$f: "
  grep "time=" "$f" | head -1 | sed 's/.*time="\([^"]*\)".*/\1/'
done | sort -t: -k2 -rn
```

## Decision Trees

### Which Build Command?

```
What changed?
├── Compilation errors
│   └── mvn clean compile -T 2C -DskipTests (FIX COMPILATION FIRST!)
├── ANTLR4 grammar (.g4)
│   └── mvn clean generate-sources -T 2C && mvn compile -T 2C -DskipTests
│       (then: mvn test -Dparallel=classes,methods -DthreadCount=4)
├── Dependencies in pom.xml
│   └── mvn clean compile -T 2C -DskipTests (verify compilation first)
├── Major refactoring
│   └── mvn clean compile -T 2C
└── Normal code edits
    └── mvn compile -T 1C
```

### Which Test Command?

```
What's the goal?
├── Run all tests normally
│   └── mvn test -T 1C -Dparallel=classes,methods -DthreadCount=4
├── Run approval tests
│   └── mvn test -Dtest=ApprovalTests -Dparallel=classes,methods -DthreadCount=4
├── Debug test failures
│   ├── Multiple failures
│   │   └── mvn test -Dtest=FailedClass1,FailedClass2 -DtrimStackTrace=false
│   ├── Single failure
│   │   └── mvn test -Dtest=FailedClass#method -DtrimStackTrace=false -X
│   └── Single approval case
│       └── mvn test -Dtest="ApprovalTests#*case_name*" -DtrimStackTrace=false
├── Check for regressions
│   └── mvn clean test -T 2C -Dparallel=classes,methods -DthreadCount=4
└── Performance testing
    └── mvn test -Dtest=PerfTest (review XML for timings)
```

## Parallel Execution Strategy

| Scenario | Maven Threads | Test Threads | Rationale |
|----------|---------------|--------------|-----------|
| Compilation debugging | `-T 2C` | N/A (`-DskipTests`) | Focus on fixing compilation first |
| Clean build | `-T 2C` | N/A | Max throughput, fresh start |
| Incremental build | `-T 1C` | N/A | Conservative, reliable |
| Standard tests | `-T 1C` | `threadCount=4, parallel=classes,methods` | Balance speed/stability |
| Approval tests | `-T 1C` | `threadCount=4, parallel=classes,methods` | Parallelize test cases |
| Debug tests | Sequential | `threadCount=1` | Clarity over speed |
| CI/CD | `-T 2C` | `threadCount=4, parallel=classes,methods` | Max speed, reproducible |

## Common Patterns

### Fixing Compilation Errors
```bash
# CRITICAL: Skip tests - focus on compilation only
mvn clean compile -T 2C -DskipTests

# Quick iteration while fixing errors
mvn compile -T 1C -DskipTests

# After compilation succeeds, run tests
mvn test -Dparallel=classes,methods -DthreadCount=4
```

### After Changing ANTLR4 Grammar
```bash
mvn clean generate-sources -T 2C
mvn compile -T 2C
mvn test -Dparallel=classes,methods -DthreadCount=4
```

### Before Git Commit
```bash
mvn clean test -T 2C -Dparallel=classes,methods -DthreadCount=4
```

### Debugging Specific Failure
```bash
# 1. Identify failure
mvn test -T 1C -Dparallel=classes,methods -DthreadCount=4

# 2. Get details
mvn test -Dtest=FailedTest -DtrimStackTrace=false

# 3. Deep dive
mvn test -Dtest=FailedTest#method -DtrimStackTrace=false -X > debug.log 2>&1
```

### Debugging Approval Test
```bash
# 1. Run all approval tests
mvn test -Dtest=ApprovalTests -Dparallel=classes,methods -DthreadCount=4

# 2. Debug specific case by name pattern
mvn test -Dtest="ApprovalTests#*edge_case_42*" -DtrimStackTrace=false

# 3. Debug with full verbose output
mvn test -Dtest="ApprovalTests#*specific_case*" -DtrimStackTrace=false -X
```

### Creating Regression Test
```bash
# 1. Write minimal reproduction
# 2. Verify it fails
mvn test -Dtest=NewRegressionTest

# 3. Fix bug
# 4. Verify test passes
mvn test -Dtest=NewRegressionTest

# 5. Run full suite
mvn test -T 1C -Dparallel=classes,methods -DthreadCount=4
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Parallel build fails, sequential works | Check for non-thread-safe plugins, missing dependencies |
| Tests pass individually, fail in parallel | Shared state, resource contention, test pollution |
| Slow test execution | Check test timings in XML, identify bottlenecks |
| ANTLR4 generation incomplete | `mvn clean`, then `generate-sources -T 2C` |
| Out of memory | Set `MAVEN_OPTS="-Xmx4g"` |
| Flaky tests | Run sequentially, check for timing dependencies |

## Environment Variables

```bash
# Increase Maven memory
export MAVEN_OPTS="-Xmx4g -XX:+UseG1GC"

# Set default Maven options
export MAVEN_OPTS="-Xmx2g"

# Skip tests by default (not recommended)
export MAVEN_SKIP_TESTS=true
```

## Integration with Claude Code

When Claude Code uses this skill, it should:

1. **Assess context**: What files changed? What type of change?
2. **Choose command**: Use decision tree to select appropriate build/test command
3. **Execute**: Run the command
4. **Analyze output**: Scan console and XML files
5. **Iterate**: Narrow focus if failures occur
6. **Preserve learning**: Convert debug code to regression tests

## Pro Tips

1. **Always start with parallelism**: Only go sequential when debugging
2. **Trust the XML**: More reliable than console output
3. **Fix related failures together**: They often share root causes
4. **Keep debug tests**: Convert to regression tests with clear documentation
5. **Clean when unsure**: `mvn clean` eliminates mysterious issues
6. **Monitor resources**: Use `htop` to check CPU/memory during builds
7. **Group test runs**: Run related tests together for efficiency
8. **Profile slow tests**: Use timing data to find bottlenecks
