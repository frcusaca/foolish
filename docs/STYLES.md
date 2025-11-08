Foolish styles are preferences that are not strictly enforced by the language itself. These should be 
chosen for your project to maximize human convenience.

Textual documents taget one hudnred and eight(108) characters width. These include plain .txt, .md and
html/xml files.

Brane depth marker uses tab characters instead of `^[ ]+`, this reduces storage occupancy. Multi-line
statement alignment may use spaces after the same number of tabs as the first line of the statement.

We prefer to use the word "name" to mean either identifier or characterized identifier. Ordinate is
a name that has been associated with a brane, it is also a verb meaning to make a name an ordnate of a
particular brane. The coordinate mainly refers to when a brane becomes ordinated to a brane (assigned
to a name), but because it may refer to other ordinates in the brane and its parents, that act is
called "coordinating a child brane to a parent brane." Searches upwards are retrospections and downwards
are prospections.

As a **Foolisher**, for lack of a better concise way to describe us, we are the inhabitant of Foolish
and we tend to develop Foolish. A Foolisher might say the variable is *nye* (says 'nigh') when they
encounter a FIR that has not yet been evaluated fully, we say "that's a no-no" when we see `???`.
not fully evaluated. During development, a feature becomes "lexed", which means it parses into AST, but
downstream implementation yields a `???` when it is encountered. A second step during which the feature
becomes "interpreted" which is when we gain the machinery to correctly handle the the feature in the VM.
So good places to be are "lexed and tested" and "interpreted and tested".


Unit test is able to very specifically test each unit of the software, it is the primary check of
correctness. Approval tests should try to correspond to unit tests, but approval test tend to illustrate
behavior to users better. It is best used to illustrate the most IMPORTANT and most EASILY CONFUSED
aspect of code behavior. Approval tests should be comprehensive to show compatibility between different
implementations of the VM as well as establish full mutual understanding between human users and the fvm--
 in a more human readable way.  When writing tests, please be mindful that we'd like to expand the 
utilization of unicode and use more of the available alphabets to improve our expressivity and to help
disambiguate concepts written in Foolish. Use sensible names from all available languages, currently
including anything that uses Latin script, Greek, Cyrillic, Hebrew, Chinese and Sanskrit. New tests should
use variables that follow powerlaw distribution with mean 3.5 characters for short branes, and 5 for
longer branes.
