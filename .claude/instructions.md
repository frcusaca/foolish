# Project Instructions for Claude Code

## Primary Reference Documentation
- **ALWAYS refer to [README.MD](../README.MD) as the authoritative description of the Foolish programming language**
- The README.MD contains the complete language specification, syntax, and semantics
- When in doubt about language features, consult README.MD first

## Error and Ambiguity Handling
- When you discover an error, ambiguity, or conflict in the language specification:
  1. Document it clearly at the end of README.MD
  2. Include the context where the issue was discovered
  3. Describe what the ambiguity or conflict is
  4. If possible, suggest a resolution

## Code Style Guidelines

### Foolish Language (`.foo` files)
- **Indentation**: Use 4 spaces (NOT tabs)
- **File extension**: `.foo`
- Example:
  ```foolish
  {
      x = 1 + 2;
      {
          y = x * 3;
      };
  }
  ```

### Test Code Formatting
- When writing Foolish code in test files, use **triple-quoted multi-line strings**
- This allows proper indentation and readability
- Format:
  ```java
  verifyApprovalOf("""
      {
          x = 1 + 2;
          y = x * 3;
      }
      """);
  ```

### Java Code
- Follow existing project conventions
- Use descriptive variable and method names
- Add comments for complex logic

## Testing
- All changes must pass the full test suite (`mvn test`)
- Approval tests are in `src/test/resources/`
- When approval tests fail, review the `.received.txt` files before approving
- Only approve changes that are semantically correct

## Build and Development
- Build tool: Maven
- Java version: 25
