---
foop: 24
title: Humanizing Sequencer formatting specification
author: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
status: Draft
type: Specification
created: 2026-06-03
phase: phase-2
supersedes: []
---

# FOOP-42: Humanizing Sequencer formatting specification


All FIR are represented in the output.

## Literals
Integer literals are rendered as interger. Int(10) looks like `10`


## Brane
Branes are always surrounded by curly braces, it starts with '{' and ends with '}'
Branes are always rended multi-line
```
     {
       a=1;
       b=2;
     }
```
Statements of brane always indents configurably by B_DENTS=2. nesting simply adds the
same indent the opening brace received
```
     {
       a=1;
       b={
           c={
               d=10;
               e=10;
           }
       }
     }
```
So if we annotate the spaces
```
     {
!!   bb<-- These are the root brane's B_DENTS
       a=1;  !! at this point, we know only there's B_DENTS deep
!!   bb<-- These are the root brane's B_DENTS
       b={ WOCONSTANIC !! When the object is not one of CONSTANT or INDEPENDENT, the Nyese is displayed
!!     aa<--- b's brane these are called A_DENTS, the Alignment indents, these vary dependeing on where an experssion's opening braces are located
!!       bb<-- Note the WOConstanic received these B_DENTS too above.
           c={
!!   bbaabb<-- b's B-dents after the other accumulated indentations for defining c
               d=10;
!!         aa <-- c's A_DENTS to aligh with opening braces
!!           bb <-- c's B_DENTS to indent brane
               e=10;
!!   bbaabbaabb^----that's how we figure out where e starts.
           }  !! To put close
```


## Searches
 look like this:
```
{
   r = Search(pattern='^x$', dir=BACKWARD, UNANCHORED, ECONSTANIC);
   r = Search(pattern='^x$', dir=BACKWARD, UNANCHORED, WOONSTANIC,
              result=Search(...));
   r = Search(pattern='^x$', dir=BACKWARD, UNANCHORED,
              result=1);
   r = Search(pattern='^x$', dir=BACKWARD, UNANCHORED, NK, result=???);
   r = Search(pattern='^x$', dir=BACKWARD, UNANCHORED, NK, result={ a=???;});
   r = Search(pattern='^x$', dir=BACKWARD, ANCHORED, ECONSTANIC);
   r = Search(pattern='^x$', dir=BACKWARD, ANCHORED, WOConstanic,
              result=1);
   r = Search(pattern='^x$', dir=BACKWARD, ANCHORED, WOConstanic,
              result=Search(...));


   r = HeadTail(HEAD, WOCONSTANIC); !! These can only be anchored btw.
   r = HeadTail(HEAD, result=1);
   r = HeadTail(TAIL, ECONSTANIC,
                result=Search(...
                )
   );
```
Search are one line, unless there is a result, if it has a result, then result is printed starting next line.


## NK
```
{
  a= ??? (division by zero)
  ??? (unknown identifier)
}
```
these do not require state, Nyes for ??? is always NK.
```
