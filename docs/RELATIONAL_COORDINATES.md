# Relational Coordinates

Relational coordinates are the *ordinates* (names) of brane members that we believe, as programmers,
are the most important aspects of a brane. When we think of coordinates, we think of x and y
coordinates in a two-dimensional Cartesian graph. But in reality, almost all relationships that we
consider can be modeled as functions of coordinates in some coordinate system.

Most often we think of binary attributes, "hot or not", for example; sometimes, an object is
observable to have higher-cardinality coordinates such as SIC or ZIP codes. There could also be
higher-valence relationships, such as `{a,b,c} sorted_by_x` or `{a,b,c} is_median_height`, which
may return a boolean on whether `a.x`, `b.x`, `c.x` is sorted, or `a` is the median of `b` and
`c`, respectively.

So when **identifying** a value inside a brane (using the `=` operator), we specify that the
expression should be referred to by that name in subsequent branes. When the assignment is
**evaluated** in a brane, the expression becomes **ordinated** to the brane, and we add an
**ordinate** (a coordinate/axis) for the brane. In some situations, the expression being ordinated
will evaluate to different values depending on where in the brane it is ordinated or which ordinate
it is assigned to. In these situations, the expression may be itself NYE (Not Yet Evaluated, any
pre-constanic state) until ordinated into a brane, at which point it becomes coordinated and reaches
CONSTANIC (constant in context, pronounced "cons-TAN-tic").

A well-coordinated brane should have its ordinates readily accessible by their names or brane search.

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
