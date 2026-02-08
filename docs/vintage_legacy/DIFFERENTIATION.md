In mathematics, the differentiation is the process of calculating the change in the output of a function
based on, sometimes specified changes to the, input. In that domain, we often look for infintesimaly small
changes due to similally miniscue changes the the function's input. In programs, at this moment, we do not
have a continuous domain to work over, however, we can still see that inputs changes outputs. So program
differentiation is the calculation of the changes to the output of a computation, when knowing the changes
to the inputs of said computation. We shall reuse the multivarite derivative symbol, aka the gradient,
nabla, del: ∇(&&#x2207, and \\nabla in latex) to indicate the demand for such a program. To make typing 
easier, we shall also use the symble Δ(&&#x0394, \\Delta0) to mean a difference.

Delta applied to two compatible value produces the change:
```foolish
{
	a=...;
    b=...;
    confirm a =C= b;
    delta = {a,b}Δ; !! Compute the program that changes a to b... somehow.
    bb = a delta;
    confirm b == bb;
}
```

```foolish
{
    !! We have a program, running on input producing an output
	input1 = ...;
    program = [*]{...};
    output1 =$ input1 program;

    !! We can calculate it's gradient
    d0      = program∇

	input2 = ...;
    !! Specify a change function that changes input1 to input2
    change ={input1, input2}Δ;

    !! Apply the gradient of the program we calculated above to the `change` in parameters
    d1      = change d0;

    !! Apply the program gradient, it is a function that takes the output of program due
    !!    to input1 and changes it to the output of program due to input2
    output2_from_diff =$ output1 d1

    !! the results should be the same as actually running it on the output
    output2 =$ input2 program;
    confirm =$  output2 == output2_from_diff

    !! Equivalently on one line:
    confirm = {output1, output2}Δ === {input1, input2}Δprogram∇;
}
```

We could also parameterize the gradient to specify what changes we need to monitor:
```foolish
{
    !! We have a program, running on input producing an output
	input1 = {a=...; b=...;};
	input2 = {a=...; b=...;};
    program = [a,b,c,d]{...};

    rest = {c=...;d=...}
    output1 =$ input1 rest program;
    output2 =$ input2 rest program;

    confirm = {output1, output2}Δ === {input1, input2}Δ program[a,b]∇∇;
    confirm = {output1, output2}Δ === rest {input1, input2}Δ program[a,b]∇∇;
}
```

The blocky looking d 'Ƌ' is alias for Δ the differential.
