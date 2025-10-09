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
- **Maximally Human**: Accessible, ergonomical, and adaptable design that is
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

### 🔍 Revolutionary Search as First Class Citizen
Stop scrolling through endless files. Foolish's integrated search system lets 
you query your code like inside the IDE. find code by variables name, by values 
or by association.

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
As we sit on the precipice of evolution, we must imbue our creations with all
the best that we have come to know. Foolish aims to be built for good not
evil.

---

## Table of Contents

- [Core Concepts](#core-concepts)
  - [Branes: The Foundation](#branes-the-foundation) 
  - [Expressions and Values](#expressions-and-values)
  - [Names and Scope](#names-and-scope)
  - [The Unknown](#the-unknown)
- [Advanced Features](#advanced-features)
  - [Brane Operations](#brane-operations)
  - [Search System](#search-system)
  - [Detachment and Parameters](#detachment-and-parameters)
  - [Control Flow](#control-flow)
  - [Recursion](#recursion)
- [Language Reference](#language-reference)
  - [Comments](#comments)
  - [TODO Items](#todo-items)
- [Development Notes](#development-notes)

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
```

This ability to create containment will ultimately help us organize ideas. 
Branes can be nested arbitrarily deep, allowing for sophisticated hierarchical 
data structures that mirror how we naturally think about complex systems. Just 
as biological cells contain organelles, which contain molecules, which contain 
atoms, Foolish branes can contain other branes in a natural, intuitive manner.

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
```
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
```
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
```
{
    1;
    2;
    3;
}
```
is a brane with three unnamed values. The following brane:
```
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
```
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
```
{
    answer=???;
}
```
In fact every unnamed expression is an assignment to an unknown name:
```
{1;2;3;}
```
is shorthand to
```
{
    ???=1;
    ???=2;
    ???=3;
}
```

### Renaming
Foolish permits reusing names. If one were to think of the semantics of static 
single assignment, this would be how Foolish interprets a reused name.
```
{
    a=1;
    b=a;
    a=2;
    c=a;
}
```
is equivalent to
```
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
```
{
    a=1;
    b=2;
}
```

and
```
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
like functions, naturally brane concatenation invokes functions in RPN.

```
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
```
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

```
{
    class_a = { a=1; b=2}
    class_b = class_a {c=3}
    a_b     = class_b
    !! So, class b is inheriting everything from a with the addition of `c`
}
```

The only way to create the effect of dynamic invocation is to create interfaces
and specify them at programming time:
```
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
   * backward search from a cursor is after the last character in the brane's 
     name document.
      * `.` means to find the last name matching the regexp: `brane.result` 
        means the last value to be named `result` in the brane. This 
        interpretation has similar effect as the normal dereferencing symbol 
        `.` used in other languages.
      * `$` means to find just the last value in a brane `{result=10;}$` would 
        extract the value 10.
      * `b#-5` means to find the fifth-last value irrespective of names. The 
        last entry in a brane is `#-1` per modern negative array indexing 
        convention.
   * forward search from a cursor that is before the first character of the 
     brane's name document.
      * `/` means to find the first name machine the regular expression: 
        `f/parameter` gives us the parameter value an evaluation of `f` used.
      * `^` means the very first value in the brane.
      * `b#5` means the SIXTH value in the brane, we use zero-based array 
        indexing for non-negative indexes as per convention.
In all cases of single regular expression matches, match failure is a 
compilation error if possible and a runtime error if it occurs.

#### Bulk Name Search
In the case regular expression is used for searching for multiple brane content,
we use the operators `//` and `??` to ask for a brane that contains all 
matchine entries of the search-subject brane.
   * `b//tmp_.*` the result is a brane that refers to the `b`
```
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
```
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
previous unanchored search result. For example, "??_*result#+1" is permitted.
Backward search is conducted in the following manner to discover their 
occurrences:
1. Start with the cursor at the beginning of the current line.
2. Search the current brane backwards from the cursor. Return if found.
3. If not found, search raises the cursor to current brane's parent expression,
   the cursor is placed at the beginning of that line.
4. goto step 2.
5. If no parent brane, then search returns ???

##### Forward  Search
Forward search is performed line by line to the end of the brane. Forward 
search is not permitted to raise the cursor to parent branes. So forward search
is limited to the current brane only.

##### Depth Search
[ ] TODO: Extend search into the computation graph with more syntactical 
capabilities for finding computational results.

### Value Search
Forward value search is triggered by the `:` operator. It is a forward search 
operator on a brane that searches for a value that can be considered equivalent
to the search parameter. `::` is the bulk forward value search operator.
```
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
Single and bulk backward value search is triggered by `<:` and `<<:` 
respectively.
The part of the search result that is used depends on the context. when used in
the RHS of a naming expression, the value is used. When used on the LHS of a 
naming expression, the name is used.


##### Expression Search
[ ] TODO: Extend search into the computation graph with more syntactical 
capabilities for finding computational results.
##### Search Combinations
[ ] TODO: Document combinations of name and value search.
### Search Cursor Movements
Recall each search result puts the cursor at start of the line just before 
variable names. We now define a few operators to move that search cursor.

| Expressions| Effect on Cursor|
|:----------:|:---------------:|
| `↑`,`#@-1` | Moves cursor to the begining of the line referring to this brane |
| `↓`,`#@+1` | Moves cursor to the begining of the brane that is the value of this line|
| `←`,`#-1`  | Moves cursor to the start of previous line|
| `→`,`#+1`  | Moves cursor to the end of this line, effectively at the start of the next line |

So we could use stack based syntax if we so desire:
```
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

[ ] TODO: Document searching upward for variables scoped in parent branes.

#### Unanchored Brane Search Leads to Self Awareness
Notice that everytime we refer to a named value by name, (aka reference a value
by a variable name), we are actually doing a `.` search on the brane itself and
it's parents as if the brane ended before the start of the current line. So all
the searches can actually be invoked without following a brane. In that case it
would mean to search present brane with the cursor just before the beginning of
the current line.
```
{
    1;
    2;
    c= #-1 + #-2;
}
```
[ ] TODO: Augment syntax to make it easier to read for humans and possible to 
parse for compilers.


### Naming a search result
With the introduction of search, we now have the ability to name results of 
search. A shorthand is introduced to make it easier to extract single values 
out of branes. the `=` naming operator may be followed by a brane search 
expression and then by the brane. The meaning is that we will apply the search 
on the RHS brane before assigning that value to the name. The brane 
`{ V =E B }` where `V` is a name and `E` is a brane search expression, and `B` 
is a brane,(NB no space between `=E`). That expression means `{V = B E}`

Recall the brane concatenated function invocation above, we have changed one 
line, which is that during the assignment of result, we have added a brane 
search expression(`$`) right after the naming operator(`=`). This puts the 
actual number into the result instead of the brane that computed the number.
```
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
```
[
    a = ???;
    b = ???;
]
```

This brane detaches the names a and b. So to declare a function with formal 
parameter names that might already be in use, one could perform detachment 
during a naming expression.
```
{  
    fn=[
        a=???;
        b=???;
    ]{
        result=a+b;
    }
}
```
The detachment is right associative brane that operates on the brane to its 
right side.

The original example of function definition probably needed this detachment for 
it to be a useful function:
```
{
    f = [↑=???]{result=↑#-1 + ↑#-2};
}
```
Ideally, an upward search like this will block out ALL references to variables 
from the parent brane and above. So that in this function
```
{
    f = [↑=???]{result=↑#-1 + ↑#-2 - universal_constant};
}
```
the `universal_constant` will be specified by caller as a parameter.

#### Complete Detachment
We now introduce complete detachment using name search to completely detach 
from a large number or all of the names and values.
```
{
    clean_heap=[tmp_*=???]old_heap_brane
}
```
Or even complete detachment like this:
```
{
    pure_func=[*=???]{...}
}
```

#### Detach to Default
In order to mimic default parameters in many modern programming languages, we 
shall offer a way to specify detachment that sets not only the expectation for 
what needs to be provided but also what happens when that expectation is not 
fulfilled.
```
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

When default branes are next to each other, they are right associative—the 
rightmost default brane is overridden by the default brane to its immediate 
left first and foremost. Default brane concatenation always has higher priority
than their concatenation with branes. When a default brane is between branes it
is always right-associative—it is applied to the brane on the right side before
other brane operations.
### Control Flow
Branching is accomplished using search.
```
{
    ifblock = {
        if x==1 then         !! `if k then` fools to the expression `f'condition=k;`
        result = 1;
        else if x==2 then    !! `else if k then` fools to the expression `f'condition=k;`
        result = 4;
        else                 !! `else` fools to the expression `f'condition=true;`
        result = 2;
        f'condition=10;      !! ERROR, cannot assign to foolish names
    }
    result = ifblock/f'condition:true→;   !! Search for a true value, then forward cursor to the next line.
    result = condition⇒;                !! shorthand for `/f'condition:true→` !! That uses a double-lined arrow in '=⇒' which is different from next statement single-arrow
    result => condition;                !! shorthand for `/f'condition:true→`
}
```
### Recursion
Since the brane search starts with cursor at beginning of the current line, 
finding the current brane is easy with `↑`. So recursion is easy to express:
```
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
Here, it seems that we often perform loops or other complex operation where a 
single line performs updates to several states. Most prototypical example is 
this python code
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
And we proceed to use fib while we operate on triangle numbers as well. This 
Fibonacci counter continues merrily on its own.

But so far Foolish has been entirely functional.
- [ ] TODO: How do we centrally organize several coordinated state updates in a 
single line of code?

#### Mutual Recursion
However, mutual recursion is not directly possible since we cannot search 
downward past the current line. The only way to create corecursion is to use an
unbound name and pass it into the context:
```
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
an AST. The units on which the computer works on are the expressions. The
largest expression being a brane, its processing take up much of the Foolish
Ecosystem.

### The Unicelluar Brane Computer (UBC)
Like our human minds, most of our computational resources have limited abilities.
A unicelluar Brane computer (UBC) is one that has just enough capacity to hold the AST
of a brane, and the ability to interpret and understand a single expression at a time.
This computer proceeds simply from beginning of the brane to the end of the brane
evaluating and creating new values to be stored in the brane. In order to process
expressions, the brane actually has two more sources of informtion: Ancestral and
Immediate. The Immediate Brane (IB) is the current contex that we have accumulated
inside the unicelluar brane computer so far, including name of the current expression.
The Ancestral Brane (AB) is the search context that contains the name of the expression
in an AST that defined the current brane, that expression's AB and IB.

In some cases, such as the recursive call to `↑`, the search finds a line of code that
has incomplete value. Note by incomplete value, we mean not the unkown value `???`, but a
value that the present UBC does not fully know the value of. In the case of this incomplete
value, it would appear that the simplest machine is unable to continue evaluating the value
of the Foolish expression. In fact, we don't even know all of the names or values this brane
will capture by its end. To continue computation, the UBC must stash the current execution
context, and evaluate that expression using the context "IB AB ↑". That says the UBC shall
evaluate the AST of the expression that created the present brane, resolving any searches
first using the current AB, then using current IB.
```
    {...{...{...

        !! At this point, we search up for name resolutions the AB below
        !! refers to this point and back and up
	f = {
            !! Start of IB
            one = 1;
            two = 2;
            !! End of IB
            if (continue) then
               r=[three=one+2;]↑ 
            else
               r=5
            !! Continue to code that we didn't know when we computed r
            four = 4;
        }
    ...}...}..}
```

In that example, in recursive call to compute r is evaluated using `[IB, AB] f`.
where the expression named `f` is available by Foolish source code only. IB and AB
would always be available at the recursive call by above definitions.

### The Multiceulluar Brane Computer
[ ] TBD
### The Next Step in the Evolution Chain
[ ] TBD


### TODO Items
 Other features that we want to expand documentation on in the future:
 * Document the call-by-value versus call-by-reference semantics: Does it 
   matter??
 * When does Foolishness become concrete and when is it still Foolish?
 * Loops, Recursions, etc.
 * Characterization
 * * characterize some names as system-generated, e.g. branching.
 * Tracability.
 * Enhanced refactoring and intervention in computation.
 * Restatable programs for better comprehension and compression.
 * Generation of programs
 * Program differentiation.
 * Program Optimization.
 * AI adaptations to support Foolish language.
 * Human-In-the-loop programming.
 * Mutable Branes. There are not much one can do to a brane. Given any line in 
   the brane: it can be deleted, appended, prepended, name changed, or value 
   changed.
 * Translation versus Transformation.
 * Specify the "Thought Block": how to stop evaluation.
 * define more precisely the verb `to fool`.

## Development Notes
  - Concatenation is adjacency of brane-valued expressions (RPN), left-to-right;
    association is immaterial for denotation.

The Foolish language is a computer programming language environment. The 
language is specified to grow as we find the market fit for it in this age of 
rapidly expanding AI usage.

Foolish language begins as an exercise in vibe programming. We use AI to 
generate much of the code for a simplistic functional programming language. 
This language has simple concept of a "brane" that largely resembles code 
blocks, lambda's, struct, class, enum, records and other similar constructs in 
modern day programming languages. To that end, it is curly bracket enclosed 
block of statements. Statements in Foolish are simply assignments of an 
expression to a name. Statements can also just be an express, the name is 
anonymized in this case. Brane, once understood, can be inspected for it's 
values at a certain key, or a certain line/statement number. It is permitted to
assign to the same name repeatedly. The language facilitates retrieval of all 
of its values convenienced by syntactical sugaring.

Branes having unbound symbols are permitted. They are essentially brane 
functions. Function invocation, along with many other programing concepts such 
as concatenation, derivation, extending classes and traits, are all 
accomplished by brane concatenation. So to call a function `F` using parameters
listed in brane `P`, the reverse polish expression creates the exact condition 
for invocation `P F` concatenates `P` in front of `F` setting values to names 
that `F` will use. Derivation simply concatenate updates to the end of the 
brane.

Note, this foolishness is unrelated with other previous efforts that may have 
used the name Foolish for name of a language. However, I'm sure we're similarly
inspired by the same foolhearty impulses to this level of lunacy.

There's some kind of very permissive software license on the code and ideas 
expressed here. Please use foolishly, but do give back to the originators and 
contributors of this project so as to sustain either their work and lives, or 
otherwise at least keep alive the spirit to develop despite our fatuity.

---

## Appendix

### Documentation Contributors

This README.MD was edited and improved with assistance from **Claude (Sonnet 4)**,
model ID: `claude-sonnet-4-20250514`, on October 8, 2025. The improvements 
included grammar corrections, restructuring for better flow, expanded 
explanations, proper line wrapping, and enhanced code examples while preserving
the original vision and technical content of the Foolish programming language.
