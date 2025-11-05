# Relational Coordinates

Relational coordinates are the *names* of brane members that we believe, as programmers, are the
most important aspects of a brane. When we think of coordinates, we think of x and y coordinates in
a two-dimensional Cartesian graph. But in reality, almost all relationships that we consider can be
modeled as functions of coordinates in some coordinate system.

Most often we think of binary attributes, "hot or not", for example; sometimes, an object is
observable to have higher-dimensional coordinates such as SIC or ZIP codes. There could also be
higher-valence relationships, such as `{a,b,c} sorted_by_x` or `{a,b,c} is_median_height`, which
may return a boolean on whether `a.x`, `b.x`, `c.x` is sorted, or `a` is the median of `b` and
`c`, respectively.

So, in Foolish, a value that is accessible *after* a brane is *FK* is considered a coordinate—the
value tells us how to relate to the brane that was just computed. A well-coordinated brane is one
that is *FK* to values so that computation can proceed to access the coordinate with no undesirable
consequences.

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
