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

**Note on brane references in concatenation**: When branes like `q`, `p`, or `f` are referenced by
name in a concatenation, each reference creates a detached clone of the original brane. During
concatenation, these clones are recoordinated with new Ancestral Brane (AB) and Immediate Brane (IB)
from the concatenation context, allowing previously unresolved names to potentially resolve. See
[ECOSYSTEM.md](ECOSYSTEM.md#brane-reference-semantics-detachment-and-coordination) for detailed semantics.

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

Foolish provides a revolutionary search system that treats code as a searchable, queryable
structure. Instead of forcing you to remember exact variable names or scroll through long files,
Foolish lets you search for things by pattern, by value, by computational relationship, or by
position.

### Quick Reference

```foolish
{
	data = {x=10; y=20; tmp_a=1; tmp_b=2;};

	!! Name search
	x_val = data.x;        !! Backward search: last 'x' → 10
	first = data^;         !! First value → 10
	last = data$;          !! Last value → 2

	!! Cursor-based search
	local = data?y;        !! Localized: only in 'data' → 20
	global = data??x;      !! Globalized: searches parents too

	!! Multi-search
	temps = data?*tmp_.*;  !! All matching names → {tmp_a=1; tmp_b=2;}

	!! Value search
	found = data:20;       !! Find value 20 → y=20
}
```

**Search operators**: `.` `$` `^` `#N` `/pattern` for name search; `?` (localized) and `??`
(globalized) for cursor-based search; `?*` for multi-search; `:` `::` for value search; `↑` `↓` `←`
`→` for cursor movements.

For comprehensive documentation on the search system, including detailed semantics of all search
operators, search paths, cursor positioning, and naming search results, see
[Names, Search, and Bound](NAME_SEARCH_AND_BOUND.md#the-search-system).

## Detachment and Parameters

Detachment branes use square brackets `[...]` to create controlled scope boundaries and define
parameters. They dissociate names from their context, making them unbound symbols that must be
supplied by callers.

### Quick Reference

```foolish
{
	!! Named detachment - create function parameters
	fn = [a=???; b=???]{result=a+b;}
	result = {a=10; b=20;} fn  !! result = 30

	!! Complete detachment - remove temporary variables
	clean = [tmp_*=???]old_heap

	!! Detach with defaults
	circle = [r=???; pi=3.14]{area = pi*r*r;}
	c1 = {r=2;} circle           !! Uses default pi=3.14
	c2 = {r=2; pi=3.14159;} circle  !! Overrides default

	!! Detachment bounds globalized search
	f = [↑=???]{result=↑#-1 + ↑#-2;}  !! Blocks parent scope access
}
```

**Key concept**: Detachment branes `[...]` define the **boundary for globalized searches** (`??`).
When `[↑=???]` is used, it prevents the function from accessing parent brane variables—they must be
passed as parameters instead.

**Ordination**: The `=` operator ordinates (coordinates) an expression, binding it to the parent
brane at the referenced sites. Detachment prevents this binding: `c=[α,β]{α+β}` keeps `α` and `β`
unbound, while `c={α+β}` binds them immediately (equivalent to `c={3}` if `α=1` and `β=2`).

For comprehensive documentation on detachment branes, including named detachment, complete
detachment, defaults, how detachment bounds globalized searches, and ordination semantics, see
[Names, Search, and Bound](NAME_SEARCH_AND_BOUND.md#detachment-branes-controlling-scope-boundaries).


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
