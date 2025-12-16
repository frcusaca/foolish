# Advanced Features

## Brane Operations

### Proximity is Combination

Branes combine freely. Listing two branes either in code or by named reference means it is desired
that the two branes be concatenated. The following code snippets have the same meaning in Foolish:

```foolish
{
	a=1;
	b=2;
}
```

and

```foolish
{
	a=1;
}
{
	b=2;
}
```

This kind of resembles some physical and biological systems where like-kinded objects combine when
they come to close proximity to each other. This principle extends throughout nature: atoms bond to
form molecules, cells aggregate to form tissues, individuals cluster to form communities. In
Foolish, this same principle governs how computational elements interact and combine, making the
language feel natural and intuitive to human programmers who are themselves products of these same
organizational principles.

### The Brane Concatenation

Concatenation operation is associative. Here is an example of branes acting like functions;
naturally, brane concatenation invokes functions in RPN.

```foolish
{
	!! ...
	f={
		y=x*x+2x-1;
	};
	p={x=1+a;}
	q={a=1;}
	result =  q p f !! We called `f` on parameter `p` which was evaluated on `q`.
	!! ...
}
```

One will note that it didn't matter how evaluation occurred—either `(q p) f` or `q (p f)` yields
the same `result`:

```foolish
{
	!! ...
	result={
		a=1;
		x=1+a;
		y=x*x+2x-1;
	}
	!! ...
}
```

Another example of using concatenation is derivation, which mimics Object-Oriented programming
inheritance. By default, if we extend a brane with another by post-concatenation, method and member
references follow "static invocation" rules.

```foolish
{
	class_a = { a=1; b=2}
	class_b = class_a {c=3}
	a_b     = class_b
	!! So, class b is inheriting everything from a with the addition of `c`
}
```

The only way to create the effect of dynamic invocation is to create interfaces and specify them at
programming time:

```foolish
{
	animal         = {kindom='Animalia';}
	mouthed_animal = animal {greet='hello';};
	cat            = mouthed_animal {greet='mewo'};
	dog            = mouthed_animal {greet='bark'};
	pet            = {play={greet;greet;greet;};};
	my_pet         = cat pet;
	your_pet       = dog pet;
}
```

## Search System

Once we have established a bundle of named value expressions that we call a brane, we expect there
to be some order and conceptual meaning to this collection—there is meaning in it and there is
meaning in their collection together. It may become useful for us to inspect this brane for some
particular aspect that interests us. We accomplish this through Foolish's revolutionary search
system, which treats code as a searchable, queryable structure rather than a linear sequence of
instructions.

The search system recognizes that programmers spend much of their time navigating and querying
their code structures. Instead of forcing you to remember exact variable names or scroll through
long files, Foolish lets you search for things by pattern, by value, by computational relationship,
or by position. This transforms programming from a process of precise recall to one of intelligent
exploration and discovery.

Brane search expresses are always written postfix except for during naming.

### Name Search

Conventional regular expressions can be used to search the namespace of a brane, treating the brane
as a text document of names. After each search, if the search continues, the cursor is placed at
the beginning of the name that was found.

* Backward search from a cursor is after the last character in the brane's name document.
  * `.` means to find the last name matching the regexp: `brane.result` means the last value to be
    named `result` in the brane. This interpretation has a similar effect as the normal
    dereferencing symbol `.` used in other languages.
  * `$` means to find just the last value in a brane. `{result=10;}$` would extract the value 10.
  * `b#-5` means to find the fifth-to-last value irrespective of names. The last entry in a brane
    is `#-1` per modern negative array indexing convention.
* Forward search from a cursor that is before the first character of the brane's name document.
  * `/` means to find the first name matching the regular expression: `f/parameter` gives us the
    parameter value an evaluation of `f` used.
  * `^` means the very first value in the brane.
  * `b#5` means the SIXTH value in the brane. We use zero-based array indexing for non-negative
    indexes as per convention.

In all cases of single regular expression matches, match failure is a compilation error if possible
and a runtime error if it occurs.

