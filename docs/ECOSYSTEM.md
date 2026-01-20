# Ecosystem

## It/Them
Foolish ecosystem has available to itself most of itself. To start, the source code of foolish is always
available. When it is not clear, Foolish code is the source program as we type it between "```follish ```" quotes. The Foolish AST is created by a Foolish parser. AST's are available to the Foolish systems, one can refer to the AST from Foolish code, as well, it is available later in the Foolish computer. A machine, let's call it the foolish brane computer, is used to translate Foolish AST into Foolish internal representations. Therefore branes have source code, always finite length Strings, AST, and Foolish Iternal Representation. The user almost always interact with the Foolish system using Foolish language.

## Computer Reading Branes

When a computer encounters a `.foo` file, it shall parse the Foolish into an AST. The units on
which the computer works are the expressions. The largest expression being a brane, its processing
takes up much of the Foolish ecosystem.

## The Unicellular Brane Computer (UBC)

Like our human minds, most of our computational resources have limited abilities. A Unicellular
Brane Computer (UBC) is one that has just enough capacity to hold the AST of a brane, and the
ability to interpret and understand a single expression at a time. This computer proceeds simply
from the beginning of the brane to the end of the brane, evaluating and creating new values to be
stored in the brane. In order to process expressions, the brane actually has two more sources of
information: Ancestral and Immediate.

### Ancestral Brane (AB) and Immediate Brane (IB)

Given a brane statement, the **Immediate Brane (IB)** is the current context that we have accumulated
inside the Unicellular Brane Computer so far (lines before current expression), not including any
definitions made in the current statement. The **Ancestral Brane (AB)** is the search context
immediately before and outside of the brane where a statement is being made. AB is a linearization of
the nesting of brane statements all collapsed into a single brane.

### Brane Reference Semantics: Identification, Ordination, and Coordination

**Three Key Concepts:**

1. **Identification**: An assignment statement **identifies** an expression with a name using the `=` operator. The RHS expression is **identified** when the LHS identifier is assigned. We most frequently talk about "brane identification."

2. **Ordination**: When an assignment is evaluated in a brane, the identified expression becomes **ordinated** to that brane—it becomes part of the brane. The brane gains an **ordinate** (the LHS identifier becomes the name of an axis, a new dimension, just like x and y are names in a 2D Cartesian system). During ordination, the expression searches for and resolves names using its AB and IB context.

3. **Coordination**: During UBC compute resolution stage, an expression located in a brane becomes **coordinated** with other ordinates of the brane and ancestral branes. It becomes coordinated because it has reacted to the other ordinates. Most expressions are only partially coordinated (with unresolved identifiers) during intermediate evaluation stages.

**When a brane is first ordinated** (identified and assigned to a name), it has already searched for and resolved all
the variables it can find in its original AB and IB context, leaving it with:
- Names bound to values (successfully resolved and coordinated)
- Names with incomputable values due to failed searches (free variables, not fully coordinated)

This example brane may change value when referenced in a statement—it is constantic. When an
assignment statement refers to a brane by name, a clone of that constantic brane is **detached** from
its original AB and IB and **recoordinated** into the new position. During recoordination:
- **New AB**: The brane it is being ordinated into
- **New IB**: All lines preceding the assignment

This allows previously failed searches to potentially resolve in the new context, so the constantic
brane can gain value and potentially achieve CONSTANT. For detailed operational semantics of how the
UBC evaluates brane source code with combinations of original and new AB/IB, see the UBC brane
evaluation section below.

In some cases, such as the recursive call to `↑`, the search finds a line of code that has an
incomplete value. Note by incomplete value, in the UBC, we mean not the *NK* value `???`, but a
value that the present UBC does not fully know the value of (yet). In the case of this incomplete
value, it would appear that the simplest machine is unable to continue evaluating the value of the
Foolish expression. In fact, we don't even know all of the names or values this brane will capture
by its end. To continue computation, the UBC must stash the current execution context, and evaluate
that expression using the context "IB AB ↑". That says the UBC shall evaluate the AST of the
expression that created the present brane, resolving any searches first using the current AB, then
using the current IB.

