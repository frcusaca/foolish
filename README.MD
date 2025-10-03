# Foolish language

## Development and Status:
  - `mvn clean generate-sources compile test`  : 
  - [![Java CI](https://github.com/frcusaca/foolish/actions/workflows/tests.yml/badge.svg)](https://github.com/frcusaca/foolish/actions/workflows/tests.yml)


## Introduction
The Foolish language project welcomes you to join us Foolish programmers. The Foolish program file has extension `.foo`. It is a language designed and built in the 21st century. It is functional and typed in its own ways. The hope is that as we develop this language, it becomes a very reliable way for humans to interface with computers with high semantic throughput with very low temporal latency. Languages and data representations are themselves very abstract ideas that either we think of in our heads or we use computer to compute. Foolish language is rigorous and easy to use in full abstraction, but also has interfaces that grounds it to realities that we wish to compute and communicate about.

## The Language
### Branes
The Foolish language is in its most primitive form a containment and organization of values. We contain values in something called a *brane*. The brane brings to mind concepts such as cell or nucleus mem*brane*. The Foolish brane resembles traditional mathematical and programmatic concepts such as sets, lists, maps or associative arrays, structs, enums or records. Foolish uses curly braces to enclose values `{}`:
```
   {}
   {{}}
   {{};{{}};{{{}}};}
```

This ability to create containment will ultimate help us to organize ideas.

### Comments
Comments in Foolish are expressions that contain unparsed Foolish. They are generally excaped using multiple exclamation marks. Comments are part of the program that has no expressive effect on the evaluation of the program.

#### Line Comments
Line comments are like those from standard langauges the marking that begins the line comment is double exclamation mark `!!`

```
   {
      !! This is a comment inside the brane
      !! This is another inside the brane.
      !! We can even exclaim inside a comment !!!
      !! TODO: Check that is possible --------^^^
   }
```


#### Block Comments
Block are enclosed by a pair of consecutive triple exclaimantion marks:
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


## Expressions
Aside from comments, branes may contain expressions. Expressions are symbols that follow Foolish rules for an expression and should be evaluable to a fixed value. Here are a few integer expressions in side a brane:

```
   {
      1; !! This is the number 1
      2; !! This is the number 2
   }
```
A brane writen in Foolish is itself an expression.

## Names
One very powerful concept we have for abstracting thoughts and think of complex matter with complex properties and interactions is the substitution of the statement or object of consideration with a name. Names such as `x`, `y`, `foolish`, `programming language`. So this is an important concept in Foolish that we are able to name value expressions using the naming operator `=`:

```
   {
      a=1;
      b=2;
      c=3; comment=!!This is a named line comment
      d=!!!This is a named block comment !!!
      !! etc.
   }
```

Names are selected from an alphabet that excludes the reserved symbols of the language. Names should include a small non-breaking white space character, currently `_`. Ideally it should be a non-breaking thin space \&thinsp "&thinsp;". But nothing is able to render these spaces right now, so we also include `_` for spacing out complicated nouns.
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
### Renaming
Foolish permits reusing names. If one were to think of the semantics of static single assignment, this would be how Foolish interprets a reused name.
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
NB: we did preserve the first assignment `a=1` as part of the brane for historical accuracy.

## Proximity is Combination
Branes combine freely. Listing two branes either in code or by named reference means it is desired that the two branes be concatenated. The following code snippets have the same meaning in Foolish:
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
This kind of resembles some physical and biological systems where like-kinded things combine when they come to close proximity to each other.

### The Brane Concatenation
Concatenation operation is associative. Here is an example of branes acting like functions, naturally brane concatenation invokes functions in RPN.

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

One will note that it didnt' matter how evaluation occurred either `(q p) f` or `q (p f)` yields the same `result`:
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

Another example of using concatenation is derivation. To mimic Object-Oriented programming inheritance. By default, if we extend a brane with another by post-concatenation, method and member references follow "static invocation" rules.

```
   {
      class_a = { a=1; b=2}
      class_b = class_a {c=3}
      a_b     = class_b
      !! So, class b is inhereting everything from a with addition of `c`
   }
```

The only way to create the effect of dynamic invocation is to create interfaces and specify them at programming time:
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

## Brane Search
Once we have established a bundle of named value expressions that we call brane, we expect there to be some order and conceptual meaning to this collection--there is meaning in it and there is meaning in their collection together. It may become useful for us to inspect this brane for some particular aspect that interest us. We accomplish this one of two ways. We can either search for something that we have named, or search for a particular way we computed something.

Brane search expresses are always written postfix except for during naming.
### Name Search
Conventional regular expressions can be used to search the name space of a brane. Supposing the brane is a text document of names. After each search, if search continues, then the cursor is placed at beginning of the name that it has found.
   * backward search from a cursor is after the last character in the brane's name document.
      * `.` means to find the last name matching the regexp: `brane.result` means the last value to be named `result` in the brane. This interpretation has similar effect as the normal dereferencing symbol `.` used in other languages.
      * `$` means to find just the last value in a brane `{result=10;}$` would extract the value 10.
      * `b#-5` means to find the fifth-last value irrespective of names. The last entry in a brane is `#-1` per modern negative array indexing convention.
   * forward search from a cursor that is before the first character of hte brane's name document.
      * `/` means to find the first name machine the regular expression: `f/parameter` gives us the parameter value an evaluation of `f` used.
      * `^` means the very first value in the brane.
      * `b#5` means the SIXTH value in the brane, we use zero-based array indexing for non-negative indexes as per convention.
In all cases of single regular expression matches, match failure is a compilation error if possible and runtime error if it occurs.

#### Bulk Name Search
In the case regular expression is used for searching for multiple brane content, we use the operators `//` and `??` to ask for a brane that contains all matchine entries of the search-subject brane.
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
### Expression Search
TBD.
### Cursor Movements
Recall each search result puts the cursor at start of the line just before variable names. We now define a few operators to move that search cursor.

| Expressions| Effect on Cursor|
|:----------:|:---------------:|
| `↑`,`#@-1`     | Moves cursor to the begining of the line referring to this brane |
| `↓`,`#@+1`     | Moves cursor to the begining of the brane that is the value of this line|
| `←`,`#-1`      | Moves cursor to the start of previous line|
| `→`,`#+1`      | Moves cursor to the end of this line, effectively at the start of the next line |

So we could use stack based syntax if we so desire:
```
   {
      f = {result=↑#-1 + ↑#-2};
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

### Unanchored Brane Search Leads to Self Awareness
Notice that everytime we refer to a named value by name, (aka reference a value by a variable name), we are actually doing a `.` search on the brane and it's parents as if the brane ended before the start of the current line. So all the searchs can actually be invoked without following a brane. In that case it would mean to search present brane with the cursor just before the beginning of the current line.
```
   {
	1;
	2;
	c= #-1 + #-2;
   }
```
TODO: Augment syntax to make it easier to read for humans and possible to parse for compilers.

Since the Foolish document is read top to bottom, it is not possible to search downward past current position. so these symbol are not permitted in an unanchored search `↓`,`#@+1`, `→`,`#+1`. However, if an existing search expression is in progress, it is possible to anchor these searches to a previous unanchored search result. "??_*result#+1" is permitted.


### Naming a search result
With the introduction of search, we now have the ability to name results of search. A shorthand is introduced to make it easier to extract single values out of branes. the `=` naming operator may be followed by a brane search expression and then by the brane. The meaning is that we will apply the search on the RHS brane before assigning that value to the name. The brane `{ V =E B }` where `V` is a name and `E` is a brane search expression, and `B` is a brane,(NB no space between `=E`). That expression means `{V = B E}`

Recall the brane concatenated function invocation above, we have changed one line, which is that during the assignment of result, we have added a brane search expression(`$`) right after the naming operator(`=`). This puts the actual number into the result instead of the brane that computed the number.
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
The flexibility of expression helps the Foolish user greatest flexibility in extracting use information from the right place, and at the same time place emphasis on code symbols to maximize readability and communication efficiency.

# TODO:
 Other features that we want to expand documentation on in the future:
 * Loops, Recursions, etc.
 * Characterization
 * Tracability.
 * Enhanced refactoring and intervention in computation.
 * Restatable programs for better comprehension and compression.
 * Generation of programs
 * Program differentiation.
 * Program Optimization.
 * AI adaptations to support Foolish language.
 * Human-In-the-loop programming.

Notes:
  - Concatenation is adjacency of brane-valued expressions (RPN), left-to-right; association is immaterial for denotation.

The Foolish language is a computer programming language environment. The language is specified to grow as we find the market fit for it in this age of rapidly expanding AI usage.

Foolish language begins as an exercise in vibe programming. We use AI to generate much of the code for a simplistic functional programming language. This language has simple concept of a "brane" that largely resembles code blocks, lambda's, struct, class, enum, records and other similar constructs in modern day programming languages. To that end, it is curly bracket enclosed block of statements. Statements in Foolish are simply assignments of an expression to a name. Statements can also just be an express, the name is anonymized in this case. Brane, once understood, can be inspected for it's values at a certain key, or a certain line/statement number. It is permitted to assign to the same name repeatedly. The language facilitates retrieval of all of its values convenienced by syntactical sugaring.

Branes having unbound symbols are permitted. They are essentially brane functions. Function invocation, along with many other programing concepts such as concatenation, derivation, extending classes and traits, are all accomplished by brane concatenation. So to call a function `F` using parameters listed in brane `P`, the reverse polish expression creates the exact condition for invocation `P F` concatenates `P` in front of `F` setting values to names that `F` will use. Derivation simply concatenate updates to the end of the brane.

Note, this foolishness is unrelated with other previous efforts that may have used the name Foolish for name of a language. However, I'm sure we're similarly inspired by the same foolhearty impulses to this level of lunacy.

There's some kind of very permissive software license on the code and ideas expressed here. Please use foolishly, but do give back to the originators and contributors of this project so as to sustain either their work and lives, or otherwise at least keep alive the spirit to develop despite our fatuity.
