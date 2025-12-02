# What does it mean to be equal?

The equal operators are equivalence relations.

## Same source code, aka syntactic equivalence.
```foolish
{
	a= {1, b=2+2};
	b= {1, b=2+2};
	c= {1, b=3+1};

    !! They have bit-wise identical source code. 
	confirm = a =s= b and a !=s= c and b !=s= c;
}
```

## Same after the compared branes are not nye.
```foolish
{
	a= {a=2,   2, b=1+2};
	b= {a=2,   2, b=2+1};
	c= {b=2, c=2, a=3};

	confirm = a == b and a !== c and b !== c;
}
```

## The most useful equivalence is semantic equivalence
semantic equivalence is evaluated on detached branes, wlog, `a` and `b`. If in all possible coordinations
these two branes are `a == b`, then the two are semantically equivalent `a === b`.
```foolish
{
	...!!All possible contexts
	{
		... !!All possible contexts
        A =$ {↑:a};
        B =$ {↑←:b}
		confirm A == B
	}
}
```
## Same names
```foolish
{
	a= {a=2, b=3};
	b= {a=3, b=2};
	c= {b=3, a=2};

    !! They have same names
	confirm = a =n= b and a=n=c and b=n=c;
    !! In the same order
	confirm = a =N= b and a !=N= c and b !=N= c;
}


```
## Same characterized names
```foolish
{
	a= {a=2, t'b=3};
	b= {a=3, t'b=2};
	c= {t'b=3, a=2};

    !! They have same names
	confirm = a =c= b and a=c=c and b =c= c;
    !! In the same order
	confirm = a =C= b and not a =C= c and not b =C= c;
}
```


## Same values appear
```foolish
{
	a= {a=2, b=3}
	b= {a=2, 3}
	c= {b=3, a=2}

    !! They have same names
	confirm = a =v= b and a =v= c and b =v= c;
    !! same values assigned to the same to the same names
	confirm = not a =V= b and a=V=c and not b=V=c; !! ordering ignored not ==
}
```