```foolish
	{...{...{...

		!! At this point, we search up for name resolutions the AB below
		!! refers to this point and back and up
		f = {
			!! Start of IB
			zero = ←; !! program may use this method to require a positional aregument.
			one = 1;
			two = 2;
			!! End of IB
			if (continue) then
				r=[three=zero+one+2;]↑
			else
				r=5
			!! Continue to code that we didn't know when we computed r
			four = 4;
		}
	...}...}..}
```

In that example, the recursive call to compute r is processed by the UBC to the equivalent of:

```foolish
	{
		...
		r = [three=zero+one+2;] IB AB !!! the code for f= !!!
		...
	}
```

Because branes are finite, the code on the right side of `f=` is surely parsed to AST and stored in
memory by the time the UBC is required to evaluate this line of Foolish. Therefore in constant
time, the uncoordinated `↑` is translated into an AST expression. This AST expression may further
contain `↑` inside or at its front. But that is for a later iteration of the UBC to evaluate. Each
step taken by the UBC should require finite time.

Generally speaking the context for brane reference is as follows:
```foolish
	{...{..
		!! SS_b1 is the code for this expression, it also has contexts AB_b1 and IB_b1.
		b1 = {...};
		{...{...
			!! The context for this assignment is AB_b2 and IB_b2;

			b2 = p1 p2 p3 b1 p4 p5 p6; 

			!! The code here is `b2 = p1 p2 p3 b1...`, when b1 is discovered to be the brane
			!!  above, it needs to be evaluated, that reference, the `b1` means evaluating the
			!!  search result, SS_b1 with context `AB_b2 IB_b2 AB_b1 IB_b1`. Implementor should
			!!  take care to not create infinite search loops due to circular reference.
	...}
```



To facilitate the multiple steps of UBC evaluation, we define an internal representation known as
the Foolish Internal Representation (FIR). FIR can be several things:

1. A Foolish AST. This is an uncoordinated and unevaluated expression in its original Foolish.
2. A finalized value—integer or brane in CONSTANT state.
3. A comment.
4. An abstract brane that is deliberately or inadvertently undercontextualized—some identifier
   referenced within has no available binding to value or brane member. An optimization we may
   perform is to coordinate the values of an abstract brane where possible. That brane with all
   possible values coordinated is a ***CONSTANTIC*** (constant in context) abstract brane. Normally,
   a recursive brane conditions on some computational result that depends on a parameter—an ordinate
   identifier that is named but unattached at the definition of the brane. In these cases, the
   abstract brane can be constantic because the recursion depends on a parameter. But it is possible
   to construct an abstract brane that calls itself irrespective of future parameters. Such a brane
   will be finite or infinite in depth depending on its construction. The abstract brane may
   therefore be permanently ***NYE*** (*Not Yet Evaluated*, pronounced "nigh", any pre-constantic
   state) due to its construction. An abstract brane is constantic if it has not reached CONSTANTIC
   due to detached identifier names.
5. A NYE FIR represents code that was expressed in Foolish, but it has not reached CONSTANTIC in
   the present UBC context. This FIR contains the AST of the expression, and a reference to the AB
   and IB that were used to attempt evaluation. Possible reasons why the FIR has not reached
   CONSTANTIC:
   1. The AST is unprocessed.
   2. The expression has an unevaluated AST that contains recursive references to names that have
      not reached CONSTANTIC in the present context.

### The UBC pass

1. If the UBC encounters Foolish text representing a brane, UBC holds it in memory as source code.
2. UBC parses the Foolish text into an AST.
3. The AST is compiled into Foolish Internal Representation (FIR). The tree rougly mirrors the AST
with the exception that the FIR has some extra facilities for tracking the process of evaluation as
well as supporting searches. Upon creation, the FIR's contain just the code's AST object.
Note: AST is immutable objects referenced by FIR nodes. The repeated AST are not quadratic space to store.

UBC expression evaluation step:

1. If the expression is a comment AST, store it as comment FIR.
2. If the AST is an arithmetic expression, evaluate math and produce a value FIR.
3. If AST represents brane concatenation , then process the branes based on precedence rules:
   detachment brane associating left using Left-First-Right-Last(LFRL), then brane concatenation
   associatively. The result is a FIR of a single brane.
4. If AST is a search expression, search is performed. The search may stay "unresolved" indefinitely.
4. If AST is a brane expression, then evaluate the brane.

UBC search evaluation:

