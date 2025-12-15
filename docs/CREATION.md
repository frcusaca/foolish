#Creation
We postulate that one can always create a new value that is distinct, in our Foolish branes, from everything else we've ever seen before. Furthermore, not only can we think of new values, we can name it and talk about it. We represent this act using a big black dot ⬤  (u+2B24, &#x2B24;). In reality, if a brane is as big as the universe, one can essentially always create ideas that are unique and new. Programatically, we construct all concepts by creating new items and relating them.

```foolish
	{!!system.foo
		??? =   ⬤  ; !! The unknown is the first idea we had.
		c   =   ⬤  ; !! Characterization is a new idea.
		c   = c'c  ; !! Let's make sure we characterize it correctly.
		b   = c'⬤  ; !! Brane is a new idea. It is characterized as a characterization.
		i   = c'⬤  ; !! Integer is a new idea for characerization.
		f   = c'⬤  ; !! Floating point numbers is a new idea for characerization.
		s   = c'⬤  ; !! String is a new idea for characerization. Although this may become a brane later. (i.e. `s = c'b`, or similar)
	}

	!! NB: Concatenation would naturally occur here.

	{!!system.i.foo
		.  .  .
		-3 = i'⬤ ; !! Integer characterization
		-2 = i'⬤ ;
		-1 = i'⬤ ;
		0  = i'⬤ ;
		1  = i'⬤ ;
		2  = i'⬤ ;
		3  = i'⬤ ;
		4  = i'⬤ ;
		.  .  .
        confirm = -1 < 0 < 1;
		.  .  .
        confirm = i'0 == f'0;
		.  .  .
        confirm = 0 == -1 + 1   ;
        confirm = 0 == ?0:↑ + 1 ;
        confirm = 0 - 1 == ?0:↑ ;
	}


	{!!system.f.foo
		.  .  .
		0 = f'⬤ ; !! Floating point characterization
.
.
.
```