Accessing `b$x` is also referred to as accessing the `x` coordinate of `b`. See the
[Relational Coordinates](RELATIONAL_COORDINATES.md) section for expanded discussion.

### Cursor-Based Search: `?` and `??`

Foolish provides cursor-based search operators that treat the brane as a text document with a cursor
position. The search semantics depend on where the cursor is positioned within the brane's namespace.

#### Localized Search: `?`

The `?` operator performs a **localized search** within a single brane without looking at parent
branes. When anchored to a brane `a`, the search `a?pattern` searches only within brane `a` for
names matching the pattern, ignoring any parent contexts.

**Note**: The `.` dereference operator is an alias for anchored localized search. So `a.x` is
equivalent to `a?x` when `x` is a literal identifier.

```foolish
{
	x = 100;
	inner = {
		y = 20;
		z = 30;
	};
	!! Anchored localized searches (only look inside 'inner')
	r1 = inner?y;     !! Finds y inside inner: r1 = 20
	r2 = inner.z;     !! Same as inner?z: r2 = 30
	r3 = inner?x;     !! NOT FOUND (doesn't search parents): r3 = ???
}
```

When unanchored, `?pattern` performs a localized search in the current brane's scope (the brane
where the search expression is written), searching backwards from the cursor position without
looking at parent branes.

```foolish
{
	outer = {
		a = 1;
		inner = {
			b = 2;
			r1 = ?b;   !! Unanchored: searches current (inner) scope: r1 = 2
			r2 = ?a;   !! Unanchored: 'a' not in current scope: r2 = ???
		};
	}
}
```

#### Globalized Search: `??`

The `??` operator performs a **cursor-based globalized search** that searches upward through parent
branes. When anchored to a brane, `a??pattern` means:

1. Place the cursor at the end of brane `a` (after the semicolon of the last statement)
2. Resolve `pattern` as if it is being referred to by a statement at that cursor position
3. Search backwards from the cursor, then upward through parent branes if not found

```foolish
{
	x = 100;
	y = 200;
	outer = {
		a = 10;
		inner = {
			b = 20;
			c = 30;
		};
		!! Anchored globalized searches (cursor positioned at end of 'inner')
		r1 = inner??c;    !! Finds c at end of inner: r1 = 30
		r2 = inner??a;    !! Searches up from end of inner, finds a: r2 = 10
		r3 = inner??y;    !! Searches up twice, finds y: r3 = 200
	}
}
```

When unanchored, `??pattern` performs default variable lookup - searching backwards from the
current cursor position and then upward through parent branes. This is equivalent to normal
identifier resolution in Foolish.

```foolish
{
	x = 100;
	outer = {
		a = 1;
		b = ??a;     !! Unanchored: default variable lookup, finds a: b = 1
		c = a;       !! Normal reference: same behavior as ??a
		d = ??x;     !! Searches up to parent, finds x: d = 100
	}
}
```

The unanchored `??` exists for consistency and explicitness but provides the same semantics as
normal identifier resolution.

#### Multi-Search: `?*`

The `?*` operator is a **multi-search** (or bulk search) operator that returns a brane containing
**all** results matching the pattern, rather than just the first match. This is useful when you
want to collect multiple values that match a regex pattern.

**Syntax**: `brane?*pattern` performs a regex search within the brane and returns a new brane
containing all matching name-value pairs.

```foolish
{
	doc = {
		tmp_a = 4;
		result = tmp_a * 2;
		tmp_b = 9;
		output = tmp_b * 3;
		tmp_c = 16;
	};
	!! Multi-search for all names starting with 'tmp_'
	temps = doc?*tmp_.*;
	!! temps = {tmp_a = 4; tmp_b = 9; tmp_c = 16;}
}
```

The `?*` operator can be combined with anchored or unanchored forms:

```foolish
{
	x_1 = 10;
	x_2 = 20;
	data = {
		x_3 = 30;
		x_4 = 40;
		y_1 = 50;
	};
	!! Anchored multi-search (searches only inside 'data')
	all_x = data?*x_.*;     !! {x_3 = 30; x_4 = 40;}

	!! Unanchored multi-search (searches current scope only)
	outer_x = ?*x_.*;       !! {x_1 = 10; x_2 = 20;}
}
```

