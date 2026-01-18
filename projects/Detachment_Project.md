This is the project notes for detachment brane. It might be slightly out of order and self conflicting. Please point out problems as we progress through. At the top here, I will include the best project progression I can come up with. Let's iterate a bit on how to prceded before doing these one at at time in separate PR's.

  - [x] Update the English description and examples befitting the docs/*.md files.
    - [x] Added SF/SFF markers section to NAMES_SEARCHES_N_BOUNDS.md
    - [x] Added Alarming Liberation section to NAMES_SEARCHES_N_BOUNDS.md
    - [x] Added CONSTANTIC rendering table to ECOSYSTEM.md
    - [x] Added Stay-Foolish quick reference to ADVANCED_FEATURES.md
  - [ ] Constantic as a known state:
    - [x] Document '?C?' rendering convention (in ECOSYSTEM.md)
    - [ ] Refactor identifier "NOT FOUND" state to CONSTANTIC state
    - [ ] Update all tests to use '?C?' for constantic rendering
    - [ ] For constantic branes, print contents to show what's constantic
  - [ ] Implement "get_my_identifiers" if it isn't already and unit test. For example it would be `this` for identifier FIR. It should recursively call same method on all expressions, BUT it does not descend into branes because those have different scopes.
  - [ ] Implement and test that parent chain on FIR is not circular.
  - [ ] Create a context manipulation FIR, it takes a single object FIR(called o below) as reference and should be the base class for many other FIRs.
    - ContexManipulationFIR, CMFir, let's call it c, it executes in 2 phases altering it's own state and it's object's state.
    - During intiation, c get's it's parent as expressions normally do. (parent brane with line number)
    - Internal Phase A:
      - if o is not either CONSTANTIC or CONSTANT, stepping the c will step o. (NB: o and it's children's parent has not changed)
      - If o becomes CONSTANT, then c is CONSTANT.
    - Internal Phase B: can be defined depending on situation. Default implementation is to continue resolution:
      - if o is constantic, then make a stay_foolish_clone of o, we will call o2, but within CMFir, o2 is now the object. sf-clone will reference CONSTANT objects, but make copies of anything CONSTANTIC. Note this works well because CONSTANT objects do not have constantic children, but coordinated constantic references cause their parents to be constantic.
      - set o2's sub-expressions (all expressions without descending into another brane), their parents are now updated to be o2. o2's parent is c. This means sub expressions will be able to use c's context to complete coordination.
      - set the state of c and o2 to identifier resolution state. (end step here)
      - continuing stepping on c will step o2 until it reaches constantic or constant.
      - when o2 is constantic or constant, c will have the same state.
        - value of c is value of o2
    - [ ] Identifier assignments are CMFir's:
      - '{c=a+b+c}', the RHS is a CMFir around the expression "CMFir(a+b+c)"
   - [ ] Implement '<>' and '<<>>'. these are called SF marker and SFF marker standing for Stay-foolish-marker and stay-fully-foolish-marker correspondingly.
    - [x] Document syntax and semantics (in NAMES_SEARCHES_N_BOUNDS.md)
    - [ ] Add grammar rules for '<>', '<<>>', '<=>', '<<=>>'
    - [ ] unit tests for behavior since it modifies FIR mostly.
    - [ ] approval tests to demonstrate behaviors.
    - Simplified implementation of '<>' a FIR containing it's contents. <>-FIR is a CMFir with the exception that Phase 2 moves the <>-FIR's own state to constantic directly without performing additional resolution.
    - Simplified implementation of '<<>>' This just re-initializes the content from AST and starts in CONSTANTIC state. It should be alarming if content has no AST.

  - [ ] Implement the simplest Detachment Brane--just perform detachment
    - [ ] the FIR and functionalities for it tested through unit tests.
    - [ ] copious and detailed Approval tests, use code below and generate more exhaustive coverage.
      - [ ] deep nesting
      - [ ] shadowing of all sorts (either detached, or redefined)
      - [ ] concatenated detachment branes function correctly.
      - [ ] concatenated detached branes in code: [][]{}[][]{})
      - [ ] concatenated detached branes by reference: d1 d2 d3 b1 d4 d5 d6 b2
      - [ ] all of these mingled with partially resolved states.

  - [ ] Implement Forward Search Liberation `[/...]` and `[#N]`
    - Forward search liberates identifiers by looking INTO the subsequent brane (not backward into scope)
    - `[/pattern]` liberates any statement whose identifier matches pattern
    - `[#N]` liberates the Nth statement (zero-indexed)
    - [ ] Add grammar rules for `[/pattern]` and `[#N]` syntax
    - [ ] Implement forward search matching in liberation brane FIR
    - [ ] Unit tests for forward search liberation:
      - [ ] `[/ingredient]{ingredient = expr}` - single forward match
      - [ ] `[/tmp_*]{tmp_a=1; tmp_b=2; result=3}` - pattern matching multiple
      - [ ] `[#1]{a=1; b=2; c=3}` - liberation by statement number
      - [ ] Mixed forward and backward: `[a, /result]{result = a + b}`
    - [ ] Approval tests demonstrating forward search behaviors
    - Design note: Forward search runs at parse/compile time since it looks into the brane's own statements, not runtime scope

* The filtering logic is Implemented as part of BraneMemory. Detachment branes shall use that feature of BraneMemory. The filters are active or inactive on DetachmentBrane's BraneMemory.
* Refactor search behavior (existing search) considering the backward detachment code
* Implement detachment brane FIR, and it's functions: which is to block coordination of some identifiers *once* during first assignment. Use Unit test to test it's behavior is correct. Essentially, a detachment brane turns inert once the brane it modifies reaches constantic.
* Implement simple detachment grammar of detachment brane [ pattern, pattern,..], thoroughly test all expected behaviors, including several of them together, including nesting and all those behaviors.

* Add `<>` and `<<>>` markers to syntax, and convert to FIR--test this step using approval tests.
  * '<<>>' Construct per Foolish code.


# Implement Detachment Brane with Free Variable Semantics
Overview
Implement detachment branes in the Foolish language using free variable semantics instead of permanent blocking. Detachment creates unresolved ordinates (free variables) that can be bound later through various mechanisms.

Core Semantic Requirements
Detachment Behavior
Detachment of a sub-brane B in Foolish creates free variables (unresolved identifiers) during assignment of B's Foolish expression to it's identifier within B's parent branes. These identifiers must be provided later when the identifier for B is referenced in a later statement.

CRITICAL: Detachment is NOT permanent blocking. It creates branes with unresolved ordinates that can be bound in various ways.

The branes always resolve and evaluates greedily when assigned in a brane statement. The detachments delays those events.

## Situation 1: Detachment During Parsing/Assignment
```foolish
{
    a = 1;
    f = [b]{r = a + b};   !! 'a' resolves to 1 (coordinated), 'b' is detached(uncoordinated)
    v =$ f                !! Since b is uncoordinated, the value is ???

    b = 3;
    vv =$ f;              !! vv = 4 (a is 1 from assignment coordination, b resolves to 3 just now)
    confirm 4 == vv;

    f2 = [a=3] f;         !! ALARMING! f already has 'a' resolved
                          !! Trying to detach 'a' again has no effect.
    v2 = f2               !! still 4 like vv, because nothing changed between f2 and f.
    confirm 4 == v2;

    f3 = [b]f;            !! This statement detaches `b` from an `f` that's ready to move past
                          !! CONSTANTIC, so b remains uncoordinated in f3
    b=4;
    v3 =$ f3              !! evaluates to 5
    confirm 5 == v3;
}
```

## Situation 2: Joining Multiple detached branes
```foolish
{
    a=1;b=2;c=3;d=4;z=-100
    f = [a,b,c  ] {d = a+b+c+z};  !! The symbol f refers to a brane that doesn't know a,b,c;
                                  !! end of that expression is 100 at constantic
    g = [ ,b,c,d] {o = -b-c-d-z}; !! The symbol g refers to a brane that doesn't know b,c,d;
                                  !! end of that expression is 100 at constantic

    r = f g ;             !! concatenates to the unresolved brane {d=a+b+c+(-100),o = -b-c-d-(-100);}
                          !! it then resolves to {d=1+2+3+(-100); o = -2-3-d-(-100);}
                          !!   That's f's d not parent's d.
                          !! it finally evalutes to {d=-94; o = -5;} 

    my_result1 =.d c      !! -94
    confirm -94 == my_result1;

    my_result2 =   c.o    !! -5
    confirm -5 == my_result_2;
}
```
Similarly, if "r=f g h i j k", all the branes with their unresolved varaiables are concatenated before resolution and evaluation occurs.

## Situation 3: I want to keep some variables unresolved but specialize a function to some parameters
```foolish
{
        a=1;b=2;c=3;d=4;
        f  =   [a,b,c,d] {r = a+b+c+d}; !! f has uncoordinated identifiers.
                                        !! It's identifier modifiers are marked as applied already.
        g  =   [a] f; !! At this point, f is ready to be coordinated, but the detachment brane
                      !! in front renews the fact that a is detached. b,c,d are attachable
  
        a=-1;b=-2;c=-3;d=-4;
        r =$ g;
        confirm 6 == r;
} 
```
This is more explicitly expressed using p-branes
```foolish{
        a=1;b=2;c=3;d=4;
        f  =   [a,b,c,d] {r = a+b+c+d}; !! f has uncoordinated identifiers.
                                        !! It's identifier modifiers are marked as applied already.
        g  =   [+b,c,d]<f>;             !! At this point, f is ready to be coordinated, It is protected
                                        !!   by the SF marker, and then modified with respect to b,c,d.
  
        a=-1;b=-2;c=-3;d=-4;
        r =$ g;
        confirm 6 == r;
}

```
We will introduce the lazy-eval angle brackets below in situation 4

## Situation 4: I need to copy most of a detached uncoordinated brane
Use lazy angle brackets `<>`, aka the stay-foolish mark, the SF marker. (Let's make a note of the SF maker in the terminology)

```foolish
{
        a=1;b=2;c=3;d=4;e=5;
        f  =   [a,b,c,d] {r = a+b+c+d+e};
        f2 =   <f>;                    !! use the SF marker to indicate this brane should be ordinated here without additional identification of explicitly detached ordinates.
                                       !! Note, the undetached `e` is actually resolved at this point.
                                       !! Primary meaning here is to reactivate the top level stated detachments to be applied during this assignment.

        f3 <=> f;                      !! Shorthand Staying-foolish assignment
                                       !! The entire RHS is kept uncoordinated
        r  =$  f      !! evalutes to 15 after it is coordinated to the context
        confirm 15 == r;

        d=5;
        r2 =$  f2     !! evalutes correctly to 16.
        confirm 16 == r2;

        d=6;
        r3 =$  f3     !! evalutes correctly to 17.
        confirm 17 == r2;
}
```
Btw, relatedly, let's introduce the stay fully foolish marker (SFF) marker
{
        a=1;b=2;c=3;d=4;e=5;
        f  = [a,b,c,d] {r = a+b+c+d+e};
        g  = <<f>>;    !! This pastes the source code for value of f's assignemnt here, as if we typed
        gg <<=>>  f;   !! Shorthand for above
        g2 = [a,b,c,d] {r = a+b+c+d+e};
        
        h  =  << {g,f} >>  !! Just constructs the Foolish AST and binds it to h, no identifier resolution and nothing is coordinated
```



Some Notes:
Let's call the function of detachment branes is to "modify identifier resolution" (MIR) The modifications stack according to the detachment brane's left associativity withitself. Say d1,d2,d3,d4 are detachment branes, with A and B as normal brane, (e.g., `A d1 d2 d3 d4 B`), the filtering logic proceeds
as follows:
1. Search for B's identifiers starts from just before `d1`, so it includes A's content, backwards.
2. Upon finding a matching identifier, the decision process flows right to left:
   - `d4` makes the initial decision: was that identifier detached or not?
   - `d3` can override that decision.
   - `d2` can override that decision.
   - `d1` (the left-most detachment) has the final say.
3. If the identifier remains undetached after this pipeline, it is considered a match.
4. Repeat if search is for multiple matchines using ??.

This "Left Overrides Right" precedence ensures that the outer-most detachment controls the final
visibility, consistent with the principle that "Left-Associate Liberation Branes" have higher
precedence.

Detachment brane affects lookup only once, so they have state inside, once resolved, they do not affect resolution. This is somewhat important to note both in the document and tests. In some cases, whena  brane, say B,  has a MIR for an identifier I, it receives another brane parameter, B2, that requires resolution of I, that brane may be assigned inside B, during that assignment, it may perform coordination of unresolved identifiers in B2. B's original MIR alrady did it's work, so it does not MIR B2's resolution process. (Odd, case to illustrate)
```foolish
    f  = [a,b,fn]{...; r =$ fn}
    g  = [a]{r=a+1}
    f2 = [a=1,b=2]<f>  !! Keeps fn unresolved, but a & b are now coordinated, fn is not
                       !!  This should be marked as alarming situation.
    a  = 10;           
    r  =$ [fn=g] f2;   !! g now seeks out 'a' during identification resolution phase of eval,
                       !!   and the MIR for f (andf2) should not block that. fn/g finds a is 10.
    confirm 11 = r;
```
)


Test 1: basicDetachmentDuringAssignment.foo
!! Test basic detachment during assignment
!! When f = [b]{r = a+b}, 'a' resolves at assignment time, 'b' is detached

{
    a = 1;
    f = [b]{r = a + b};
    b = 3;
    x =$ f;              !! Should be 4: a captured as 1, b resolves to 3

    !! Nested case: detachment within detachment
    {
        c = 10;
        g = [d]{
            inner = [e]{result = c + d + e};
            result =$ inner
        };
        !! c captured as 10, d detached, e is not found
        d = 20;
        e = 5;
        y =$ g;          !! Should be 35: c=10, d=20, e=5
    };

    !! Test with multiple detached ordinates
    h = [p, q, r]{sum = p + q + r};
    p = 1; q = 2; r = 3;
    z =$ h;              !! Should be 6
}

Test 2: alarmingUselessDetachment.foo
!! Test alarming behavior when trying to detach already-resolved variables

{
    a = 1;
    f = [b]{r = a + b};   !! 'a' resolves to 1, 'b' is detached

    !! This should trigger an alarm - 'a' is already resolved in f
    !! The detachment [a=3] has no effect
    f2 = [a=3] f;         !! ALARMING: trying to detach resolved ordinate 'a'

    b = 2;
    x =$ f;               !! Should be 3
    y =$ f2;              !! Should also be 3 (the [a=3] had no effect)

    !! Test alarming with multiple ordinates
    g = [c]{result = a + c};  !! 'a' already resolved (1), 'c' detached
    g2 = [a=5, c=10] g;       !! ALARMING: 'a' already resolved
                              !! Only 'c=10' should take effect

    z =$ g2;                  !! Should be 11: a=1 (original), c=10 (newly bound)
}

Test 3: pbranePartialApplication.foo
!! Test P-brane [+...] partial application from current scope

{
    !! Basic P-brane: bind some ordinates from current scope
    f = [a, b, c, d]{r = a + b + c + d};
    a = 3;
    c = 1;
    f2 = [+a, c]<f>;      !! Binds a=3, c=1 from current scope
                          !! f2 still has detached ordinates b, d

    b = 2;
    d = 4;
    x =$ f2;              !! Should be 10: a=3, b=2, c=1, d=4

    !! Change scope values - f2 already captured a and c
    a = 100;
    c = 200;
    y =$ f2;              !! Should still use a=3, c=1 from capture
                          !! But b, d resolve from current scope
    !! Expected: 3 + 2 + 1 + 4 = 10 (b,d from current scope)

    !! Nested P-brane application
    {
        g = [p, q, r, s]{sum = p + q + r + s};
        p = 1;
        q = 2;
        g2 = [+p]<g>;       !! Partially apply p=1

        r = 3;
        g3 = [+r]<g2>;      !! Further partially apply r=3

        s = 4;
        z =$ [q=10]<g3>;    !! Provide q=10 at evaluation
                          !! Expected: p=1, q=10, r=3, s=4 = 18
    };

    !! P-brane with explicit values mixed in
    h = [a, b, c]{result = a + b + c};
    a = 5;
    h2 = [+a, b=10]<h>;     !! a from scope (5), b explicit (10), c detached
    c = 3;
    w =$ h2;              !! Should be 18: a=5, b=10, c=3
}

Test 4: constanticBracketsAndAssignment.foo
!! Test constantic brackets <> and constantic assignment <=>

{
    !! Basic constantic brackets - capture all detached ordinates
    a = 1; b = 2; c = 3; d = 4;
    f = [a, b, c, d]{r = a + b + c + d};
    f2 = <f>;             !! Captures a=1, b=2, c=3, d=4 at assignment

    r =$ f;               !! Uses current scope: 1+2+3+4 = 10
    d = 5;
    r2 =$ f2;             !! Uses captured values: 1+2+3+5 = 11
    d = 6;
    r3 =$ f2;             !! Still 11 (f2's capture doesn't change)

    !! Constantic assignment <=> shorthand
    f3 <=> f;             !! Equivalent to f3 = <f>;
    r4 =$ f3;             !! Captures d=6: 1+2+3+6 = 12

    !! Constantic brackets with nested scopes
    {
        p = 10; q = 20; r = 30;
        g = [p, q, r]{sum = p + q + r};
        g2 = <g>;         !! Captures p=10, q=20, r=30

        {
            !! Inner scope changes variables
            p = 100;
            q = 200;
            x =$ g;       !! Uses current scope: 100+200+30 = 330
            y =$ g2;      !! Uses captured: 10+20+30 = 60
        };
    };

    !! Important: constantic brackets only affect brane references
    !! Not freshly parsed branes
    h1 = <[x, y]{z = x + y}>;    !! <> has no effect on fresh brane
    h2 = [x, y]{z = x + y};      !! Equivalent to h1

    x = 5; y = 7;
    z1 =$ h1;             !! Both should resolve x, y from current scope
    z2 =$ h2;             !! Expected: 12 for both

    !! Constantic capture of partially applied brane
    m = [a, b, c]{result = a + b + c};
    a = 1; b = 2;
    m2 = [+a]<m>;           !! m2 has a=1 captured, b,c detached
    m3 = <m2>;            !! Captures remaining: b=2, c from scope
    c = 3;
    w =$ m3;              !! a=1 (from m2), b=2 (from m3), c=3 (current) = 6
}

Test 5: reDetachmentOnBraneReferences.foo
!! Test re-applying detachment to already-evaluated brane references

{
    !! Basic re-detachment
    f = [a, b, c, d]{r = a + b + c + d};
    f2 = [a, b, c, d]f;   !! Re-detaches (behaves same as f)

    r =$ [a=1, b=2, c=3, d=4]f;   !! Evaluates to 10
    r2 =$ [a=1, b=2, c=3, d=5]f2; !! Evaluates to 11

    !! Re-detachment can change which ordinates are detached
    g = [a, b]{sum = a + b};      !! a, b detached
    a = 10;
    g2 = [b]g;            !! Re-detach only b, resolve a from current scope
                          !! g2 now has a=10 captured, only b detached
    b = 5;
    x =$ g2;              !! Should be 15: a=10, b=5

    !! Confirm g still has both ordinates detached
    y =$ [a=1, b=2]g;     !! Should be 3

    !! Re-detachment with explicit values
    h = [p, q, r]{product = p * q * r};
    h2 = [p=2, q=3]h;     !! Bind p=2, q=3, r still detached
    r = 4;
    z =$ h2;              !! Should be 24: 2*3*4

    !! Complex re-detachment chain
    {
        k = [a, b, c, d]{result = a + b + c + d};
        a = 1;
        k2 = [+a]k;       !! k2: a=1 captured, b,c,d detached

        b = 2;
        k3 = [+b]k2;      !! k3: a=1, b=2 captured, c,d detached

        k4 = [c=3]k3;     !! k4: a=1, b=2, c=3, only d detached
        d = 4;
        w =$ k4;          !! Should be 10

        !! Re-detach k4 to free up 'c' again
        k5 = [c]k4;       !! Should alarm - c was already bound to 3
                          !! Or should it create new detachment?
        c = 100;
        w2 =$ k5;         !! If c re-detached: 1+2+100+4=107
                          !! If alarmed: 1+2+3+4=10
    };

    !! Evaluation-time binding doesn't modify original
    m = [x, y]{z = x + y};
    x = 5; y = 6;
    result1 =$ [x=10]m;   !! Provides x=10 at eval: 10+6=16
    result2 =$ m;         !! m unchanged, uses scope: 5+6=11
}

Test 6: complexNestedDetachment.foo
!! Test complex nested detachment scenarios

{
    !! Multi-level nested branes with mixed detachment
    {
        a = 1;
        b = 2;
        outer = [c]{
            inner1 = [d]{
                inner2 = [e]{
                    result = a + b + c + d + e
                };
                result =$ inner2
            };
            result =$ inner1
        };

        !! a=1, b=2 captured at outer definition
        !! c detached in outer
        !! d detached in inner1
        !! e detached in inner2

        c = 10;
        d = 20;
        e = 30;
        x =$ outer;       !! Should be 63: 1+2+10+20+30
    };

    !! Currying chain with P-branes
    {
        curry_test = [a, b, c, d, e]{sum = a + b + c + d + e};

        a = 1;
        step1 = [+a]curry_test;       !! a=1

        b = 2;
        step2 = [+b]step1;            !! a=1, b=2

        c = 3;
        step3 = [+c]step2;            !! a=1, b=2, c=3

        !! Now bind d and e at evaluation
        d = 4;
        e = 5;
        result =$ step3;  !! Should be 15
    };

    !! Mixed binding strategies
    {
        mixer = [p, q, r, s, t]{product = p * q * r * s * t};

        p = 2;
        q = 3;

        !! Use P-brane for p, explicit for q, constantic for rest
        partial = [+p, q=10]mixer;    !! p=2 from scope, q=10 explicit
        r = 4;
        s = 5;
        t = 6;

        captured <=> partial;         !! Constantic: captures r=4, s=5, t=6

        !! Change scope
        r = 100;
        s = 200;
        t = 300;

        y =$ captured;    !! Should use p=2, q=10, r=4, s=5, t=6
                          !! Result: 2*10*4*5*6 = 2400
    };

    !! Detachment with brane factories
    {
        factory = [multiplier]{
            generated = [x]{result = x * multiplier};
            result = generated
        };

        multiplier = 10;  !! Should be detached
        double_it <=> factory;  !! Captures multiplier=10

        x = 5;
        z =$ double_it;   !! double_it is a brane with x detached, multiplier=10
                          !! Result should be 50

        !! Create another factory with different multiplier
        multiplier = 3;
        triple_it <=> factory;

        w =$ triple_it;   !! Should be 15: x=5, multiplier=3
    };

    !! Re-detachment of nested branes
    {
        nested = [a]{
            level2 = [b]{
                level3 = [c]{answer = a + b + c};
                answer =$ level3
            };
            answer =$ level2
        };

        a = 1;
        b = 2;
        c = 3;

        !! Re-detach 'a' with explicit value
        redetached = [a=100]nested;
        result1 =$ redetached;  !! Should be 105: a=100, b=2, c=3

        !! Original still works
        result2 =$ nested;      !! Should be 6: a=1, b=2, c=3
    };
}

Test 7: evaluationTimeBinding.foo
!! Test evaluation-time ordinate binding with =$

{
    !! Basic evaluation-time binding
    f = [a, b, c]{r = a + b + c};
    result1 =$ [a=1, b=2, c=3]f;  !! Binds at evaluation: 1+2+3=6

    !! f is unchanged, still has detached ordinates
    a = 10; b = 20; c = 30;
    result2 =$ f;                  !! Uses current scope: 60

    !! Partial binding at evaluation
    g = [x, y, z]{sum = x + y + z};
    x = 5;
    result3 =$ [y=10, z=15]g;     !! x from scope (5), y=10, z=15: 30

    !! Evaluation-time binding with partially applied brane
    h = [a, b, c, d]{product = a * b * c * d};
    a = 2;
    h2 = [+a]h;                   !! h2 has a=2 captured

    result4 =$ [b=3, c=4, d=5]h2; !! Binds remaining at eval: 2*3*4*5=120

    !! Multiple evaluations with different bindings
    k = [p, q]{diff = p - q};
    r1 =$ [p=10, q=3]k;           !! 7
    r2 =$ [p=100, q=50]k;         !! 50
    r3 =$ [p=5, q=2]k;            !! 3

    !! Evaluation binding overrides scope values
    {
        m = [x, y]{z = x + y};
        x = 1;
        y = 2;

        !! Even though scope has x=1, y=2, evaluation binding takes precedence
        result5 =$ [x=100, y=200]m;  !! 300, not 3
    };

    !! Nested evaluation binding
    {
        outer = [a]{
            inner = [b]{result = a + b};
            result =$ [b=5]inner      !! Binds b=5 at inner evaluation
        };

        result6 =$ [a=10]outer;       !! Binds a=10 at outer evaluation
                                       !! Expected: 10+5=15
    };

    !! Evaluation binding with constantic-captured brane
    {
        n = [i, j, k]{sum = i + j + k};
        i = 1; j = 2; k = 3;
        n2 <=> n;                     !! Captures i=1, j=2, k=3

        !! Can we override captured values at evaluation?
        result7 =$ [i=100]n2;         !! Should this use i=100 or i=1?
                                       !! Expectation: captured values can't be overridden
                                       !! Result should be 1+2+3=6 (ignore i=100)
    };

    !! Evaluation binding with brane that has mix of captured and detached
    {
        p_val = 10;
        q_val = 20;
        mixed = [r_val, s_val]{
            answer = p_val + q_val + r_val + s_val
        };
        !! p_val=10, q_val=20 captured
        !! r_val, s_val detached

        result8 =$ [r_val=30, s_val=40]mixed;  !! 10+20+30+40=100
    };
}

Additional Requirements
Remove old detachment tests that use incorrect blocking semantics:

detachmentBlockingIsApproved.foo
detachmentChainingIsApproved.foo
detachmentFilterChainIsApproved.foo
detachmentNonDistributionIsApproved.foo
pbraneSelectiveBindingIsApproved.foo
[x] Create documentation explaining the free variable semantics, P-brane, constantic brackets, and all binding mechanisms.
    - Integrated into existing docs (NAMES_SEARCHES_N_BOUNDS.md, ECOSYSTEM.md, ADVANCED_FEATURES.md) rather than separate file.

CCW Setup: Update the CCW setup script to use a local SDKMAN stub (since get.sdkman.io is blocked by CCW proxy).

Java Version: Set Maven compiler to Java 21 for CCW compatibility (Java 25 not available in CCW).

Expected Test Results
All 111 tests should pass:

Parser tests
Core Java tests
Core Scala tests
LSP tests
Cross-validation tests (41 matches, 0 mismatches)

There's a mistake in the specifications. The detachment should not occur permannently. It is only effective in two situations

1.) When Foolish code is parsed, during evalution to obtain value, a detachment is applied to a brane, these modifications apply.
```foolish
{
        a = 1;
        f = [b]{r = a+b};
        b = 3;
        x2 =$ f;      !! x2 should take on value of 4, here the detachment brane is nolonger in force and b resolves.
        f2 = [a=3] f  !! f2 is same as f, because when f was assigned the value of that expression, a was already resolved. This situation should be marked as alarming. as it has no effect.
}
```
Let's add lots of tests, nested situations to test that this behavior is as described.

```foolish
{
        f = [a,b,c,d]{r = a+b+c+d};
        a = 3;
        c = 1;
        f2 = [+ a,c]f    !! f2 is curried f with first and third parameter specified already
        f3 = [a=1,c=2]f  !! f2 is detached from parent brane at ordinates a and c, a,c receives the specified values.
}
```

2.) an assignment to a brane function can be re-escape resolution.
{
        f  = [a,b,c,d] {r = a+b+c+d};
        f2 = [a,b,c,d] f;               !! behaves same as f.
        r  =$ [a=1,b=2,c=3,d=4] f       !! evalutes to 10.
        r2 =$ [a=1,b=2,c=3,d=5] f2      !! evalutes correctly to 11.
}

There're two short hands that we're introducing here (syntax and code TODO; documentation DONE)
{
        a=1;b=2;c=3;d=4;
        f  =   [a,b,c,d] {r = a+b+c+d};
        f2 =   <f>;                    !! use a angle bracket to indicate this brane should be ordinated here without additional coordination of detached ordinates.
        f3 <=> f;                      !! Shorthand for the above, the entire RHS is inside `<>` brackets automatically.
        r  =$  f      !! evalutes to 10.
        d=5;
        r2 =$  f2     !! evalutes correctly to 11.
        d=6;
        r3 =$  f3     !! evalutes correctly to 12. 
}
Note the constantic bracket and constantic assignment only affects brane references, they do nothing for freshly parsed foolish branes with detachments.
