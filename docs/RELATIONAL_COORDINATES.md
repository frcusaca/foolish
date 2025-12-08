# Relational Coordinates

Relational coordinates are the *names* of brane members that we believe, as programmers, are the
most important aspects of a brane. Conventionally, when we think of coordinates, we think of x and y
coordinates in a two-dimensional Cartesian graph. But in reality, almost all relationships that we
consider can be modeled as functions of coordinates in some coordinate system.

Most often we think of binary attributes, "hot or not", for example; sometimes, an object is
observable to have higher-cardinality coordinates such as SIC or ZIP codes. There could also be
higher-valence relationships, such as `{a,b,c} sorted_by_x` or `{a,b,c} is_median_height`, which
may return a boolean on whether `a.x`, `b.x`, `c.x` is sorted, or `a` is the median of `b` and
`c`, respectively.

So while naming a value inside a brane, we also added a coordinate dimension for the brane. In some
situations, the exprssion being coordinated will evaluate to different values depending on where in the
brane it is named or which coordinate it is assigned to. In these situations, the expression may be itself
nye until coordinated into a brane, at which point it may becomes fk.

From a slightly different perspective, if the names of values in a brane are the *abscissa* of a two
dimensional Cartesian plane, it occupies the X-axis on the horizontal in an XY plot. Then the *ordinate*
is the value we should access by that name, AKA the value on the Y-axis. So naming a value in a brane
has the effect of assigning an *ordinate* value to a *abscissa* rendering the brane a relation between the
two.

Thence thrusting upon a cup the ordinate of negative elevation over an ocean, the cup is destined to fill
with the content of the ocean. The same idea applies to coordinating a value to a name of a brane. An
otherwise abstract idea of a cup(whether or not with content), now is necessarily filled with contents of
the brane ocean surrounding it. It is in many ways identical to function invocation or injection of
dependencies. The coordination of an expression into a brane injects the dependencies it requires as water
would flow into a submerged cup. If said submerging cup A has another smaller cup B that is an ordinate
relative to a name(*abscissa*) in A, then that cut B must also fill with ocean water because of A. The
cavity within A and B that admits water can of course be selective either by name or by character. This
replicates the cell membranes by selecting what is admitted.


A well coordinate brane should have readily accessible by their names or brane search.

## Single Coordinate Matching

```foolish
{
	point_a = {x=1; y=2; z=3;};
	point_b = {x=1; y=3; z=4;};
	is_aligned = point_a.x == point_b.x; !! is_aligned = true;
}
```

## Relational Coordinate Matching

- [ ] TODO: {a,b,c,d} product_filter [ae,be,ce,de]{ae.x==be.x; be.y==ce.z; ce.a==de.b}AND
