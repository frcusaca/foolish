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
  load_skills=["rust-debugging"],
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

Where `commands.txt` contains one GDB command per line. GDB executes each command in order, then exits. This is the mode used by sub-agents and scripts.

### rust-gdb (better Rust type display)

```bash
rust-gdb ./my_program
```

`rust-gdb` wraps GDB with Rust-aware pretty-printers. It renders `Vec`, `String`, `Option`, `HashMap`, and other standard types in a human-readable form. Use this when you need to see the contents of collections, not just their raw memory layout.

### Set up GDB for Rust

Add these to the top of every GDB session or command script:

```gdb
set pagination off
set confirm off
```

- `set pagination off` — don't pause after each screen of output (essential for batch mode)
- `set confirm off` — don't ask "are you sure?" on quit/delete (essential for batch mode)

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

GDB confirms:

```
Breakpoint 1 at 0x2d7615: file foolish-ubca/src/fir_kinds.rs, line 2850.
```

### Break at a trait method implementation

Use the fully-qualified demangled name with angle brackets:

```gdb
break "<foolish_ubca::fir_kinds::BraneFir as foolish_ubca::fir_trait::Fir>::fir_op_step"
```

GDB confirms:

```
Breakpoint 2 at 0x2b05ae: file foolish-ubca/src/fir_kinds.rs, line 787.
```

### Break at a specific file and line

```gdb
break foolish-ubca/src/fir_kinds.rs:787
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

Break only when a condition is true. First set the breakpoint, then attach a condition:

```gdb
break foolish_ubca::fir_kinds::tests::step_to_settled
condition 3 1
```

Here `3` is the breakpoint number (from `info breakpoints`), and `1` is the condition expression. The breakpoint fires only when the condition is true. GDB shows:

```
Num     Type           Disp Enb Address            What
3       breakpoint     keep y   0x00000000002d7615 in step_to_settled
	stop only if 1
```

You can also set a conditional breakpoint in one line:

```gdb
break my_function if count > 100
```

### Delete breakpoints

```gdb
delete 1          # delete breakpoint number 1
delete            # delete all breakpoints
```

### Enable/disable breakpoints

```gdb
disable 1         # temporarily disable breakpoint 1
enable 1          # re-enable it
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

Output:

```
fir_kinds::tests::operator_add_two_constants: test
fir_kinds::tests::concatenation_nyes_transitions: test
fir_kinds::tests::step_to_settled: test
```

Note: test names do **not** include the crate prefix. Use `fir_kinds::tests::foo`, not `foolish_ubca::fir_kinds::tests::foo`.

### Resume execution

```gdb
continue
```

This resumes until the next breakpoint, watchpoint, or program exit. GDB prints:

```
Thread 2 "fir_kinds::test" hit Breakpoint 1, ...
```

### Run to a specific location (temporary breakpoint)

```gdb
tbreak my_function
continue
```

