---
name: rust-debugging
description: "MUST USE for debugging Rust programs with GDB — stepping through code, inspecting values, setting breakpoints, diagnosing panics, wrong values, state machine bugs. Covers building with debug symbols, breakpoint types (function, file:line, conditional), step-into/over/out, value inspection, running test binaries, and Rust-specific GDB techniques. Triggers: 'debug Rust', 'gdb', 'breakpoint', 'step through', 'inspect value', 'debug test', 'debugger', 'single step', 'watchpoint', 'backtrace', 'debug symbols', 'how to debug'."
---

# Rust Debugging with GDB

GDB is the standard debugger for Rust on Linux. This skill covers the complete workflow: building with debug symbols, setting breakpoints, stepping through code, inspecting values, and running test binaries.

---

## 0. Use a sub-agent for debugging

GDB output is verbose — backtraces, memory dumps, and stepping logs quickly fill a context window. **Delegate debugging to a sub-agent** to keep the main context clean and focused on the bigger task.

The sub-agent loads this skill, runs GDB, interprets the output, and returns a concise summary of findings:

```
task(
  category="deep",
  load_skills=["rust-debug"],
  prompt="Debug why concatenation_nyes_transitions produces Constant instead of NK.
         The test is in foolish-ubca/src/fir_kinds.rs. Break at fir_op_step,
         step through the Braning branch, and report what nyes state each child
         has and what decision _decide_nyes_due_to_children makes."
)
```

The sub-agent returns a summary like:

> "The NK child's `is_constanic()` returns true (NK is a terminal state), so
> `all_constanic` is true, and the brane transitions to Constant — not NK.
> The `_decide_nyes_due_to_children` function checks `all_constanic` before
> `any_nk`, so Constant wins."

No raw GDB output reaches the main session. The sub-agent absorbs the noise.

---

## 1. Build for debugging

To get full debug info and no inlining, compile with `opt-level=0`:

### For standalone files

```bash
rustc -g -C opt-level=0 -o my_program my_program.rs
```

### For cargo test binaries

The test profile uses `opt-level = 1` which inlines functions. Override it:

```bash
CARGO_PROFILE_TEST_OPT_LEVEL=0 cargo test -p <crate> --lib --no-run
```

This produces a binary at `target/debug/deps/<crate>-<hash>`. Find the exact path:

```bash
ls -lt target/debug/deps/<crate>-* | head -5
```

### For cargo binaries (non-test)

```bash
CARGO_PROFILE_DEV_OPT_LEVEL=0 cargo build
```

The binary lands at `target/debug/<binary_name>`.

---

## 2. Start GDB

### Interactive session

```bash
gdb ./my_program
```

### Batch mode (scripted, for automation)

```bash
gdb -batch -x commands.txt ./my_program
```

Where `commands.txt` contains one GDB command per line.

### rust-gdb (better Rust type display)

```bash
rust-gdb ./my_program
```

`rust-gdb` wraps GDB with Rust-aware pretty-printers. It renders `Vec`, `String`, `Option`, `HashMap`, and other standard types in a human-readable form.

---

## 3. Set breakpoints

### Break at a function by name

```gdb
break my_module::my_function
```

Example — break at a test helper:

```gdb
break foolish_ubca::fir_kinds::tests::step_to_settled
```

### Break at a trait method implementation

Use the fully-qualified demangled name with angle brackets:

```gdb
break "<foolish_ubca::fir_kinds::BraneFir as foolish_ubca::fir_trait::Fir>::fir_op_step"
```

### Break at a specific file and line

```gdb
break src/fir_kinds.rs:787
```

### Find the right symbol name

List all symbols matching a pattern:

```bash
nm ./my_program | c++filt | grep my_function
```

This shows demangled names like:

```
foolish_ubca::fir_kinds::tests::step_to_settled
<foolish_ubca::fir_kinds::BraneFir as foolish_ubca::fir_trait::Fir>::fir_op_step
```

### Conditional breakpoints

Break only when a condition is true:

```gdb
break src/fir_kinds.rs:787 if nyes == 3
```

Example — break only when a variable equals a specific value:

```gdb
break my_function if count > 100
```

### List all breakpoints

```gdb
info breakpoints
```

### Delete a breakpoint

```gdb
delete 1          # delete breakpoint number 1
delete            # delete all breakpoints
```

---

## 4. Run the program

### Without arguments

```gdb
run
```

### With arguments

```gdb
run arg1 arg2 arg3
```

### Running a specific test (Rust test binaries)

Rust test binaries take the test filter as a positional argument:

```gdb
run fir_kinds::tests::operator_add_two_constants --exact --test-threads=1
```

- `--exact` — match the test name exactly (not a substring)
- `--test-threads=1` — run one test at a time (prevents parallel interference)

### List available tests in a binary

```bash
./my_test_binary --list
```

---

## 5. Stepping through code

### Step over (`next`) — execute the current line, skip into calls

Move to the next line in the same function. If the line calls another function, execute it entirely and stop at the next line.

```gdb
next
```

Or step N lines at once:

```gdb
next 3
```

Example session:

```gdb
(gdb) list 59,65
59          fn fir_op_step(&mut self) -> StepReport {
60              match self.nyes {
61                  Nyes::Prembrionic => {
62                      if self.children.is_empty() {
63                          self.nyes = Nyes::Constant;
64                          StepReport::Progress(Nyes::Constant)
(gdb) next
62                      if self.children.is_empty() {
```

