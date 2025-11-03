# Foolish Programming Language

*Where proximity meets computation, and containment creates clarity*

[![Java CI](https://github.com/frcusaca/foolish/actions/workflows/tests.yml/badge.svg)](https://github.com/frcusaca/foolish/actions/workflows/tests.yml)
[![License](https://img.shields.io/badge/license-Open%20Source-blue.svg)](#)
[![Status](https://img.shields.io/badge/status-Active%20Development-green.svg)](#)

## Welcome to the Future of Programming

Foolish is a revolutionary programming language that reimagines how humans 
interface with computers. Built from the ground up for the 21st century, 
Foolish combines functional programming elegance with intuitive natural 
metaphors to create a uniquely expressive and powerful development experience.

### Why Foolish?

- **Nature- and Life-Inspired Design**: Programming concepts mirror natural
  processes like cellular organization and proximity-based interactions found
  throughout living systems
- **High Semantic Throughput**: Express complex ideas with minimal cognitive
  overhead and remarkably low temporal latency
- **Advanced Search Capabilities**: Revolutionary brane search system for
  navigating and querying code structures with unprecedented flexibility
- **Compositional Architecture**: Functions, objects, and data structures unify
  seamlessly through elegant concatenation patterns
- **Concise and Sweet Syntax**: Extensive syntactical sugaring focuses your
  attention on logic rather than ceremony

### Our Vision and Goals

Foolish aspires to become the natural way anyone thinks about interacting with
computers. We are building toward a future where programming is:

- **Safe and Precise**: Side-effect-free abstract functional programming
  foundation ensures reliability and predictability
- **Grounded in Reality**: Built-in methods seamlessly connect abstract thought
  to real-world implementations
- **Completely Transparent**: Full code transparency enables unprecedented
  understanding and debugging capabilities
- **Automatically Enhanced**: Built-in hooks support automatic programming,
  formal proofs, verification, and computing with uncertainty
- **Maximally Human**: Accessible, ergonomic, and adaptable design that is
  precise and high-functioning for human use
- **Bidirectionally Communicative**: The system maximizes both human-to-computer
  and computer-to-human communication efficiency

---

## Quick Start

```bash
mvn clean generate-sources compile test
```

Foolish programs use the `.foo` extension and embrace a philosophy where 
**proximity creates combination** and **containment enables organization**. The 
language provides rigorous abstraction capabilities while maintaining interfaces
that ground your computations to the physical and biological realities you want
to model.

## Key Features That Set Foolish Apart

### 🧬 Bio-Inspired Programming Model
Unlike traditional languages that force you to think in terms of machines, 
Foolish lets you think in terms of natural systems. Branes mirror cellular 
organization, proximity drives interaction, and containment enables natural 
hierarchies.

### 🔍 Revolutionary Search as First-Class Citizen
Stop scrolling through endless files. Foolish's integrated search system lets 
you query your code from within the language itself. Find code by variable name, 
by value, or by association.

### 🎯 Functional Purity with Real-World Grounding
Enjoy the safety and predictability of pure functional programming while 
maintaining seamless interfaces to the messy, stateful real world. The best of 
both paradigms, unified.

### 🔗 Natural Composition Through Proximity
Functions, data, and objects combine simply by being placed near each other. 
No complex import systems, no verbose inheritance hierarchies—just natural, 
intuitive composition that mirrors how ideas naturally combine in human thought.

### 🍯 Sweetened Syntax, Powerful Semantics
Extensive syntactical sugar means you write what you mean, not what the 
compiler wants. Focus on your logic while Foolish handles the ceremony.

### 🤖 Built for the AI Age
Native support for uncertainty, automatic program generation, formal 
verification, and human-AI collaborative programming. Foolish doesn't just run 
on computers—it bridges human and computational intelligence.

### 👼 Built for Good
As we stand at the precipice of evolution, we must imbue our creations with all
the best that we have come to know. Foolish aims to be built for good, not
evil.

---

## Table of Contents

- [Core Concepts](#core-concepts)
  - [Branes: The Foundation](#branes-the-foundation)
  - [Sizes](#sizes)
  - [Expressions and Values](#expressions-and-values)
  - [Names and Scope](#names-and-scope)
    - [Scope of Name](#scope-of-name)
  - [The Unknown](#the-unknown)
  - [Renaming](#renaming)
- [Advanced Features](#advanced-features)
  - [Brane Operations](#brane-operations)
    - [Proximity is Combination](#proximity-is-combination)
    - [The Brane Concatenation](#the-brane-concatenation)
  - [Search System](#search-system)
    - [Name Search](#name-search)
    - [Bulk Name Search](#bulk-name-search)
    - [Search Paths](#search-paths)
    - [Value Search](#value-search)
    - [Search Cursor Movements](#search-cursor-movements)
    - [Unanchored Brane Search Leads to Self-Awareness](#unanchored-brane-search-leads-to-self-awareness)
    - [Naming a Search Result](#naming-a-search-result)
  - [Detachment and Parameters](#detachment-and-parameters)
    - [Named Detachment](#named-detachment)
    - [Complete Detachment](#complete-detachment)
    - [Detach to Default](#detach-to-default)
  - [Control Flow](#control-flow)
  - [Recursion](#recursion)
    - [Corecursion](#corecursion)
    - [Mutual Recursion](#mutual-recursion)
- [Ecosystem](#ecosystem)
  - [Computer Reading Branes](#computer-reading-branes)
  - [The Unicellular Brane Computer (UBC)](#the-unicellular-brane-computer-ubc)
  - [The Multicellular Brane Computer](#the-multicellular-brane-computer)
  - [The Next Step in the Evolution Chain](#the-next-step-in-the-evolution-chain)
  - [Typing](#typing)
  - [Relational Coordinates](#relational-coordinates)
    - [Relational Coordinate Matching](#relational-coordinate-matching)
- [Language Reference](#language-reference)
  - [Comments](#comments)
    - [Line Comments](#line-comments)
    - [Block Comments](#block-comments)
  - [TODO Items](#todo-items)
- [Development Notes](#development-notes)
- [Appendix](#appendix)
  - [Documentation Contributors](#documentation-contributors)

---

## Core Concepts
### Branes: The Foundation
The Foolish language is in its most primitive form a containment and 
organization of values. We contain values in something called a *brane*. The 
brane brings to mind concepts such as cell or nucleus mem*brane*. The Foolish 
brane resembles traditional mathematical and programmatic concepts such as sets,
lists, maps or associative arrays, structs, enums or records. Foolish uses 
curly braces to enclose values `{}`:
```foolish
{}                    !! Empty brane
{{}}                  !! Brane containing an empty brane  
{{};{{}};{{{}}};}     !! Complex nested structure with multiple branes
{🌌={};}              !! Supports customizable alphabet
{愚=↑;}
{👶=👍;}
```

This ability to create containment will ultimately help us organize ideas. 
Branes can be nested arbitrarily deep, allowing for sophisticated hierarchical 
data structures that mirror how we naturally think about complex systems. Just 
as biological cells contain organelles, which contain molecules, which contain 
atoms, Foolish branes can contain other branes in a natural, intuitive manner.

### Sizes
In the physical world, containment membranes and units of organization tend to
have observably limited size. Cells can only be so big before they split or
die. Atomic and subatomic objects have to be so close, or otherwise the forces 
that keep them together stop working. Therefore, the Foolish brane is also 
limited in size. The Foolish brane certainly should have finite size, and 
depending on the computer, it may have a specified limit on the number of 
entries.

Inside the brane, it is a one-dimensional object. Its entries line up one after 
another. The true dimension of entries are computational dependencies. So 
depending on the code inside, the actual dependency axis could have smaller 
dimensions best visualized as a dependency DAG. The dependency DAG and its 
subdimensions branch out and progress forward in their own time dimensions. 
Sometimes more than one timeline merges together by being involved in an 
expression that refers to all of them. Overall, the brane's fracturing timelines 
can be linearized, by stringing them into the sequence of code as they originally
appear inside the Foolish code, into a single time dimension.

## Language Reference

### Comments
Comments in Foolish are expressions that contain unparsed Foolish. They are 
generally escaped using multiple exclamation marks. Comments are part of the 
program that have no expressive effect on the evaluation of the program.

#### Line Comments
Line comments are like those from standard languages. The marking that begins 
the line comment is a double exclamation mark `!!`.

```foolish
{
   !! This is a comment inside the brane
   !! This is another comment inside the brane.
   !! We can even exclaim inside a comment !!!
   !! - [ ] TODO: Check that this is possible ----^^^
}
```


#### Block Comments
Block comments are enclosed by a pair of consecutive triple exclamation marks:
```foolish
{
    !!! Move along nothing to see here.

         ##
        ### 
         ##
                    #
        ###################
        #####################
                    #    ## #
                        #####
                           
            #### 
          #########
         ##       ##
        #           #
        #          ##
         ##       ##
          #########
            #### 
            #### 
          #########
         ##       ##
        #           #
        #          ##
         ##       ##
          #########
            #### 
    !!!
}   
```


### Expressions and Values
Aside from comments, branes may contain expressions. Expressions are symbols 
that follow Foolish rules and should be evaluable to a fixed value. Here are a 
few integer expressions inside a brane:

```foolish
{
   1;    !! This is the number 1
   2;    !! This is the number 2  
   3.14; !! Floating point numbers work too
   "hello"; !! Strings are also expressions
}
```
A brane written in Foolish is itself an expression.

### Names and Scope
One very powerful concept we have for abstracting thoughts and thinking of 
complex matter with complex properties and interactions is the substitution of 
the statement or object of consideration with a name. Names such as `x`, `y`, 
`foolish`, `programming language`. So this is an important concept in Foolish 
that we are able to name value expressions using the naming operator `=`:

```foolish
{
   a = 1;                                    !! Simple name binding
   b = 2;                                    !! Another binding
   c = 3;                                    !! Value assignment
   greeting = "Hello, Foolish world!";      !! String binding
   calculation = a + b + c;                  !! Expression binding
   !! etc.
}
```

Names are selected from an alphabet that excludes the reserved symbols of the 
language. Names should include a small non-breaking white space character, 
currently `_`. Ideally it should be a non-breaking thin space \&thinsp 
"&thinsp;". But nothing is able to render these spaces right now, so we also 
include `_` for spacing out complicated nouns.
```foolish
{
    
    a_name=0;
    a&#x2009;name=1;   !! Thin space in hexmal
    a&#8201;name=1;    !! Thin space in decimal
    a&thinsp;name=2;   !! Thin space word entity
    a&hairsp;name=3;   !! hair space
    a&#x202F;name=4;   !! narrow non-breaking space
}
```

Foolish expressions and values inherently do not have names. They are only 
marked with names when the naming operator `=` is used. So the following brane:
```foolish
{
    1;
    2;
    3;
}
```
is a brane with three unnamed values. The following brane:
```foolish
{
    a=1;
    b=2;
    c=3;
}
```
is a brane with three named values.

#### Scope of Name
Names are scoped to "before the current expression". So the current naming 
operation is not yet in scope. This is similar to how many programming 
languages work. So the following brane:
```foolish
{
    a=1;
    a=a+1; !! a=2
    a=a+1; !! a=3
}
```
Such a convention has the direct benefit of delaying recursive concepts until 
more thought is put into the line of code.

### The Unknown
The unknown is of paramount importance to us, therefore we dedicate a symbol 
to express Unknown in Foolish `???`. Here we declare we do not know the answer:
```foolish
{
    answer=???;
}
```
In fact every unnamed expression is an assignment to an unknown name:
```foolish
{1;2;3;}
```
is shorthand to
```foolish
{
    ???=1;
    ???=2;
    ???=3;
}
```

### Renaming
Foolish permits reusing names. If one were to think of the semantics of static 
single assignment, this would be how Foolish interprets a reused name.
```foolish
{
    a=1;
    b=a;
    a=2;
    c=a;
}
```
is equivalent to
```foolish
{
      a=1;
      b=1;
      a=2;
      c=2;
}
```
NB: we did preserve the first assignment `a=1` as part of the brane for 
historical accuracy.

## Advanced Features

### Brane Operations
#### Proximity is Combination
Branes combine freely. Listing two branes either in code or by named reference 
means it is desired that the two branes be concatenated. The following code 
snippets have the same meaning in Foolish:
```foolish
{
    a=1;
    b=2;
}
```

and
```foolish
{
    a=1;
}
{
    b=2;
}
```
This kind of resembles some physical and biological systems where like-kinded
objects combine when they come to close proximity to each other. This principle 
extends throughout nature: atoms bond to form molecules, cells aggregate to 
form tissues, individuals cluster to form communities. In Foolish, this same 
principle governs how computational elements interact and combine, making the 
language feel natural and intuitive to human programmers who are themselves 
products of these same organizational principles.

#### The Brane Concatenation
Concatenation operation is associative. Here is an example of branes acting 
like functions; naturally, brane concatenation invokes functions in RPN.

```foolish
{
    !! ...
    f={
        y=x*x+2x-1;
    };
    p={x=1+a;}
    q={a=1;}
    result =  q p f !! We called `f` on parameter `p` which was evaluated on `q`.
    !! ...
}
```

One will note that it didn't matter how evaluation occurred—either `(q p) f` 
or `q (p f)` yields the same `result`:
```foolish
{
    !! ...
    result={
        a=1;
        x=1+a;
        y=x*x+2x-1;
    }
    !! ...
}
```

Another example of using concatenation is derivation, which mimics 
Object-Oriented programming inheritance. By default, if we extend a brane with 
another by post-concatenation, method and member references follow "static 
invocation" rules.

```foolish
{
    class_a = { a=1; b=2}
    class_b = class_a {c=3}
    a_b     = class_b
    !! So, class b is inheriting everything from a with the addition of `c`
}
```

The only way to create the effect of dynamic invocation is to create interfaces
and specify them at programming time:
```foolish
{
    animal         = {kindom='Animalia';}
    mouthed_animal = animal {greet='hello';};
    cat            = mouthed_animal {greet='mewo'};
    dog            = mouthed_animal {greet='bark'};
    pet            = {play={greet;greet;greet;};};
    my_pet         = cat pet;
    your_pet       = dog pet;
}
```

### Search System
Once we have established a bundle of named value expressions that we call a 
brane, we expect there to be some order and conceptual meaning to this 
collection—there is meaning in it and there is meaning in their collection 
together. It may become useful for us to inspect this brane for some particular
aspect that interests us. We accomplish this through Foolish's revolutionary 
search system, which treats code as a searchable, queryable structure rather 
than a linear sequence of instructions.

The search system recognizes that programmers spend much of their time 
navigating and querying their code structures. Instead of forcing you to 
remember exact variable names or scroll through long files, Foolish lets you 
search for things by pattern, by value, by computational relationship, or by 
position. This transforms programming from a process of precise recall to one 
of intelligent exploration and discovery.

Brane search expresses are always written postfix except for during naming.
#### Name Search
Conventional regular expressions can be used to search the namespace of a brane,
treating the brane as a text document of names. After each search, if the 
search continues, the cursor is placed at the beginning of the name that was 
found.
   * Backward search from a cursor is after the last character in the brane's 
     name document.
      * `.` means to find the last name matching the regexp: `brane.result` 
        means the last value to be named `result` in the brane. This 
        interpretation has a similar effect as the normal dereferencing symbol 
        `.` used in other languages.
      * `$` means to find just the last value in a brane. `{result=10;}$` would 
        extract the value 10.
      * `b#-5` means to find the fifth-to-last value irrespective of names. The 
        last entry in a brane is `#-1` per modern negative array indexing 
        convention.
   * Forward search from a cursor that is before the first character of the 
     brane's name document.
      * `/` means to find the first name matching the regular expression: 
        `f/parameter` gives us the parameter value an evaluation of `f` used.
      * `^` means the very first value in the brane.
      * `b#5` means the SIXTH value in the brane. We use zero-based array 
        indexing for non-negative indexes as per convention.
In all cases of single regular expression matches, match failure is a 
compilation error if possible and a runtime error if it occurs.

Accessing `b$x` is also referred to as accessing the `x` coordinate of `b`. See
the [Relational Coordinates](#relational-coordinates) section for expanded discussion.


#### Bulk Name Search
In the case where a regular expression is used for searching for multiple brane 
content, we use the operators `//` and `??` to ask for a brane that contains all 
matching entries of the search-subject brane.
   * `b//tmp_.*` the result is a brane that refers to the `b`
```foolish
{
    doc={
        tmp_a = 2*2;
        c     = tmp_a sqrt;
        tmp_b = 3*3;
        d     = tmp_b cbrt;
    }
    r = doc//tmp.*;
    r2 = doc??tmp.*;
}
```
evaluates to
```foolish
{
    doc={
        tmp_a = 2*2;
        c     = tmp_a sqrt;
        tmp_b = 3*3;
        d     = tmp_b cbrt;
    }
    r = {
        tmp_a = doc.tmp_a;
        tmp_b = doc.tmp_b;
    }
    r2 = {
        tmp_b = doc.tmp_b;
        tmp_a = doc.tmp_a;
    }
}
```


#### Search Paths 
##### Backward Search
Since the Foolish document is read top to bottom, it is not possible to search 
downward past the current position. Therefore, these symbols are not permitted 
in an unanchored search: `↓`, `#@+1`, `→`, `#+1`. However, if an existing 
search expression is in progress, it is possible to anchor these searches to a 
previous unanchored search result. For example, `??_*result#+1` is permitted.
Backward search is conducted in the following manner to discover their 
occurrences:
1. Start with the cursor at the beginning of the current line.
2. Search the current brane backwards from the cursor. Return if found.
3. If not found, search raises the cursor to the current brane's parent expression;
   the cursor is placed at the beginning of that line.
4. Go to step 2.
5. If no parent brane, then search returns `???`.

##### Forward Search
Forward search is performed line by line to the end of the brane. Forward 
search is not permitted to raise the cursor to parent branes. So forward search
is limited to the current brane only.

##### Depth Search
- [ ] TODO: Extend search into the computation graph with more syntactical 
capabilities for finding computational results.

### Value Search
Forward value search is triggered by the `:` operator. It is a forward search 
operator on a brane that searches for a value that can be considered equivalent
to the search parameter. `::` is the bulk forward value search operator.
```foolish
{
    doc={
        tmp_a = 2*2;
        c     = tmp_a sqrt;
        tmp_b = 1+3;
        d     = tmp_b cbrt;
    }
    r = doc:4;    !! r = 2*2;
    r2= doc::4;   !! r2 = {tmp_a=2*2; tmp_b=1+3;}
    doc:4 = 10    !! `tmp_a = 10;`
}
```
Single and bulk backward value search are triggered by `<:` and `<<:` 
respectively.
The part of the search result that is used depends on the context. When used on
the RHS of a naming expression, the value is used. When used on the LHS of a 
naming expression, the name is used.


##### Expression Search
- [ ] TODO: Extend search into the computation graph with more syntactical 
capabilities for finding computational results.
##### Search Combinations
- [ ] TODO: Document combinations of name and value search.
### Search Cursor Movements
Recall each search result puts the cursor at the start of the line just before 
variable names. We now define a few operators to move that search cursor.

| Expressions| Effect on Cursor|
|:----------:|:---------------:|
| `↑`,`#@-1` | Moves cursor to the beginning of the line referring to this brane |
| `↓`,`#@+1` | Moves cursor to the beginning of the brane that is the value of this line|
| `←`,`#-1`  | Moves cursor to the start of previous line|
| `→`,`#+1`  | Moves cursor to the end of this line, effectively at the start of the next line |

So we could use stack based syntax if we so desire:
```foolish
{
    f = {result=↑#-1 + ↑#-2}; !! See below detachment for clarification
    0;
    1;
    f$; !! 1
    f$; !! 2
    f$; !! 3
    f$; !! 5
    f$; !! 8
    f$; !! 13
}
```

For now we forbid the use of unanchored `#0` searching to the beginning of the current line.
The only permitted recursive call is the `↑` operator. An expression that contains 
recursion must be an expression with a brane that refers to the line in the parent brane
where it is coordinated.

- [ ] TODO: Document searching upward for variables scoped in parent branes.

#### Unanchored Brane Search Leads to Self-Awareness
Notice that every time we refer to a named value by name (a.k.a. reference a value
by a variable name), we are actually doing a `.` search on the brane itself and
its parents as if the brane ended before the start of the current line. So all
the searches can actually be invoked without following a brane. In that case, it
would mean to search the present brane with the cursor just before the beginning of
the current line.
```foolish
{
    1;
    2;
    c= #-1 + #-2;
}
```
- [ ] TODO: Augment syntax to make it easier to read for humans and possible to 
parse for compilers.


### Naming a Search Result
With the introduction of search, we now have the ability to name results of 
search. A shorthand is introduced to make it easier to extract single values 
out of branes. The `=` naming operator may be followed by a brane search 
expression and then by the brane. The meaning is that we will apply the search 
on the RHS brane before assigning that value to the name. The brane 
`{ V =E B }` where `V` is a name and `E` is a brane search expression, and `B` 
is a brane (NB: no space between `=E`). That expression means `{V = B E}`.

Recall the brane concatenated function invocation above; we have changed one 
line, which is that during the assignment of result, we have added a brane 
search expression (`$`) right after the naming operator (`=`). This puts the 
actual number into the result instead of the brane that computed the number.
```foolish
{
    !! ...
    f={
        y=x*x+2x-1;
    };
    p={x=1+a;}
    q={a=1;}
    result =$  q p f !! result is now the actual `result=7;`

    !! ... A few more examples
    calculation =  q p f !! result is now the actual `result=7;`
    secondary_result =.x calculation
    secondary_result =x calculation
    secondary_result = calculation.x
    result = calculation$
}
```
The flexibility of expression helps the Foolish user greatest flexibility in 
extracting use information from the right place, and at the same time place 
emphasis on code symbols to maximize readability and communication efficiency.

### Detachment and Parameters
Humans have long sought detachment from the material world. Even though Foolish
is inherently detached and completely abstract, we find that we need to perform
further detachment.
#### Named Detachment
```foolish
[
    a = ???;
    b = ???;
]
```

This brane detaches the names a and b. So to declare a function with formal 
parameter names that might already be in use, one could perform detachment 
during a naming expression.
```foolish
{  
    fn=[
        a=???;
        b=???;
    ]{
        result=a+b;
    }
}
```
The detachment is left associative with detachment branes and right associative
with branes--It overwrites detachment branes on the left, and affects brane on
it's right.

The original example of function definition probably needed this detachment for 
it to be a useful function:
```foolish
{
    f = [↑=???]{result=↑#-1 + ↑#-2};
}
```
Ideally, an upward search like this will block out ALL references to variables 
from the parent brane and above. So that in this function
```foolish
{
    f = [↑=???]{result=↑#-1 + ↑#-2 - universal_constant};
}
```
the `universal_constant` will be specified by caller as a parameter.

An example of its use:
```foolish
{
   x=1;
   y=2;
   z=3;
   f=[x;z]{ result = x - y; }
   result = [y,x] f
}
```
The detachment brane dissociates the ensuing brane on its right side from the
context, decontextualizing x and y. x and y become unbound symbols. From the timeline
perspective, the detachment snips off dependency DAG edges.

#### Complete Detachment
We now introduce complete detachment using name search to completely detach 
from a large number or all of the names and values.
```foolish
{
    clean_heap=[tmp_*=???]old_heap_brane
}
```
Or even complete detachment like this:
```foolish
{
    pure_func=[*=???]{...}
}
```

#### Detach to Default
In order to mimic default parameters in many modern programming languages, we 
shall offer a way to specify detachment that sets not only the expectation for 
what needs to be provided but also what happens when that expectation is not 
fulfilled.
```foolish
...
f=[r=???; pi=3.14]{
    pir = pi * r
    area = pir * r
    circumference = 2*pir
}
...
c       = {r=2} f
c_hires = {r=2, pi=3.14159} f
c_huh   =$ [r=4] f  !! c_huh = 25.12; 
```
Note, though, that the binding of RHS within the detachments are again within 
the scope of the current brane. The compiler shall give warnings in 
suspiciously ambiguous cases. The `c_huh` is a brane now having a default value
for r as well, but since the assignment asks for the last entry, the result is 
computed and assigned in the referring brane.

When default branes are next to each other, they are left associative—the 
left-most default brane is overridden by the default brane to its immediate 
right first and foremost. Default brane concatenation always has higher priority
than their concatenation with branes. When a default brane is between branes it
is always right-associative—it is applied to the brane on the right side before
other brane operations.
### Control Flow
Branching is accomplished using search.
```foolish
{
    ifblock = {
        if x==1 then         !! `if k then` fools to the expression `f'condition=k;`
        result = 1;
        else if x==2 then    !! `else if k then` fools to the expression `f'condition=k;`
        result = 4;
        else                 !! `else` fools to the expression `f'condition=true;`
        result = 2;
        f'condition=10;      !! ERROR, cannot assign to Foolish names
    }
    result = ifblock/f'condition:true→;   !! Search for a true value, then forward cursor to the next line.
    result = condition⇒;                !! shorthand for `/f'condition:true→` !! That uses a double-lined arrow in '=⇒' which is different from next statement single-arrow
    result => condition;                !! shorthand for `/f'condition:true→`
}
```
### Recursion
Since the brane search starts with cursor at beginning of the current line, 
finding the current brane is easy with `↑`. So recursion is easy to express:
```foolish
{
    factorial = [n=???]{
        result = if n <= 1 
                 then result = 1 
                 else [n=n-1]↑$;
    }
    five_fact = [n=5] factorial    !! five_fact = 120;
}
```
#### Corecursion
Here, it seems that we often perform loops or other complex operations where a 
single line performs updates to several states. The most prototypical example is 
this Python code:
```python
def fib():
    l = 0
    ll = 1
    yield l;
    yield ll;
    while true:
        l, ll = ll, l + ll
        yield ll
def triangle_number():
.
.
.
while not quit():
    print(next(fib()) * next(triangle_number()))
```
And we proceed to use `fib` while we operate on triangle numbers as well. This 
Fibonacci counter continues merrily on its own.

But so far Foolish has been entirely functional.
- [ ] TODO: How do we centrally organize several coordinated state updates in a 
single line of code?

#### Mutual Recursion
However, mutual recursion is not directly possible since we cannot search 
downward past the current line. The only way to create corecursion is to use an
unbound name and pass it into the context:
```foolish
{
    even = [n=???; odd=???]{
        result = if n == 0 
                 then result = true 
                 else [n=n-1;] odd$;
                 !! - [ ] TODO: Explain why we can't do this:
                 !! else [n=n-1;] →;
    };
    odd  = [n=???]{
        result = if n == 0 
                 then result = false 
                 else [n=n-1; odd=↑;] even$; !! Already defined above, so we can find it directly by name.
                 !! By Foolish convention, the symbol 'odd' is not yet in scope, so we need to pass it by 
                 !! searching backwards for odd function itself.
    };
    is_four_even = [n=4] even{odd=odd} !! is_four_even = true;
    is_five_even = [n=5] even{odd=odd} !! is_five_even = false;
}
```

## Ecosystem
### Computer Reading Branes
When a computer encounters a `.foo` file, it shall parse the Foolish into
an AST. The units on which the computer works are the expressions. The
largest expression being a brane, its processing takes up much of the Foolish
ecosystem.

### The Unicellular Brane Computer (UBC)
Like our human minds, most of our computational resources have limited abilities.
A Unicellular Brane Computer (UBC) is one that has just enough capacity to hold the AST
of a brane, and the ability to interpret and understand a single expression at a time.
This computer proceeds simply from the beginning of the brane to the end of the brane,
evaluating and creating new values to be stored in the brane. In order to process
expressions, the brane actually has two more sources of information: Ancestral and
Immediate. The Immediate Brane (IB) is the current context that we have accumulated
inside the Unicellular Brane Computer so far, not including the name of the current expression.
The Ancestral Brane (AB) is the search context that contains the name of the expression
in an AST that defined the current brane, that expression's AB and IB.

In some cases, such as the recursive call to `↑`, the search finds a line of code that
has an incomplete value. Note by incomplete value, in the UBC, we mean not the unknown 
value `???`, but a value that the present UBC does not fully know the value of (yet). In 
the case of this incomplete value, it would appear that the simplest machine is unable 
to continue evaluating the value of the Foolish expression. In fact, we don't even 
know all of the names or values this brane will capture by its end. To continue 
computation, the UBC must stash the current execution context, and evaluate that 
expression using the context "IB AB ↑". That says the UBC shall evaluate the AST of the 
expression that created the present brane, resolving any searches first using the 
current AB, then using the current IB.
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

In that example, the recursive call to compute r is processed by the UBC to the
equivalent of:
```foolish
   {
      ...
      r = [three=zero+one+2;] IB AB !!! the code for f= !!!
      ...
   }
```
Because branes are finite, the code on the right side of `f=` is surely parsed
to AST and stored in memory by the time the UBC is required to evaluate this line of
Foolish. Therefore in constant time, the uncoordinated `↑` is translated into an
AST expression. This AST expression may further contain `↑` inside or at its front.
But that is for a later iteration of the UBC to evaluate. Each step taken by the UBC
should require finite time.

To facilitate the multiple steps of UBC evaluation, we define an internal representation
known as the Foolish Internal Representation (FIR). FIR can be several things:
1. A Foolish AST. This is an uncoordinated and unevaluated expression in its original
   Foolish.
2. A finalized value—integer or brane.
3. A comment.
4. An abstract brane in that it is deliberately or inadvertently undercontextualized—
     some identifier referenced within has no available binding to value or brane member.
     An optimization we may perform is to coordinate the values of an abstract brane where
     possible. That brane with all possible values coordinated is a fully evaluated
     abstract brane. Normally, a recursive brane conditions on some computational result
     that depends on a parameter—a coordinate identifier that is named but unattached
     at the definition of the brane. In these cases, the abstract brane can be fully
     evaluated because the recursion depends on a parameter. But it is possible to 
     construct an abstract brane that calls itself irrespective of future parameters.
     Such a brane will be finite or infinite in depth depending on its construction.
     The abstract brane may therefore be permanently underevaluated due to its construction.
     An abstract brane is fully evaluated if the underevaluated members are due to detached
     identifier names.
5. An underevaluated FIR represents code that was expressed in Foolish, but it 
   has not been fully evaluated in the present UBC context. This FIR contains the AST of
   the expression, and a reference to the AB and IB that were used to attempt
   evaluation. But there are three possible reasons why the FIR is underevaluated:
   1. The AST is unprocessed.
   2. The expression has an unevaluated AST that contains recursive references to names
      that are not yet fully evaluated in the present context.

#### The UBC pass
1. If the UBC encounters Foolish text representing a brane, UBC holds it in memory.
2. UBC parses the Foolish text into an AST--this AST is always a brane
3. The brane is evaluated.

UBC expression evaluation step:
1. If the expression is a comment AST, store it as comment FIR.
2. If arithmetic expression, evaluate math and produce a value FIR.
3. If brane concatenation is involved, then process the branes based on precedence
   rules: detachment brane associating left using Left-First-Right-Last(LFRL), then brane
   concatenation associatively. The result is a FIR of a single brane.
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

(*) The expression is underevaluated if it makes underevaluated references.
When the underevaluation is caused by `↑` recursion, this single step materializes one
step of the recursion. The FIR is updated to replace `↑` with a FIR representing 
`[IB, AB] a`—the `a` being the AST of `↑`. This concludes the iteration.
Note, the next time we take a step in the UBC pass, this FIR is a brane concatenation that
can be processed by the brane evaluation. It may or may not produce another recursion
step depending on the program and context.

The UBC FIR processing has the following stages:
1. AST
2. FIR with all evaluation results coordinated. (Completion, another UBC does nothing)
3. FIR with some underevaluated coordinates. (In progress, another UBC step may
   produce changes)

Because of the step-wise evaluation, the brane tree is evaluated breadth-first. The
brane having finite size means each UBC step terminates in finite time. As long as
the UBC FIR is not complete, each step makes progress towards completion.
### The Multicellular Brane Computer
- [ ] TBD
### The Next Step in the Evolution Chain
- [ ] TBD

### Typing
Types have become very central to humanity's approach to organizing and understanding
programs. Foolish therefore will also propose to use types to understand Foolish.
Foolish would like to use the name *characterization* to refer to representations
and functionalities that are traditionally within the realm of type systems. Foolish
values, branes, expressions, and names are characterizables. Foolish
characterizations are Foolish programs that establish whether a characterizable has
or does not have said characterization. These are the most inclusive and the most exclusive
characterizations available in Foolish.

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

### Relational Coordinates
Relational coordinates are the *names* of brane members that we believe, as programmers,
are the most important aspects of a brane. When we think of coordinates, we think of
x and y coordinates in a two-dimensional Cartesian graph. But in reality, almost all
relationships that we consider can be modeled as functions of coordinates in some 
coordinate system. Most often we think of binary attributes, "hot or not", for example;
sometimes, an object is observable to have higher-dimensional coordinates such as SIC
or ZIP codes. There could also be higher-valence relationships, such as `{a,b,c} sorted_by_x`
or `{a,b,c} is_median_height`, which may return a boolean on whether `a.x`, `b.x`, `c.x` is
sorted, or `a` is the median of `b` and `c`, respectively.

So, in Foolish, a value that is accessible *after* a brane is fully evaluated is considered 
a coordinate—the value tells us how to relate to the brane that was just computed. A well-
coordinated brane is one that is fully evaluated to values so that computation can proceed
to access the coordinate with no undesirable consequences.
#### Single Coordinate Matching
```
{
    point_a = {x=1; y=2; z=3;};
    point_b = {x=1; y=3; z=4;};
    is_aligned = point_a.x == point_b.x; !! is_aligned = true;
}
```
#### Relational Coordinate Matching
- [ ] TODO: {a,b,c,d} product_filter [ae,be,ce,de]{ae.x==be.x; be.y==ce.z; ce.a==de.b}AND

### TODO Items
 Other features that we want to expand documentation on in the future:
 * Document the call-by-value versus call-by-reference semantics: Does it 
   matter?
 * When does Foolishness become concrete and when is it still Foolish?
 * Loops, Recursions, etc.
 * Characterization
   * Characterize some names as system-generated, e.g. branching.
 * Traceability.
 * Enhanced refactoring and intervention in computation.
 * Restatable programs for better comprehension and compression.
 * Generation of programs.
 * Program differentiation.
 * Program optimization.
 * AI adaptations to support Foolish language.
 * Human-in-the-loop programming.
 * Mutable branes. There is not much one can do to a brane. Given any line in 
   the brane: it can be deleted, appended, prepended, name changed, or value 
   changed.
 * Translation versus Transformation.
 * Specify the "Thought Block": how to stop evaluation.
 * Define more precisely the verb `to fool`.
 * Homomorphic encryption and equivalent "thought block" builtin.
 * Asynchronized execution.
 * Real-time computations.
 * Built-in masking of data from compute.
 * Library ecosystem for interactivity with Python/Java/C++/Rust/Go/etc. Other 
   popular open systems such as PyTorch, R, Octave, Apache systems such as Ray,
   Spark, Arrow, standards such as JSON, XML, HTML, PPML, ONNX, Protocol Buffers, etc.
## Development Notes
  - Concatenation is adjacency of brane-valued expressions (RPN), left-to-right;
    association is immaterial for denotation.

The Foolish language is a computer programming language environment. The 
language is specified to grow as we find the market fit for it in this age of 
rapidly expanding AI usage.

Foolish language begins as an exercise in vibe programming. We use AI to 
generate much of the code for a simplistic functional programming language. 
This language has a simple concept of a "brane" that largely resembles code 
blocks, lambdas, structs, classes, enums, records, and other similar constructs in 
modern-day programming languages. To that end, it is a curly-bracket-enclosed 
block of statements. Statements in Foolish are simply assignments of an 
expression to a name. Statements can also just be an expression; the name is 
anonymized in this case. Branes, once understood, can be inspected for their 
values at a certain key, or a certain line/statement number. It is permitted to
assign to the same name repeatedly. The language facilitates retrieval of all 
of its values, convenienced by syntactical sugaring.

Branes having unbound symbols are permitted. They are essentially brane 
functions. Function invocation, along with many other programming concepts such 
as concatenation, derivation, extending classes and traits, are all 
accomplished by brane concatenation. So to call a function `F` using parameters
listed in brane `P`, the reverse-polish expression creates the exact condition 
for invocation: `P F` concatenates `P` in front of `F`, setting values to names 
that `F` will use. Derivation simply concatenates updates to the end of the 
brane.

Note, this foolishness is unrelated to other previous efforts that may have 
used the name Foolish for the name of a language. However, I'm sure we're similarly
inspired by the same foolhardy impulses to this level of lunacy.

There's some kind of very permissive software license on the code and ideas 
expressed here. Please use foolishly, but do give back to the originators and 
contributors of this project so as to sustain either their work and lives, or 
otherwise at least keep alive the spirit to develop despite our fatuity.

---

## Appendix
### Keyboard Aid
#The ~/.XCompose file may contain the following short cuts
```
<Multi_key> <Up> : "↑" U2191 # Up Arrow
<Multi_key> <Down> : "↓" U2191 # Dow Arrow
<Multi_key> <Left> : "←" U2191 # Left Arrow
<Multi_key> <Right> : "→" U2191 # Right Arrow
<Multi_key> G : "▼" U9660 # Gradient
<Multi_key> D : "⍍" U234D # Differential
<Multi_key> t : "⊤" U234D # Truth, Top
<Multi_key> f : "⊥" U234D # False, Bottom


```
### Documentation Contributors

This README.MD was edited and improved with assistance from **Claude (Sonnet 4)**,
model ID: `claude-sonnet-4-20250514`, on October 29, 2025. The improvements 
included grammar corrections, spelling fixes, restructuring for better flow, 
updated table of contents, expanded explanations, proper line wrapping, and 
enhanced code examples while preserving the original vision and technical 
content of the Foolish programming language.