**Note**: The `?*` operator performs **localized** multi-search (does not search parent branes).
For a globalized multi-search that includes parent scopes, combine with the globalized search
semantics (implementation-dependent).

**Common Use Cases**:
- Collecting temporary variables: `brane?*tmp_.*`
- Gathering test results: `results?*test_.*`
- Extracting configuration values: `config?*debug_.*`
- Filtering brane members by naming convention

### Search Paths

#### Backward Search

Since the Foolish document is read top to bottom, it is not possible to search downward past the
current position. Therefore, these symbols are not permitted in an unanchored search: `↓`, `#@+1`,
`→`, `#+1`. However, if an existing search expression is in progress, it is possible to anchor
these searches to a previous unanchored search result. For example, `??_*result#+1` is permitted.
Backward search is conducted in the following manner to discover their occurrences:

1. Start with the cursor at the beginning of the current line.
2. Search the current brane backwards from the cursor. Return if found.
3. If not found, search raises the cursor to the current brane's parent expression; the cursor is
   placed at the beginning of that line.
4. Go to step 2.
5. If no parent brane, then search returns `???`.

#### Forward Search

Forward search is performed line by line to the end of the brane. Forward search is not permitted
to raise the cursor to parent branes. So forward search is limited to the current brane only.

#### Depth Search

- [ ] TODO: Extend search into the computation graph with more syntactical capabilities for finding
  computational results.

## Value Search

Forward value search is triggered by the `:` operator. It is a forward search operator on a brane
that searches for a value that can be considered equivalent to the search parameter. `::` is the
bulk forward value search operator.

```foolish
{
	doc={
		tmp_a = 2*2;
		c     = tmp_a sqrt;
		tmp_b = 1+3;
		d     = tmp_b cbrt;
	}
	r = doc:4;    !! r = 2*2;
	r2= doc::4;   !! r2 = {tmp_a=2*2; tmp_b=1+3;}
	doc:4 = 10    !! `tmp_a = 10;`
}
```

Single and bulk backward value search are triggered by `<:` and `<<:` respectively. The part of the
search result that is used depends on the context. When used on the RHS of a naming expression, the
value is used. When used on the LHS of a naming expression, the name is used.

### Expression Search

- [ ] TODO: Extend search into the computation graph with more syntactical capabilities for finding
  computational results.

### Search Combinations

- [ ] TODO: Document combinations of name and value search.

## Search Cursor Movements

Recall each search result puts the cursor at the start of the line just before variable names. We
now define a few operators to move that search cursor.

| Expressions| Effect on Cursor|
|:----------:|:---------------:|
| `↑`,`#@-1` | Moves cursor to the beginning of the line referring to this brane |
| `↓`,`#@+1` | Moves cursor to the beginning of the brane that is the value of this line|
| `←`,`#-1`  | Moves cursor to the start of previous line|
| `→`,`#+1`  | Moves cursor to the end of this line, effectively at the start of the next line |

So we could use stack based syntax if we so desire:

```foolish
{
	f = {result=↑#-1 + ↑#-2}; !! See below detachment for clarification
	0;
	1;
	f$; !! 1
	f$; !! 2
	f$; !! 3
	f$; !! 5
	f$; !! 8
	f$; !! 13
}
```

For now we forbid the use of unanchored `#0` searching to the beginning of the current line. The
only permitted recursive call is the `↑` operator. An expression that contains recursion must be an
expression with a brane that refers to the line in the parent brane where it is coordinated.

- [ ] TODO: Document searching upward for variables scoped in parent branes.

## Search Result
Search results in the value found at the cursor unless it is marked with `:` in which case, the search results in the name resolution context at the cursor, and `|` which results in the name at the cursor.

### Unanchored Brane Search Leads to Self-Awareness

