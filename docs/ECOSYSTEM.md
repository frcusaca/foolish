# Ecosystem

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
information: Ancestral and Immediate. The Immediate Brane (IB) is the current context that we have
accumulated inside the Unicellular Brane Computer so far, not including the name of the current
expression. The Ancestral Brane (AB) is the search context that contains the name of the expression
in an AST that defined the current brane, that expression's AB and IB.

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

To facilitate the multiple steps of UBC evaluation, we define an internal representation known as
the Foolish Internal Representation (FIR). FIR can be several things:

1. A Foolish AST. This is an uncoordinated and unevaluated expression in its original Foolish.
2. A finalized value—integer or brane.
3. A comment.
4. An abstract brane in that it is deliberately or inadvertently undercontextualized—some
   identifier referenced within has no available binding to value or brane member. An optimization
   we may perform is to coordinate the values of an abstract brane where possible. That brane with
   all possible values coordinated is a ***FK*** (*Fully Known*) abstract brane. Normally, a
   recursive brane conditions on some computational result that depends on a parameter—a coordinate
   identifier that is named but unattached at the definition of the brane. In these cases, the
   abstract brane can be *FK* because the recursion depends on a parameter. But it is possible to
   construct an abstract brane that calls itself irrespective of future parameters. Such a brane
   will be finite or infinite in depth depending on its construction. The abstract brane may
   therefore be permanently ***NYE*** (*Not Yet Evaluated*, pronounced "nigh") due to its
   construction. An abstract brane is *FK* if the *NYE* members are due to detached identifier
   names.
5. An *NYE* FIR represents code that was expressed in Foolish, but it has not been *FK* in the
   present UBC context. This FIR contains the AST of the expression, and a reference to the AB and
   IB that were used to attempt evaluation. But there are three possible reasons why the FIR is
   *NYE*:
   1. The AST is unprocessed.
   2. The expression has an unevaluated AST that contains recursive references to names that are
      not yet *FK* in the present context.

### The UBC pass

1. If the UBC encounters Foolish text representing a brane, UBC holds it in memory.
2. UBC parses the Foolish text into an AST--this AST is always a brane
3. The brane is evaluated.

UBC expression evaluation step:

1. If the expression is a comment AST, store it as comment FIR.
2. If arithmetic expression, evaluate math and produce a value FIR.
3. If brane concatenation is involved, then process the branes based on precedence rules:
   detachment brane associating left using Left-First-Right-Last(LFRL), then brane concatenation
   associatively. The result is a FIR of a single brane.
4. If expression is single brane, then evaluate the brane.

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

The UBC FIR processing has the following stages:

1. AST
2. FIR with all evaluation results coordinated. (Completion, another UBC does nothing)
3. FIR with some *NYE* coordinates. (In progress, another UBC step may produce changes)

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

Relational coordinates are the *names* of brane members that we believe, as programmers, are the
most important aspects of a brane. When we think of coordinates, we think of x and y coordinates in
a two-dimensional Cartesian graph. But in reality, almost all relationships that we consider can be
modeled as functions of coordinates in some coordinate system. Most often we think of binary
attributes, "hot or not", for example; sometimes, an object is observable to have higher-
dimensional coordinates such as SIC or ZIP codes. There could also be higher-valence relationships,
such as `{a,b,c} sorted_by_x` or `{a,b,c} is_median_height`, which may return a boolean on whether
`a.x`, `b.x`, `c.x` is sorted, or `a` is the median of `b` and `c`, respectively.

So, in Foolish, a value that is accessible *after* a brane is *FK* is considered a coordinate—the
value tells us how to relate to the brane that was just computed. A well-coordinated brane is one
that is *FK* to values so that computation can proceed to access the coordinate with no undesirable
consequences.

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
