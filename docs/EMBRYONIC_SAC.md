# The Embryonic Sac

The embryonic sac is a brane within the brane that is visible only to the brane itself. The embryonc sac
and its contents are invisible to searches.

```foolish
	{
		b = {E'ec={asdf=1;};
		     public_brane={t=$ec};
		     public_asdf=asdf;
		    };

		!! Outter brane cannot see into sub-brane's embryonic sac.
		confirm b.ec === ???;

		!! The public_brane of b is a sub-brane of b, so it too cannot see into the embryonic sac.
		confirm b.public_brane.t === ???;


		!! Finally, if the brane `b` itself makes a value available, then it is visible by the public name.
		confirm b.public_asdf !== ???;
	}

```


The embryonic sac should only appear as the very first brane. If empryonic sacs do not appear as the first
member of a brane, it may become dependent on a name or value that must be calculated as part of the
valuation of the brane. This would be quite alarming as embryonic branes are the branes to be coordinated
only in the embryonic stages of the brane, other ordinates has not become known or even nye.

```foolish
	{
		E'ec={
		      	something_private="rsa_key";
              	private_use = {
              	               	E'passed={!!!Do all the work in the embryonic sac};}
              	               	result=$passed;
              	              }

     confirm = something_private !== ???;

     public_value =$ something_private private_use;
	}
```

In programming languages, this is typically a `final` declaration. However, in foolish, ONLY branes may be
embryonic ordinates and they must not depend on non-embryonic values of the brane. The best way to
guarantee this is to only write the embrynoic sac's at the beginning of the brane.