Notice that every time we refer to a named value by name (a.k.a. reference a value by a variable
name), we are actually doing a `.` search on the brane itself and its parents as if the brane ended
before the start of the current line. So all the searches can actually be invoked without following
a brane. In that case, it would mean to search the present brane with the cursor just before the
beginning of the current line.

```foolish
{
	1;
	2;
	c= #-1 + #-2;
}
```

- [ ] TODO: Augment syntax to make it easier to read for humans and possible to parse for
  compilers.

### Naming a Search Result

With the introduction of search, we now have the ability to name results of search. A shorthand is
introduced to make it easier to extract single values out of branes. The `=` naming operator may be
followed by a brane search expression and then by the brane. The meaning is that we will apply the
search on the RHS brane before assigning that value to the name. The brane `{ V =E B }` where `V`
is a name and `E` is a brane search expression, and `B` is a brane (NB: no space between `=E`).
That expression means `{V = B E}`.

Recall the brane concatenated function invocation above; we have changed one line, which is that
during the assignment of result, we have added a brane search expression (`$`) right after the
naming operator (`=`). This puts the actual number into the result instead of the brane that
computed the number.

```foolish
{
	!! ...
	f={
		y=x*x+2x-1;
	};
	p={x=1+a;}
	q={a=1;}
	result =$  q p f !! result is now the actual `result=7;`

	!! ... A few more examples
	calculation =  q p f !! result is now the actual `result=7;`
	secondary_result =.x calculation
	secondary_result =x calculation
	secondary_result = calculation.x
	result = calculation$
}
```

The flexibility of expression helps the Foolish user greatest flexibility in extracting use
information from the right place, and at the same time place emphasis on code symbols to maximize
readability and communication efficiency.

## Detachment and Parameters

Humans have long sought detachment from the material world. Even though Foolish is inherently
detached and completely abstract, we find that we need to perform further detachment.

### Named Detachment

```foolish
[
	a = ???;
	b = ???;
]
```

This brane detaches the names a and b. So to declare a function with formal parameter names that
might already be in use, one could perform detachment during a naming expression.

```foolish
{
	fn=[
		a=???;
		b=???;
	]{
		result=a+b;
	}
}
```

The detachment is left associative with detachment branes and right associative with branes--It
overwrites detachment branes on the left, and affects brane on it's right.

The original example of function definition probably needed this detachment for it to be a useful
function:

```foolish
{
	f = [↑=???]{result=↑#-1 + ↑#-2};
}
```

Ideally, an upward search like this will block out ALL references to variables from the parent
brane and above. So that in this function

```foolish
{
	f = [↑=???]{result=↑#-1 + ↑#-2 - universal_constant};
}
```

the `universal_constant` will be specified by caller as a parameter.

An example of its use:

```foolish
{
	x=1;
	y=2;
	z=3;
	f=[x;z]{ result = x - y; }
	result = [y,x] f
}
```

The detachment brane dissociates the ensuing brane on its right side from the context,
decontextualizing x and y. x and y become unbound symbols. From the timeline perspective, the
detachment snips off dependency DAG edges.

### Complete Detachment

We now introduce complete detachment using name search to completely detach from a large number or
all of the names and values.

```foolish
{
	clean_heap=[tmp_*=???]old_heap_brane
}
```

Or even complete detachment like this:

```foolish
{
	pure_func=[*=???]{...}
}
```

### Detach to Default

In order to mimic default parameters in many modern programming languages, we shall offer a way to
specify detachment that sets not only the expectation for what needs to be provided but also what
happens when that expectation is not fulfilled.

```foolish
...
f=[r=???; pi=3.14]{
	pir = pi * r
	area = pir * r
	circumference = 2*pir
}
...
c       = {r=2} f
c_hires = {r=2, pi=3.14159} f
c_huh   =$ [r=4] f  !! c_huh = 25.12;
```

Note, though, that the binding of RHS within the detachments are again within the scope of the
current brane. The compiler shall give warnings in suspiciously ambiguous cases. The `c_huh` is a
brane now having a default value for r as well, but since the assignment asks for the last entry,
the result is computed and assigned in the referring brane.

