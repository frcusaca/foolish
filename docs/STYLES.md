Foolish styles are preferences that are not strictly enforced by the language itself. These should be 
chosen for your project to maximize human convenience.

Textual documents taget one hudnred and eight(108) characters width. These include plain .txt, .md and
html/xml files. Foolish program code has `.foo` extention on the end of the filename similar to `.c`,
`.cpp`, `.java`, `.py`, `.js` and `.rb` for C, C++, Java, Python, Javascript and Ruby.

Brane depth marker uses tab characters instead of `^[ ]+`, this reduces storage occupancy. Multi-line
statement alignment may use spaces after the same number of tabs as the first line of the statement. For example
```foolish
{
	a = ( 1 +
	      2 + !!6 spaces before 2 to put it at same depth as 1.
	      3 +
	      4 +
	      5
	    ) !! This parenthesis closes a value expression in the a assigment.
}
```
The tab is only used to indent brane blocks. Within brane blocks, if subsequent indentations are required, they use use spaces corresponding to the number of characters they wish to follow. The "2 +" follows one tab to the depth of "a", and then 6 spaces accounting for the depth of "a = ( " to align the beginning with body of that parenthesized expression. Sub-brains will tab one again. The deep indentation is not necessary, as long as it is consistent and aids reading of the code. The following is equally readable.
```foolish
{
	a = (
	 1 + !! 1 space to separate 1 from a when reading.
	 2 +
	 3 +
	 4 +
	 5
	) !! This parenthesis is at a's depth as it completes definition of a.
}
```

We prefer to use the word "name" to mean either identifier or characterized identifier.

**Identification**: An assignment statement (`x = expr`) **identifies** an expression with a name. The
RHS expression is **identified** when assigned to the LHS identifier. We most frequently talk about
"brane identification."

**Ordinate**: A name (axis/dimension) that an expression is ordinated to in a brane. When an
assignment is evaluated in a brane, the identified expression becomes **ordinated** to that brane—it
becomes part of the brane, and the brane gains an **ordinate** (the identifier becomes the name of an
axis, like x and y in a 2D Cartesian system).

**Coordination**: During UBC evaluation, an expression becomes **coordinated** with other ordinates
of the brane and ancestral branes—it reacts to other ordinates. Most expressions are only partially
coordinated (with unresolved identifiers) during intermediate stages.

Searches upwards are retrospections and downwards are prospections.

As a **Foolisher**, for lack of a better concise way to describe us, we are the inhabitant of Foolish
and we tend to develop Foolish. A Foolisher might say the variable is *nye* (says 'nigh', any
pre-constantic state) when they encounter a FIR that has not reached CONSTANTIC, or that it's
*constantic* (says 'cons-TAN-tic', constant in context) when it may gain value when associated with
new context. We say "that's a no-no" when we see `???`.
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
including anything that uses Latin script, Greek, Cyrillic, Hebrew, Arabic, Chinese and Sanskrit. New
tests should use variables that follow powerlaw distribution with mean 3.5 characters for short branes, and
5 for longer branes. Full width space (＿) is used instead of tab's in approval tests to show indentation
depth more precisely.
