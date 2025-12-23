# Names, Search, and Bound: The Foolish Reference System

One very powerful concept we have for abstracting thoughts and thinking of complex matter with complex
properties and interactions is the substitution of the statement or object of consideration with a
name. Names such as `x`, `y`, `foolish`, `programming language`. This is an important concept in
Foolish—we are able to name value expressions, search for them, and control the boundaries of those
searches through detachment.

## Table of Contents

- [Names and Ordinates](#names-and-ordinates)
  - [The Naming Operator](#the-naming-operator)
  - [Coordinates: Names as Navigation](#coordinates-names-as-navigation)
  - [Permitted Name Characters](#permitted-name-characters)
  - [Unnamed Values and ???](#unnamed-values-and-)
- [Scope and Name Resolution](#scope-and-name-resolution)
  - [Scope of Name](#scope-of-name)
  - [Renaming and Static Single Assignment](#renaming-and-static-single-assignment)
  - [Retrospective Search](#retrospective-search)
- [The Search System](#the-search-system)
  - [Name Search](#name-search)
  - [Cursor-Based Search: ? and ??](#cursor-based-search--and-)
  - [Multi-Search: ?*](#multi-search-)
  - [Value Search](#value-search)
  - [Search Cursor Movements](#search-cursor-movements)
  - [Search Results](#search-results)
  - [Unanchored Search and Self-Awareness](#unanchored-search-and-self-awareness)
  - [Naming Search Results](#naming-search-results)
- [Detachment Branes: Controlling Scope Boundaries](#detachment-branes-controlling-scope-boundaries)
  - [Named Detachment](#named-detachment)
  - [Complete Detachment](#complete-detachment)
  - [Detach to Default](#detach-to-default)
  - [How Detachment Bounds Globalized Searches](#how-detachment-bounds-globalized-searches)
  - [Detachment and Ordination](#detachment-and-ordination)
- [Search Paths](#search-paths)
  - [Backward Search](#backward-search)
  - [Forward Search](#forward-search)
  - [Depth Search](#depth-search)

---

## Names and Ordinates

### The Naming Operator

In Foolish, we name value expressions using the naming operator `=`:

```foolish
{
	a = 1;                                    !! Simple name binding
	b = 2;                                    !! Another binding
	c = 3;                                    !! Value assignment
	greeting = "Hello, Foolish world!";      !! String binding
	calculation = a + b + c;                  !! Expression binding
}
```

### Coordinates: Names as Navigation

In Foolish, names are also called *ordinates* or *coordinates*. This terminology reflects the idea
that names serve as navigational reference points within branes, allowing us to locate and access
specific values.

When we think of coordinates, we think of x and y coordinates in a two-dimensional Cartesian graph.
But in reality, almost all relationships that we consider can be modeled as functions of coordinates
in some coordinate system. When we use a name to access a value (such as `brane.x`), we are
accessing that brane's coordinate.

Most often we think of binary attributes, "hot or not", for example; sometimes, an object is
observable to have higher-cardinality coordinates such as SIC or ZIP codes. There could also be
higher-valence relationships, such as `{a,b,c} sorted_by_x` or `{a,b,c} is_median_height`, which
may return a boolean on whether `a.x`, `b.x`, `c.x` is sorted, or `a` is the median of `b` and
`c`, respectively.

A well-coordinated brane should have its members readily accessible by their names or brane search.
See [Relational Coordinates](RELATIONAL_COORDINATES.md) for a detailed discussion of how names
function as coordinates in relational contexts.

### Permitted Name Characters

Names are selected from an alphabet that excludes the reserved symbols of the language. For
intra-identifier word separation, only two symbols are permitted: the underscore `_` (U+005F) and
the narrow non-breaking space (U+202F). The narrow non-breaking space provides visually cleaner
separation when rendering systems support it.

```foolish
{
	a_name=0;          !! Underscore separator
	a&#x202F;name=1;   !! Narrow non-breaking space (U+202F)
}
```

### Unnamed Values and ???

Foolish expressions and values inherently do not have names. They are only marked with names when
the naming operator `=` is used. A brane with three unnamed values:

```foolish
{
	1;
	2;
	3;
}
```

In fact, every unnamed expression is an assignment to an *NK* (Not Knowable, "no-no") name:

```foolish
{1;2;3;}
```

is shorthand for:

```foolish
{
	???=1;
	???=2;
	???=3;
}
```

---

## Scope and Name Resolution

### Scope of Name

Names are scoped to "before the current expression". The current naming operation is not yet in
scope. This is similar to how many programming languages work:

```foolish
{
	a=1;
	a=a+1; !! a=2 (refers to previous a)
	a=a+1; !! a=3 (refers to previous a)
}
```

Such a convention has the direct benefit of delaying recursive concepts until more thought is put
into the line of code.

### Renaming and Static Single Assignment

Foolish permits reusing names. If one were to think of the semantics of static single assignment,
this would be how Foolish interprets a reused name:

```foolish
{
	a=1;
	b=a;
	a=2;
	c=a;
}
```

is equivalent to:

```foolish
{
	a=1;
	b=1;
	a=2;
	c=2;
}
```

NB: we preserve the first assignment `a=1` as part of the brane for historical accuracy.

### Retrospective Search

Foolish implements **retrospective search**: searches backwards in the current brane, then upwards
through parent branes. Names resolve based on proximity: "containment creates organization,
proximity creates combination".

This search pattern is fundamental to Foolish's scope resolution and powers both simple identifier
references and complex search operations.

---

## The Search System

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

Brane search expressions are always written postfix except during naming.

### Name Search

Conventional regular expressions can be used to search the namespace of a brane, treating the brane
as a text document of names. After each search, if the search continues, the cursor is placed at
the beginning of the name that was found.

#### Backward Search from End

The cursor starts after the last character in the brane's name document:

- `.` - Find the last name matching the regexp: `brane.result` means the last value to be named
  `result` in the brane. This interpretation has a similar effect as the normal dereferencing
  symbol `.` used in other languages.
- `$` - Find just the last value in a brane. `{result=10;}$` would extract the value 10.
- `#-N` - Find the Nth-to-last value irrespective of names. The last entry in a brane is `#-1` per
  modern negative array indexing convention.

Example:
```foolish
{
	point = {x=1; y=2; z=3;};
	x_coord = point.x;     !! x_coord = 1
	last = point$;         !! last = 3
	second_last = point#-2; !! second_last = 2
}
```

#### Forward Search from Beginning

The cursor starts before the first character of the brane's name document:

- `/` - Find the first name matching the regular expression: `f/parameter` gives us the first
  parameter value in `f`.
- `^` - The very first value in the brane.
- `#N` - The (N+1)th value in the brane. We use zero-based array indexing for non-negative indexes
  as per convention.

Example:
```foolish
{
	data = {a=10; b=20; c=30;};
	first = data^;      !! first = 10
	second = data#1;    !! second = 20 (zero-indexed)
	b_val = data/b;     !! b_val = 20
}
```

In all cases of single regular expression matches, match failure is a compilation error if possible
and a runtime error if it occurs.

Accessing `b$x` is also referred to as accessing the `x` coordinate of `b`.

### Cursor-Based Search: ? and ??

Foolish provides cursor-based search operators that treat the brane as a text document with a cursor
position. The search semantics depend on where the cursor is positioned within the brane's namespace.

#### Localized Search: ?

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

#### Globalized Search: ??

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

### Multi-Search: ?*

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

### Value Search

Forward value search is triggered by the `?=` operator. It is a forward search operator on a brane
that searches for a value that can be considered equivalent to the search parameter. `?=*` is the
bulk forward value search operator.

```foolish
{
	doc={
		tmp_a = 2*2;
		c     = tmp_a sqrt;
		tmp_b = 1+3;
		d     = tmp_b cbrt;
	}
	r = doc?=4;    !! r = 2*2;
	r2= doc?=*4;   !! r2 = {tmp_a=2*2; tmp_b=1+3;}
	doc:4 = 10    !! `tmp_a = 10;`
}
```

### Context Search
Sometimes, it becomes bulky to perform several searches into a brane where we need to access several
names and values. In such a situation, we may create a brane in a search result context, as if we inserted
the new brane in the middle of that brane (without shifting original line numbers).
```foolish
{
	doc={
	     paragraph = "The unanimous Declaration of the thirteen united States of America, When in the Course of human events, it becomes necessary for one people to dissolve the political bands which have connected them with another, and to assume among the powers of the earth, the separate and equal station to which the Laws of Nature and of Nature's God entitle them, a decent respect to the opinions of mankind requires that they should declare the causes which impel them to the separation."
	    paragraph = "We hold these truths to be self-evident, that all men are created equal, that they are endowed by their Creator with certain unalienable Rights, that among these are Life, Liberty and the pursuit of Happiness.--That to secure these rights, Governments are instituted among Men, deriving their just powers from the consent of the governed, --That whenever any Form of Government becomes destructive of these ends, it is the Right of the People to alter or to abolish it, and to institute new Government, laying its foundation on such principles and organizing its powers in such form, as to them shall seem most likely to effect their Safety and Happiness. Prudence, indeed, will dictate that Governments long established should not be changed for light and transient causes; and accordingly all experience hath shewn, that mankind are more disposed to suffer, while evils are sufferable, than to right themselves by abolishing the forms to which they are accustomed. But when a long train of abuses and usurpations, pursuing invariably the same Object evinces a design to reduce them under absolute Despotism, it is their right, it is their duty, to throw off such Government, and to provide new Guards for their future security.--Such has been the patient sufferance of these Colonies; and such is now the necessity which constrains them to alter their former Systems of Government. The history of the present King of Great Britain is a history of repeated injuries and usurpations, all having in direct object the establishment of an absolute Tyranny over these States. To prove this, let Facts be submitted to a candid world."
	     paragraph = "He has refused his Assent to Laws, the most wholesome and necessary for the public good."
	     ....
	     paragraph = "He has excited domestic insurrections amongst us, and has endeavoured to bring on the inhabitants of our frontiers, the merciless Indian Savages, whose known rule of warfare, is an undistinguished destruction of all ages, sexes and conditions."
	     ...
	     paragraph = "We, therefore, the Representatives of the united States of America, in General Congress, Assembled, appealing to the Supreme Judge of the world for the rectitude of our intentions, do, in the Name, and by Authority of the good People of these Colonies, solemnly publish and declare, That these United Colonies are, and of Right ought to be Free and Independent States; that they are Absolved from all Allegiance to the British Crown, and that all political connection between them and the State of Great Britain, is and ought to be totally dissolved; and that as Free and Independent States, they have full Power to levy War, conclude Peace, contract Alliances, establish Commerce, and to do all other Acts and Things which Independent States may of right do. And for the support of this Declaration, with a firm reliance on the protection of divine Providence, we mutually pledge to each other our Lives, our Fortunes and our sacred Honor."
	     ...
	}

	!! Compute a vectorized representation of just a part of the brane.
	opening_vector $= {doc?="We hold"; vector=summarize_current_brane$}
	grievances_vector $= {doc?=*("^He has"|"^For "); vector=summarize_current_brane$}

	!! Note that's not the same as the following depending on what context summarize_current_brane uses.
	opening_vector $= (doc?="We hold"){vector=summarize_current_brane$}
	grievances_vector $= (doc?=*("^He has"|"^For ")){vector=summarize_current_brane$}
	!! 
}	

### Search Cursor Movements

Recall each search result puts the cursor at the start of the line just before variable names. We
now define a few operators to move that search cursor.

| Expression | Effect on Cursor |
|:----------:|:---------------:|
| `↑`,`#@-1` | Moves cursor to the beginning of the line referring to this brane |
| `↓`,`#@+1` | Moves cursor to the beginning of the brane that is the value of this line|
| `←`,`#-1`  | Moves cursor to the start of previous line|
| `→`,`#+1`  | Moves cursor to the end of this line, effectively at the start of the next line |

So we could use stack based syntax if we so desire:

```foolish
{
	f = {result=↑#-1 + ↑#-2}; !! See Detachment section for clarification
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

### Search Results

Search results in the value found at the cursor unless it is marked with `:` in which case, the
search results in the name resolution context at the cursor, and `|` which results in the name at
the cursor.

### Unanchored Search and Self-Awareness

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

### Naming Search Results

With the introduction of search, we now have the ability to name results of search. A shorthand is
introduced to make it easier to extract single values out of branes. The `=` naming operator may be
followed by a brane search expression and then by the brane. The meaning is that we will apply the
search on the RHS brane before assigning that value to the name. The brane `{ V =E B }` where `V`
is a name and `E` is a brane search expression, and `B` is a brane (NB: no space between `=E`).
That expression means `{V = B E}`.

Recall the brane concatenated function invocation; we have changed one line, which is that
during the assignment of result, we have added a brane search expression (`$`) right after the
naming operator (`=`). This puts the actual number into the result instead of the brane that
computed the number.

```foolish
{
	f={
		y=x*x+2x-1;
	};
	p={x=1+a;}
	q={a=1;}
	result =$  q p f !! result is now the actual `result=7;`

	!! A few more examples
	calculation =  q p f
	secondary_result =.x calculation
	secondary_result =x calculation
	secondary_result = calculation.x
	result = calculation$
}
```

The flexibility of expression helps the Foolish user gain greatest flexibility in extracting useful
information from the right place, and at the same time place emphasis on code symbols to maximize
readability and communication efficiency.

---

## Detachment Branes: Controlling Scope Boundaries

Humans have long sought detachment from the material world. Even though Foolish is inherently
detached and completely abstract, we find that we need to perform further detachment. Detachment
branes use square brackets `[...]` to create controlled scope boundaries.

### Named Detachment

```foolish
[
	a = ???;
	b = ???;
]
```

This brane detaches the names `a` and `b`. So to declare a function with formal parameter names
that might already be in use, one could perform detachment during a naming expression:

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

The detachment is left associative with detachment branes and right associative with branes—it
overwrites detachment branes on the left, and affects branes on its right.

An example of use:

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
decontextualizing `x` and `y`. `x` and `y` become unbound symbols. From the timeline perspective,
the detachment snips off dependency DAG edges.

### Complete Detachment

We now introduce complete detachment using name search to completely detach from a large number or
all of the names and values:

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
happens when that expectation is not fulfilled:

```foolish
f=[r=???; pi=3.14]{
	pir = pi * r
	area = pir * r
	circumference = 2*pir
}

c       = {r=2} f
c_hires = {r=2, pi=3.14159} f
c_huh   =$ [r=4] f  !! c_huh = 25.12;
```

Note, though, that the binding of RHS within the detachments are again within the scope of the
current brane. The compiler shall give warnings in suspiciously ambiguous cases. The `c_huh` is a
brane now having a default value for `r` as well, but since the assignment asks for the last entry,
the result is computed and assigned in the referring brane.

When default branes are next to each other, they are left associative—the left-most default brane
is overridden by the default brane to its immediate right first and foremost. Default brane
concatenation always has higher priority than their concatenation with branes. When a default brane
is between branes it is always right-associative—it is applied to the brane on the right side
before other brane operations.

### How Detachment Bounds Globalized Searches

This is the crucial concept: **detachment branes define the boundary for globalized searches**.
When you use a detachment brane like `[↑=???]`, it creates a scope barrier that prevents the `??`
globalized search from looking beyond it into parent branes.

Consider this example:

```foolish
{
	universal_constant = 42;
	f = [↑=???]{result=↑#-1 + ↑#-2 - universal_constant};
}
```

The `[↑=???]` detachment blocks ALL references to variables from the parent brane and above. So
`universal_constant` will NOT be found in the outer scope—instead, it must be specified by the
caller as a parameter. The detachment creates a **bound** on the search space.

Another example:

```foolish
{
	f = {result=↑#-1 + ↑#-2};  !! Without detachment
}
```

Without detachment, the function can access the parent brane's variables during recursion. But with
detachment:

```foolish
{
	f = [↑=???]{result=↑#-1 + ↑#-2};  !! With detachment
}
```

The detachment bounds the search—`↑` refers only to the function itself for recursion, but cannot
access any other variables from parent scopes unless they are passed as parameters.

### Detachment and Ordination

What should the following code do?

```foolish
{
	a = 1;
	b = 2;
	c = {a+b};
	d =$ [a=-1,b=-2]c;
}
```

If detachment actually worked in a time-traveling way, one might think `d=-3`. Since detachment
should take the brane `c` and change its mind about what `a` and `b` are supposed to be? It looks
like this detachment facility allows us to time travel back or "unthink" something that we already
thought?

Thankfully, the name operator `=` also **ordinates** (coordinates) an expression. During the assignment:

```foolish
c = {a+b};
```

The act of ordinating that foolish brane expression to the parent brane has the effect of
localizing references to that line in that brane. When the brane is first coordinated, it searches
for and resolves all variables it can find using its Ancestral Brane (AB) and Immediate Brane (IB)
context. Ordinating an expression is like chemically binding a smaller brane organelle inside of a
larger brane at the sites `a` and `b`. And the only way to prevent that binding from happening is
to use detachment brane to prevent binding of specified coordinates during the ordination where
that detachment brane appears.

```foolish
c = {a+b}; !! is equivalent to c={3} due to ordination at this location.
```

Detachment blocker branes prevent that from happening:

```foolish
α = 1;
β = 2;
c = [α, β]{α+β}; !! Look ma, I respond to α- and β-blockers, I'm a real boy now!
```

**Note on brane references**: When a brane name like `c` is referenced later in another assignment,
a clone of that brane is detached from its original AB and IB and recoordinated with new AB/IB from
the referencing location. This allows previously failed searches to resolve in the new context. For
detailed semantics, see [ECOSYSTEM.md](ECOSYSTEM.md#brane-reference-semantics-detachment-and-coordination).

---

## Search Paths

### Backward Search

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

### Forward Search

Forward search is performed line by line to the end of the brane. Forward search is not permitted
to raise the cursor to parent branes. So forward search is limited to the current brane only.

### Depth Search

- [ ] TODO: Extend search into the computation graph with more syntactical capabilities for finding
  computational results.

---

## Summary

The Foolish name, search, and detachment system creates a coherent model for:

1. **Naming**: Using `=` to create ordinates/coordinates that serve as navigational points
2. **Searching**: Multiple operators (`.`, `?`, `??`, `?*`, `??*`, `/`, `//`, `/*`, `//*`) to find values by name,
   pattern, or value
3. **Bounding**: Using detachment branes `[...]` to control scope boundaries and limit the reach of
   globalized searches

Together, these features enable Foolish to be both highly expressive and properly scoped, allowing
programmers to write code that is simultaneously flexible and controlled. The search system
transforms programming from precise recall to intelligent exploration, while detachment ensures
that scope boundaries remain clear and intentional.
