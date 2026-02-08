What is concatenation.
    The semantics is confusing

     { 
       OB={
            a=2;
            r={a=1}{b=1}{c=a};
          };
       result = OB.r.c !! result==1 comes from OB.r.a
     }

     vs.

     {
       OB={
            a=2;
            A={a=1};
            B={b=1};
            C={c=a};   !! It coordinated with OB.a here.
            r=A B C    !! By the time we get here, C's value is coordinated already.
          }
       result = OB.r.c !! result=2 comes from OB.a
     }

     vs.

     {
       OB={
            A={a=1};
            B={b=1};
            C={c=a};
            a=2;
            r=A B C    !! They join before coordination.
          }
       result = OB.r.c !! result=1 comes from OB.A.a
     }

    at joining (at the instantiation of concatenation fir), they're not all constanic.

So it seems ConcatenationBrane needs to do this:
      - Stage A: My state is Initialized, step all references and identifiers until they're constanic. noting else happens.
      - Stage B:
         - cloneConstanic all non-constant statements set state to CHECKED
         - set myself to their parent.
         - enqueue everything non-constant.
      - Now I am Primed
      - continue normal execution.
 
 for phase1 (without relocation).
      - 