1. TBD:

UBC brane evaluation:

1. The context is established: AB's and IB storage spaces are allocated.
2. Each line of FIR is evaluated using expression evaluation.
   1. If the FIR is a value, it is maintained as a value.
   2. If the FIR is an expression, it is evaluated, the new FIR retained. (*)
   3. IB is updated.
   4. Proceed to next line.
3. When the brane ends, the brane is returned as a value.

(*) The expression is *NYE* if it makes *NYE* references. When the *NYE* state is caused by `↑`
recursion, this single step materializes one step of the recursion. The FIR is updated to replace
`↑` with a FIR representing `[IB, AB] a`—the `a` being the AST of `↑`. This concludes the
iteration. Note, the next time we take a step in the UBC pass, this FIR is a brane concatenation
that can be processed by the brane evaluation. It may or may not produce another recursion step
depending on the program and context.

The UBC FIR processing has the following stages (NYES, pronounced like "NICE", encapsulates all
evaluation stages from UNINITIALIZED through CONSTANT):

1. **AST** - uncoordinated, unevaluated
2. **NYE** (Not Yet Evaluated) - FIR with some NYE ordinates (In progress, another UBC step may produce changes)
3. **CONSTANTIC** (pronounced "cons-TAN-tic", constant in context) - FIR with all ordinates coordinated as much as possible in current context; may gain value when associated with new context
4. **CONSTANT** - FIR with all evaluation results fully coordinated (Completion, another UBC does nothing). During coordination, a brane may stay CONSTANT or transition from CONSTANTIC to ALLOCATED if it started constantic.

**State Rendering in Output:**

| State | Rendering | Notes |
|-------|-----------|-------|
| NYE | `???` | Not yet evaluated |
| CONSTANTIC | `⎵⎵` | Constant in context; for constantic branes, contents may be shown |
| CONSTANT | value | The final computed value |

Because of the step-wise evaluation, the brane tree is evaluated breadth-first. The brane having
finite size means each UBC step terminates in finite time. As long as the UBC FIR is not complete,
each step makes progress towards completion.

## The Multicellular Brane Computer

- [ ] TBD

## The Next Step in the Evolution Chain

- [ ] TBD

## Typing

Types have become very central to humanity's approach to organizing and understanding programs.
Foolish therefore will also propose to use types to understand Foolish. Foolish would like to use
the name *characterization* to refer to representations and functionalities that are traditionally
within the realm of type systems. Foolish values, branes, expressions, and names are
characterizables. Foolish characterizations are Foolish programs that establish whether a
characterizable has or does not have said characterization. These are the most inclusive and the
most exclusive characterizations available in Foolish.

```foolish
	{
		all=[*]{
			true;
		};
		none=[*]{
			false;
		}
	}
```

Note that this this is nominal in that the characterizers have names.

## Relational Coordinates

Relational coordinates are the *ordinates* (names) of brane members that we believe, as programmers,
are the most important aspects of a brane. When we think of coordinates, we think of x and y
coordinates in a two-dimensional Cartesian graph. But in reality, almost all relationships that we
consider can be modeled as functions of coordinates in some coordinate system. Most often we think
of binary attributes, "hot or not", for example; sometimes, an object is observable to have
higher-dimensional coordinates such as SIC or ZIP codes. There could also be higher-valence
relationships, such as `{a,b,c} sorted_by_x` or `{a,b,c} is_median_height`, which may return a
boolean on whether `a.x`, `b.x`, `c.x` is sorted, or `a` is the median of `b` and `c`, respectively.

So, in Foolish, a value that is accessible *after* a brane has reached CONSTANTIC is considered a
coordinate—the value tells us how to relate to the brane that was just computed. A well-coordinated
brane is one that is constantic or has achieved CONSTANT with values so that computation can proceed
to access the coordinate with no undesirable consequences.

### Single Coordinate Matching

```foolish
{
	point_a = {x=1; y=2; z=3;};
	point_b = {x=1; y=3; z=4;};
	is_aligned = point_a.x == point_b.x; !! is_aligned = true;
}
```

### Relational Coordinate Matching

- [ ] TODO: {a,b,c,d} product_filter [ae,be,ce,de]{ae.x==be.x; be.y==ce.z; ce.a==de.b}AND