When default branes are next to each other, they are left associative—the left-most default brane
is overridden by the default brane to its immediate right first and foremost. Default brane
concatenation always has higher priority than their concatenation with branes. When a default brane
is between branes it is always right-associative—it is applied to the brane on the right side
before other brane operations.

### Detachment Would Have to be Timetraveling Unthink Operator ???
What should the following code do?
```foolish
{
	a = 1;
	b = 2;
	c = {a+b};
	d =$ [a=-1,b=-2]c;
}
```
If detachment actually worked, one might think `d=-3`. Since detachment should take the brane `c` and
change it's mind about what a and b are supposed to be? It looks like this detachment facility allows us
to time travel back or "unthink" something that we already thought?

Thankfully, the name operator `=` also coordinatse an exprssion. during the assignment
```foolish
...
   c = {a+b};
...
```

The act of coordinating that foolish brane exprssion to the parent brane has the effect of localizing
references to that line in that brane. Coordinating an expression is like chemically binding a smaller
brane organelle into inside of a larger brane at the sites `a` and `b`. And the only way to prevent that
binding from happening is to use detachment brane to prevent binding of specified coordinates during the
coordination where that detachment brane appears.
```foolish
...
   c = {a+b}; !! is equivalent to c={3} due ordination at this location.
...
```

Detachment blocker branes prevents that from happening.
```foolish
   α = 1;
   β = 2;
   c = [α, β]{α+β}; !! Look ma, I respond to α- and β-blockers, I'm a real boy now !
...
```


## Control Flow

Branching is accomplished using search.

```foolish
{
	ifblock = {
		if x==1 then         !! `if k then` fools to the expression `f'condition=k;`
		result = 1;
		else if x==2 then    !! `else if k then` fools to the expression `f'condition=k;`
		result = 4;
		else                 !! `else` fools to the expression `f'condition=true;`
		result = 2;
		f'condition=10;      !! ERROR, cannot assign to Foolish names
	}
	result = ifblock/f'condition:true→;   !! Search for a true value, then forward cursor to next
	result = condition⇒;                  !! shorthand for `/f'condition:true→`
	result => condition;                  !! shorthand for `/f'condition:true→`
}
```

## Recursion

Since the brane search starts with cursor at beginning of the current line, finding the current
brane is easy with `↑`. So recursion is easy to express:

```foolish
{
	factorial = [n=???]{
		result = if n <= 1
		         then result = 1
		         else [n=n-1]↑$;
	}
	five_fact = [n=5] factorial    !! five_fact = 120;
}
```

### Corecursion

Here, it seems that we often perform loops or other complex operations where a single line performs
updates to several states. The most prototypical example is this Python code:

```python
def fib():
    l = 0
    ll = 1
    yield l;
    yield ll;
    while true:
        l, ll = ll, l + ll
        yield ll
def triangle_number():
.
.
.
while not quit():
    print(next(fib()) * next(triangle_number()))
```

And we proceed to use `fib` while we operate on triangle numbers as well. This Fibonacci counter
continues merrily on its own.

But so far Foolish has been entirely functional.

- [ ] TODO: How do we centrally organize several coordinated state updates in a single line of
  code?

### Mutual Recursion

However, mutual recursion is not directly possible since we cannot search downward past the current
line. The only way to create corecursion is to use an unbound name and pass it into the context:

```foolish
{
	even = [n=???; odd=???]{
		result = if n == 0
		         then result = true
		         else [n=n-1;] odd$;
		         !! - [ ] TODO: Explain why we can't do this:
		         !! else [n=n-1;] →;
	};
	odd  = [n=???]{
		result = if n == 0
		         then result = false
		         else [n=n-1; odd=↑;] even$; !! Already defined above, so we can find it directly
		         !! By Foolish convention, the symbol 'odd' is not yet in scope, so we need to pass
		         !! it by searching backwards for odd function itself.
	};
	is_four_even = [n=4] even{odd=odd} !! is_four_even = true;
	is_five_even = [n=5] even{odd=odd} !! is_five_even = false;
}
```