`tbreak` sets a breakpoint that deletes itself after the first hit. Useful for "run to cursor" style navigation.

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
(gdb) list 2866,2875
2866    fn operator_add_two_constants() {
2867        let a = make_constant_int(3);
2868        let b = make_constant_int(5);
2869        let op = make_operator("+", vec![Rc::clone(&a), Rc::clone(&b)]);
2870        let scope = Scope::empty();

(gdb) next
2868        let b = make_constant_int(5);
(gdb) next
2869        let op = make_operator("+", vec![Rc::clone(&a), Rc::clone(&b)]);
(gdb) next
2870        let scope = Scope::empty();
```

Each `next` executes one line and advances. The function calls (`make_constant_int`, `make_operator`) run to completion without entering them.

### Step into (`step`) — enter the function on the current line

If the current line calls a function, enter that function and stop at its first line.

```gdb
step
```

Example — step into `fir_op_step`:

```gdb
(gdb) next
113         let report = node.borrow_mut().fir_op_step();
(gdb) step
debug_test::FirNode::fir_op_step (self=0x5625213ebd70) at debug_test.rs:62
62              if self.children.is_empty() {
```

GDB entered `fir_op_step` and stopped at its first executable line. Now `self` is in scope and inspectable.

**Caution:** `step` at a `for` loop line enters `Range::into_iter()` (the desugared loop machinery) rather than the loop body. Use `next` to advance past the loop setup, then `step` to enter the function you actually want.

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

**Key detail:** GDB prints the return value automatically. For Rust enums, it shows the variant name (`StepReport::Progress(Nyes::Constant)`), not just a number.

### Continue (`continue`) — resume until the next breakpoint

```gdb
continue
```

Resumes execution until the next breakpoint or watchpoint fires, or the program exits.

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

GDB's Rust support renders enum variants by name, not by number. `$1` is a convenience variable — you can reference it later as `$1`.

### Print a struct and its fields

```gdb
print *self
```

Output:

```
$2 = debug_test::FirNode {
    nyes: debug_test::Nyes::Prembrionic,
    name: alloc::string::String {vec: ... {len: 5}},
    children: alloc::vec::Vec<...> {buf: ... {len: 0}}
}
```

Access individual fields:

```gdb
print self.nyes
print self.name
print self.children.len
```

### Call a method

GDB can execute Rust methods on live objects:

```gdb
call self.core.get_nyes()
```

Output:

```
$3 = 3
```

The return value is a raw integer for enums. To map it: `0=Prembrionic, 1=Embryonic, 2=Braning, 3=Econstanic, 4=Woconstanic, 5=Constant, 6=Independent, 7=Nk`.

You can also call methods that take arguments:

```gdb
call self.core.set_nyes(5)
```

### Print all local variables

```gdb
info locals
```

Output:

```
all_constanic = true
all_independent = false
any_preconstanic = false
any_nk = false
new_nyes = debug_test::Nyes::Constant
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

### Auto-print at every stop (`display`)

```gdb
display self.nyes
display self.children.len
```

These values are printed automatically every time execution stops (after `next`, `step`, `continue`, etc.). Remove with:

```gdb
undisplay 1
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

This switches context to frame 1 (`step_to_settled`) and shows its local variables. Switch back with:

```gdb
frame 0
```

---

## 8. Advanced techniques

### Watchpoints — break when a value changes

```gdb
watch self.nyes
```

This stops execution whenever `self.nyes` is written to. Useful for finding where a variable changes unexpectedly. GDB reports:

```
Hardware watchpoint 2: self.nyes

Old value = debug_test::Nyes::Prembrionic
New value = debug_test::Nyes::Constant
```

### Read watchpoints — break when a value is read

```gdb
rwatch self.nyes
```

Stops when `self.nyes` is read. Useful for tracking who reads a value.

### Catchpoints — break on panics

```gdb
catch throw
```

This stops when a Rust panic is thrown (panics are implemented as `throw` in the unwinding ABI).

### Conditional catchpoints

```gdb
catch throw if $_exception == 1
```

### Repeat the last command

Just press Enter — GDB repeats the previous command. This is especially useful for stepping:

```gdb
(gdb) next
2868        let b = make_constant_int(5);
(gdb)         ← pressed Enter
2869        let op = make_operator("+", vec![Rc::clone(&a), Rc::clone(&b)]);
(gdb)         ← pressed Enter
2870        let scope = Scope::empty();
```

### Examine memory

```gdb
x/10xb &self.nyes    # 10 bytes in hex starting at nyes
x/4xw &self          # 4 words in hex starting at self
x/s &self.name       # print as string
```

### Search memory

```gdb
find /b &self, &self+100, 0x03   # search for byte 0x03
```

### Set a variable

```gdb
set self.nyes = 5
```

This modifies the running program's state. Useful for testing "what if" scenarios without recompiling.

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
list
next
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

## 10. Full example: stepping through a NYES state machine

This is a real session debugging the Foolish evaluation engine. The test creates a brane with two constant children and steps through the NYES state machine.

### The test

```rust
fn concatenation_nyes_transitions() {
    let brane1 = make_brane(vec![
        make_statement("a", 0, make_constant_int(1)),
        make_statement("b", 1, make_constant_int(2)),
    ]);
    let brane2 = make_brane(vec![
        make_statement("c", 0, make_constant_int(3)),
        make_statement("d", 1, make_constant_int(4)),
    ]);
    let cat = make_concatenation(vec![brane1, brane2]);
    let trace = step_to_settled(&cat, &Scope::empty());
    assert_progression(&trace, Nyes::Constant, "Concatenation(extended)");
}
```

### The GDB script

```
set pagination off
set confirm off

# Break at the core evaluation step
break "<foolish_ubca::fir_kinds::ConcatenationFir as foolish_ubca::fir_trait::Fir>::fir_op_step"

run fir_kinds::tests::concatenation_nyes_transitions --exact --test-threads=1

# HIT 1: ConcatenationFir in Prembrionic state
echo === HIT 1: ConcatenationFir::fir_op_step ===\n
list 2381,2400

# Step: check children, set Braning, push child tasks
next
next
next

# Continue to next fir_op_step (a BraneFir being stepped)
continue

echo === HIT 2: BraneFir::fir_op_step ===\n
list 786,808

# Step: check if children empty, set Braning
next
next

# Continue to the Braning state check
continue

echo === HIT 3: BraneFir::fir_op_step (Braning state) ===\n

# Step: check children's NYES → decide new state
next
next
next

continue
quit
```

### Running it

```bash
gdb -batch -x commands.txt target/debug/deps/foolish_ubca-<hash>
```

### What you see

```
=== HIT 1: ConcatenationFir::fir_op_step ===
2382        match self.core.get_nyes() {
2383            Nyes::Prembrionic | Nyes::Embryonic => {
2384                let children: Vec<FirRef> = self.core.foolish_children().to_vec();
...

=== HIT 2: BraneFir::fir_op_step ===
787         match self.core.get_nyes() {
788             Nyes::Prembrionic | Nyes::Embryonic => {
789                 let children: Vec<FirRef> = self.core.foolish_children().to_vec();
...

=== HIT 3: BraneFir::fir_op_step (Braning state) ===
787         match self.core.get_nyes() {
...
```

The debugger stops at each `fir_op_step` call, shows the source code, and lets you step through the NYES state machine line by line.

---

## 11. Key limitation: `self` in trait methods

GDB **cannot see `self`** in trait method implementations (`impl Fir for BraneFir`). This is a GDB limitation with Rust's vtable dispatch.

**What works:**
- `self` is visible in **inherent methods** (`impl FirNode { fn fir_op_step }`) — these don't go through vtable dispatch
- Local variables are visible in both cases

**Workarounds for trait methods:**

1. **Break at the call site instead** — `self` is visible in the caller:
   ```gdb
   break step_to_settled
   # From here, step into fir_op_step and use 'finish' to see return values
   ```

2. **Use the register** — on x86_64, `self` is passed in `$rdi`:
   ```gdb
   info registers rdi
   ```

3. **Use inherent methods for debugging** — write a small wrapper that calls the trait method and break on the wrapper:
   ```rust
   // In your test
   fn debug_step(node: &mut FirNode) -> StepReport {
       node.fir_op_step()  // inherent method — self visible in GDB
   }
   ```

---

## 12. Troubleshooting

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

### Program exits before breakpoints are hit

The test may have passed and exited before your breakpoint was reached. Use `--test-threads=1` and ensure your test name matches exactly (use `--list` to verify).

---

## Last Updated

**Date**: 2026-07-31
**Updated By**: Sisyphus / mimo-v2.5-pro
**Changes**: Comprehensive rewrite with tested examples for all GDB capabilities — breakpoints (function, trait, file:line, conditional), stepping (over/into/out), value inspection (print, call, display, memory), backtrace, watchpoints, catchpoints, and the self-in-trait-method limitation.