### Step into (`step`) — enter the function on the current line

If the current line calls a function, enter that function and stop at its first line.

```gdb
step
```

Example — step into `fir_op_step`:

```gdb
(gdb) next
113             let report = node.borrow_mut().fir_op_step();
(gdb) step
debug_test::FirNode::fir_op_step (self=0x5625213ebd70) at debug_test.rs:62
62                      if self.children.is_empty() {
```

### Step out (`finish`) — run until the current function returns

Execute the rest of the current function and stop when it returns to the caller.

```gdb
finish
```

Example — step out of `fir_op_step` back to the caller:

```gdb
(gdb) finish
Run till exit from #0  debug_test::FirNode::fir_op_step (self=...) at debug_test.rs:62
debug_test::step_to_settled (node=...) at debug_test.rs:113
Value returned is $1 = debug_test::StepReport::Progress(debug_test::Nyes::Constant)
```

The return value is printed automatically.

### Continue (`continue`) — resume until the next breakpoint

```gdb
continue
```

---

## 6. Inspect values

### Print a variable

```gdb
print self.nyes
```

Output:

```
$1 = debug_test::Nyes::Prembrionic
```

### Print a struct field

```gdb
print self.name
print self.children.len
```

### Print with debug format (`{:?}`)

```gdb
print self.nyes
```

GDB's Rust support renders enum variants by name.

### Dereference a pointer

```gdb
print *self
```

### Call a method (GDB can execute Rust methods)

```gdb
call self.core.get_nyes()
```

### Print all local variables

```gdb
info locals
```

### Print function arguments

```gdb
info args
```

### Print the current source location

```gdb
list
```

Or list a specific range:

```gdb
list 59,80
```

---

## 7. Call stack

### Show backtrace

```gdb
bt
```

Output:

```
#0  debug_test::FirNode::fir_op_step (self=0x5625213ebd70) at debug_test.rs:62
#1  debug_test::step_to_settled (node=0x7ffd05bf0b68) at debug_test.rs:113
#2  debug_test::main () at debug_test.rs:138
```

### Show N frames

```gdb
bt 3
```

### Select a frame (to inspect its locals)

```gdb
frame 1
info locals
```

---

## 8. Advanced techniques

### Watchpoints — break when a value changes

```gdb
watch self.nyes
```

This stops execution whenever `self.nyes` is written to. Useful for finding where a variable changes unexpectedly.

### Catchpoints — break on panics

```gdb
catch throw
```

This stops when a Rust panic is thrown.

### Conditional breakpoints with complex expressions

```gdb
break fir_op_step if self.children.len > 0
```

### Repeat the last command

Just press Enter — GDB repeats the previous command.

### Examine memory

```gdb
x/10xb &self.nyes    # 10 bytes in hex starting at nyes
```

---

## 9. Running tests under GDB — complete workflow

Here is the full sequence for debugging a failing test:

### Step 1: Find the test binary

```bash
CARGO_PROFILE_TEST_OPT_LEVEL=0 cargo test -p foolish-ubca --lib --no-run 2>&1 | tail -3
```

Output:

```
Finished `test` profile [unoptimized + debuginfo]
Executable unittests src/lib.rs (target/debug/deps/foolish_ubca-<hash>)
```

### Step 2: List available tests

```bash
target/debug/deps/foolish_ubca-<hash> --list 2>&1 | grep my_test
```

### Step 3: Create a GDB command script

```
# commands.txt
set pagination off
set confirm off

# Break where you want to stop
break "<foolish_ubca::fir_kinds::BraneFir as foolish_ubca::fir_trait::Fir>::fir_op_step"

# Run the specific test
run fir_kinds::tests::my_test_name --exact --test-threads=1

# At each breakpoint: show where we are, inspect state
print self.nyes
print self.name
list

# Step through
next
next

# Continue to next breakpoint
continue

quit
```

### Step 4: Run GDB with the script

```bash
gdb -batch -x commands.txt target/debug/deps/foolish_ubca-<hash>
```

---

## 10. Troubleshooting

### "No symbol 'self' in current context"

GDB sometimes cannot see `self` in trait method implementations (`impl Fir for BraneFir`). Two workarounds:

1. **Break at inherent methods instead** — `self` is visible in `impl BraneFir { fn fir_op_step(...) }` when compiled without optimizations.

2. **Use the register directly** — on x86_64, `self` is passed in `$rdi`:
   ```gdb
   set $self = (BraneFir*)$rdi
   print $self->core
   ```

### "No compiled code for line N"

The line is not executable (comment, blank, doc comment). Use the next executable line number. To find it:

```gdb
list my_function
```

### Breakpoint not hit

Verify the breakpoint is resolved:

```gdb
info breakpoints
```

If it says "pending", the symbol name is wrong. Find the correct name:

```bash
nm ./my_binary | c++filt | grep my_function
```

### `next` steps into stdlib code

This happens at `for` loop boundaries (desugared to `Range::into_iter()`). Use `next` again to advance, or set a breakpoint on the next interesting line and `continue`.

---

## Last Updated

**Date**: 2026-07-31
**Updated By**: Sisyphus / mimo-v2.5-pro
**Changes**: Initial creation — comprehensive GDB debugging skill for Rust, covering build flags, breakpoints, stepping, value inspection, test binary debugging, and troubleshooting.
