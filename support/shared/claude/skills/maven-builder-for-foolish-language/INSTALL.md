# Maven Builder Skill - Installation Guide

## Installation

1. **Download the skill package**
   - Download `maven-builder-skill.zip`

2. **Extract the skill package**
   ```bash
   # Extract to a location where Claude Code can find it
   # The exact directory depends on your Claude Code configuration
   unzip maven-builder-skill.zip -d /path/to/your/skills/directory/
   
   # Common possibilities (check your Claude Code settings):
   # ~/.claude/skills/
   # ~/.config/claude-code/skills/
   # Or a custom directory you've configured
   ```

3. **Verify installation**
   ```bash
   # Check the directory where you extracted it
   ls /path/to/your/skills/directory/maven-builder/
   # Should see: SKILL.md  README.md  examples/
   ```
   
   **Note**: If you're unsure where Claude Code looks for skills, check your Claude Code 
   configuration or documentation for the skills directory path.

4. **Restart Claude Code** (if running)

## Usage

Once installed, Claude Code will automatically use this skill when:
- Working with Maven projects (detects `pom.xml`)
- Building mixed Java/Scala projects
- Running tests
- Handling ANTLR4 or other source generators
- Debugging test failures

### Manual Invocation

You can also explicitly request Maven operations:

```
"Build the project with parallel compilation"
→ Claude uses: mvn clean compile -T 2C

"Run all tests in parallel"
→ Claude uses: mvn test -T 1C -Dparallel=classes -DthreadCount=4

"Debug the failing PaymentTest"
→ Claude uses: mvn test -Dtest=PaymentTest -DtrimStackTrace=false

"We modified the ANTLR grammar, rebuild everything"
→ Claude uses: mvn clean generate-sources -T 2C && mvn verify -DskipTests -T 2C
```

## Configuration

### Memory Settings

For large projects, configure Maven memory:

```bash
# Add to ~/.bashrc or ~/.zshrc
export MAVEN_OPTS="-Xmx4g -XX:+UseG1GC"
```

### Parallel Execution Defaults

The skill uses sensible defaults:
- Build threads: `-T 2C` (clean builds) or `-T 1C` (incremental)
- Test threads: `-DthreadCount=4`

You can request different settings:
```
"Build with 4 threads exactly"
→ Claude uses: mvn verify -DskipTests -T 4

"Run tests with 8 threads"
→ Claude uses: mvn test -DthreadCount=8 -Dparallel=classes
```

## Verification

Test the skill with a sample Maven project:

```bash
# 1. Create test project
mvn archetype:generate -DgroupId=com.test -DartifactId=test-project \
  -DarchetypeArtifactId=maven-archetype-quickstart -DinteractiveMode=false

cd test-project

# 2. Ask Claude Code to build it
# "Build this project with parallel compilation"

# 3. Claude should execute
mvn clean compile -T 2C

# 4. Ask Claude Code to run tests
# "Run the tests in parallel"

# 5. Claude should execute  
mvn test -T 1C -Dparallel=classes -DthreadCount=4
```

## Customization

### Modifying Thread Counts

Edit `SKILL.md` to change default thread counts:

```markdown
# Change from:
mvn clean compile -T 2C

# To:
mvn clean compile -T 4  # Use exactly 4 threads
```

### Adding Custom Scenarios

Add your own examples to `examples/scenarios.md`:

```markdown
## Scenario X: Your Custom Workflow

**Situation**: Describe your specific case

**Workflow**:
```bash
# Your custom Maven commands
```

### Project-Specific Profiles

For projects with specific Maven profiles:

```
"Build with the production profile"
→ Claude can use: mvn clean compile -T 2C -Pprod

"Run integration tests with staging profile"
→ Claude can use: mvn verify -Pstaging -Dparallel=classes
```

## Troubleshooting Installation

### Skill not detected
1. Check where Claude Code expects skills (consult Claude Code documentation)
2. Verify `SKILL.md` exists in the correct directory
3. Check file permissions (should be readable)
4. Restart Claude Code

### Commands not working as expected
1. Check Maven is installed: `mvn --version`
2. Verify project has `pom.xml`
3. Review `SKILL.md` for command reference

### Performance issues
1. Reduce thread count: use `-T 1C` instead of `-T 2C`
2. Increase Maven memory: `export MAVEN_OPTS="-Xmx4g"`
3. Check system resources: `htop` or `top`

## Uninstallation

```bash
# Remove from wherever you installed it
rm -rf /path/to/your/skills/directory/maven-builder/
```

## Support

For issues or enhancements:
1. Review the examples in `examples/scenarios.md`
2. Check the quick reference in `examples/quick-reference.md`
3. Consult the full documentation in `SKILL.md`

## Updates

To update the skill:
1. Download the new version
2. Remove old version from your skills directory
3. Extract new version to the same location
4. Restart Claude Code

## Compatibility

- **Claude Code**: All versions with skill support
- **Maven**: 3.3+ (for parallel build support)
- **JDK**: 8+ (Java projects)
- **Scala**: 2.12+ (if applicable)
- **OS**: Linux, macOS, Windows with WSL

## Tips for Effective Use

1. **Let Claude assess the context**: It will choose appropriate commands
2. **Request parallel by default**: Claude uses `-T 1C` or `-T 2C` automatically
3. **Trust the debugging workflow**: Claude narrows from broad to specific
4. **Review regression test suggestions**: Claude may suggest preserving debug code
5. **Check XML reports**: Claude analyzes `target/surefire-reports/` for details

## Advanced Configuration

### Custom Maven Repository
```bash
# Add to ~/.m2/settings.xml
<settings>
  <mirrors>
    <mirror>
      <id>custom-repo</id>
      <url>https://your-repo.com/maven2</url>
      <mirrorOf>central</mirrorOf>
    </mirror>
  </mirrors>
</settings>
```

### Plugin-Specific Settings
Edit your project's `pom.xml` for project-specific behavior:

```xml
<build>
  <plugins>
    <plugin>
      <groupId>org.apache.maven.plugins</groupId>
      <artifactId>maven-surefire-plugin</artifactId>
      <configuration>
        <parallel>classes</parallel>
        <threadCount>4</threadCount>
      </configuration>
    </plugin>
  </plugins>
</build>
```

## Next Steps

After installation:
1. Try the examples in `examples/scenarios.md`
2. Reference `examples/quick-reference.md` for common commands
3. Explore the full documentation in `SKILL.md`
4. Start building your Maven projects with Claude Code!

---

**Note**: This skill enhances Claude Code's existing Maven knowledge with structured parallel build strategies, test debugging workflows, and regression test preservation patterns.
