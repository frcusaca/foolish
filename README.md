# Foolish Programming Language

*Where proximity meets computation, and containment creates clarity*

[![Java Tests](https://github.com/frcusaca/foolish/actions/workflows/java-tests.yml/badge.svg)](https://github.com/frcusaca/foolish/actions/workflows/java-tests.yml)
[![Scala Tests](https://github.com/frcusaca/foolish/actions/workflows/scala-tests.yml/badge.svg)](https://github.com/frcusaca/foolish/actions/workflows/scala-tests.yml)
[![Cross Validation](https://github.com/frcusaca/foolish/actions/workflows/tests.yml/badge.svg)](https://github.com/frcusaca/foolish/actions/workflows/tests.yml)
[![Rust Tests](https://github.com/frcusaca/foolish/actions/workflows/rust-tests.yml/badge.svg)](https://github.com/frcusaca/foolish/actions/workflows/rust-tests.yml)
[![License](https://img.shields.io/badge/license-Open%20Source-blue.svg)](#)
[![Status](https://img.shields.io/badge/status-Active%20Development-green.svg)](#)

## Welcome to the Future of Programming

Foolish is a revolutionary programming language that reimagines how humans interface with computers.
Built from the ground up for the 21st century, Foolish combines functional programming elegance with
intuitive natural metaphors to create a uniquely expressive and powerful development experience.

### Why Foolish?

- **Nature- and Life-Inspired Design**: Programming concepts mirror natural processes like cellular
  organization and proximity-based interactions found throughout living systems
- **High Semantic Throughput**: Express complex ideas with minimal cognitive overhead and remarkably
  low temporal latency
- **Advanced Search Capabilities**: Revolutionary brane search system for navigating and querying
  code structures with unprecedented flexibility
- **Compositional Architecture**: Functions, objects, and data structures unify seamlessly through
  elegant concatenation patterns
- **Concise and Sweet Syntax**: Extensive syntactical sugaring focuses your attention on logic
  rather than ceremony

### Our Vision and Goals

Foolish aspires to become the natural way anyone thinks about interacting with computers. We are
building toward a future where programming is:

- **Maximally Human**: Accessible, ergonomic, and adaptable design that is precise and
  high-functioning for human use
- **Safe and Precise**: Side-effect-free abstract functional programming foundation ensures
  reliability and predictability
- **Completely Transparent**: Full code transparency enables unprecedented understanding and
  debugging capabilities
- **Automatically Enhanced**: Built-in hooks support automatic programming, formal proofs,
  verification, and computing with uncertainty
- **Bidirectionally Communicative**: The system maximizes both human-to-computer and
  computer-to-human communication efficiency
- **Grounded in Reality**: Built-in methods seamlessly connect abstract thought to real-world
  implementations

A Foolisher might say the variable is *nye* (says 'nigh') when they encounter a FIR that has
not yet been evaluated fully, we say "that's a no-no" when we see `???`. Fully evaluated
expressions are values.

---

## Quick Start

```bash
mvn clean generate-sources compile test
```

Foolish programs use the `.foo` extension and embrace a philosophy where **proximity creates
combination** and **containment enables organization**. The language provides rigorous abstraction
capabilities while maintaining interfaces that ground your computations to the physical and
biological realities you want to model.

Additional implementation notes are in the docs folder.

## Key Features That Set Foolish Apart

### 🧬 Bio-Inspired Programming Model

Unlike traditional languages that force you to think in terms of machines, Foolish lets you think in
terms of natural systems. Branes mirror cellular organization, proximity drives interaction, and
containment enables natural hierarchies.

### 🔍 Revolutionary Search as First-Class Citizen

Stop scrolling through endless files. Foolish's integrated search system lets you query your code
from within the language itself. Find code by variable name, by value, or by association.

### 🎯 Functional Purity with Real-World Grounding

Enjoy the safety and predictability of pure functional programming while maintaining seamless
interfaces to the messy, stateful real world. The best of both paradigms, unified.

### 🔗 Natural Composition Through Proximity

Functions, data, and objects combine simply by being placed near each other. No complex import
systems, no verbose inheritance hierarchies—just natural, intuitive composition that mirrors how
ideas naturally combine in human thought.

### 🍯 Sweetened Syntax, Powerful Semantics

Extensive syntactical sugar means you write what you mean, not what the compiler wants. Focus on
your logic while Foolish handles the ceremony.

### 🤖 Built for the AI Age

Native support for uncertainty, automatic program generation, formal verification, and human-AI
collaborative programming. Foolish doesn't just run on computers—it bridges human and computational
intelligence.

### 👼 Built for Good

As we stand at the precipice of evolution, we must imbue our creations with all the best that we
have come to know. Foolish aims to be built for good, not evil.

---

## Table of Contents

- [Branes: The Foundation](#branes-the-foundation)
- [Sizes](#sizes)
- [Comments](#comments)
  - [Line Comments](#line-comments)
  - [Block Comments](#block-comments)
- [Expressions and Values](#expressions-and-values)
- [Names and Scope](#names-and-scope)
  - [Scope of Name](#scope-of-name)
- [The Unknown](#the-unknown)
- [Renaming](#renaming)
- [Advanced Features](docs/ADVANCED_FEATURES.md)
  - [Brane Operations](docs/ADVANCED_FEATURES.md#brane-operations)
  - [Search System](docs/ADVANCED_FEATURES.md#search-system)
  - [Detachment and Parameters](docs/ADVANCED_FEATURES.md#detachment-and-parameters)
  - [Control Flow](docs/ADVANCED_FEATURES.md#control-flow)
  - [Recursion](docs/ADVANCED_FEATURES.md#recursion)
- [Ecosystem](docs/ECOSYSTEM.md)
  - [Computer Reading Branes](docs/ECOSYSTEM.md#computer-reading-branes)
  - [The Unicellular Brane Computer (UBC)](docs/ECOSYSTEM.md#the-unicellular-brane-computer-ubc)
  - [The Multicellular Brane Computer](docs/ECOSYSTEM.md#the-multicellular-brane-computer)
  - [Typing](docs/ECOSYSTEM.md#typing)
  - [Relational Coordinates](docs/RELATIONAL_COORDINATES.md)
- [Development Notes](docs/DEVELOPMENT_NOTES.md)
- [TODO Items](docs/TODO_FEATURES.md)
- [Appendix](docs/APPENDIX.md)
  - [Styles](docs/STYLES.md)
  - [Keyboard Aid](docs/APPENDIX.md#keyboard-aid)
  - [Documentation Contributors](docs/APPENDIX.md#documentation-contributors)

---

## Branes: The Foundation

The Foolish language is in its most primitive form a containment and organization of values. We
contain values in something called a *brane*. The brane brings to mind concepts such as cell or
nucleus mem*brane*. The Foolish brane resembles traditional mathematical and programmatic concepts
such as sets, lists, maps or associative arrays, structs, enums or records. Foolish uses curly
braces to enclose values `{}`:

```foolish
{}                    !! Empty brane
{{}}                  !! Brane containing an empty brane
{{};{{}};{{{}}};}     !! Complex nested structure with multiple branes
{🌌={};}              !! Supports customizable alphabet
{愚=↑;}
{👶=👍;}
```

This ability to create containment will ultimately help us organize ideas. Branes can be nested
arbitrarily deep, allowing for sophisticated hierarchical data structures that mirror how we
naturally think about complex systems. Just as biological cells contain organelles, which contain
molecules, which contain atoms, Foolish branes can contain other branes in a natural, intuitive
manner.

## Sizes

In the physical world, containment membranes and units of organization tend to have observably
limited size. Cells can only be so big before they split or die. Atomic and subatomic objects have
to be so close, or otherwise the forces that keep them together stop working. Therefore, the Foolish
brane is also limited in size. The Foolish brane certainly should have finite size, and depending on
the computer, it may have a specified limit on the number of entries.

Inside the brane, it is a one-dimensional object. Its entries line up one after another. The true
dimension of entries are computational dependencies. So depending on the code inside, the actual
dependency axis could have smaller dimensions best visualized as a dependency DAG. The dependency
DAG and its subdimensions branch out and progress forward in their own time dimensions. Sometimes
more than one timeline merges together by being involved in an expression that refers to all of
them. Overall, the brane's fracturing timelines can be linearized, by stringing them into the
sequence of code as they originally appear inside the Foolish code, into a single time dimension.

## Comments

Comments in Foolish are expressions that contain unparsed Foolish. They are generally escaped using
multiple exclamation marks. Comments are part of the program that have no expressive effect on the
evaluation of the program.

### Line Comments

Line comments are like those from standard languages. The marking that begins the line comment is a
double exclamation mark `!!`.

```foolish
{
	!! This is a comment inside the brane
	!! This is another comment inside the brane.
	!! We can even exclaim inside a comment !!!
	!! - [ ] TODO: Check that this is possible ----^^^
}
```

### Block Comments

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

## Expressions and Values

Aside from comments, branes may contain expressions. Expressions are symbols that follow Foolish
rules and should be evaluable to a fixed value. Here are a few integer expressions inside a brane:

```foolish
{
	1;    !! This is the number 1
	2;    !! This is the number 2
	3.14; !! Floating point numbers work too
	"hello"; !! Strings are also expressions
}
```

A brane written in Foolish is itself an expression.

## Names and Scope

One very powerful concept we have for abstracting thoughts and thinking of complex matter with
complex properties and interactions is the substitution of the statement or object of consideration
with a name. Names such as `x`, `y`, `foolish`, `programming language`. So this is an important
concept in Foolish that we are able to name value expressions using the naming operator `=`:

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

Names are selected from an alphabet that excludes the reserved symbols of the language. Names should
include a small non-breaking white space character, currently `_`. Ideally it should be a
non-breaking thin space &thinsp; "&thinsp;". But nothing is able to render these spaces right now,
so we also include `_` for spacing out complicated nouns.

```foolish
{

	a_name=0;
	a&#x2009;name=1;   !! Thin space in hexmal
	a&#8201;name=1;    !! Thin space in decimal
	a&thinsp;name=2;   !! Thin space word entity
	a&hairsp;name=3;   !! hair space
	a&#x202F;name=4;   !! narrow non-breaking space
	a&#x2060;name=5;   !! word joiner
}
```

Foolish expressions and values inherently do not have names. They are only marked with names when
the naming operator `=` is used. So the following brane:

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

**Note on Coordinates**: In Foolish, names are also called *coordinates*. This terminology reflects
the idea that names serve as navigational reference points within branes, allowing us to locate and
access specific values. When we use a name to access a value (such as `brane.x`), we are accessing
that brane's coordinate. See [Relational Coordinates](docs/RELATIONAL_COORDINATES.md) for a detailed
discussion of how names function as coordinates in relational contexts.

### Scope of Name

Names are scoped to "before the current expression". So the current naming operation is not yet in
scope. This is similar to how many programming languages work. So the following brane:

```foolish
{
	a=1;
	a=a+1; !! a=2
	a=a+1; !! a=3
}
```

Such a convention has the direct benefit of delaying recursive concepts until more thought is put
into the line of code.

## The Unknown

The ***NK*** (*Not Knowable*, pronounced "no-no") is of paramount importance to us, therefore we
dedicate a symbol to express the *NK* state in Foolish `???`. Here we declare we do not know the
answer:

```foolish
{
	answer=???;
}
```

In fact every unnamed expression is an assignment to an *NK* name:

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

## Renaming

Foolish permits reusing names. If one were to think of the semantics of static single assignment,
this would be how Foolish interprets a reused name.

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

NB: we did preserve the first assignment `a=1` as part of the brane for historical accuracy.

---

For advanced features including brane operations, the search system, detachment and parameters,
control flow, and recursion, see the [Advanced Features](docs/ADVANCED_FEATURES.md) documentation.

For implementation details including the Unicellular Brane Computer (UBC), typing systems, and
relational coordinates, see the [Ecosystem](docs/ECOSYSTEM.md) documentation.
