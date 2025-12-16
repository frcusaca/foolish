Foolish has a logic system. One can assert anything thusly:

```foolish
{
	|= {1,2}less;
}
```
Stipulates that any time we characterize the RHS as a boolean, it equals true.
The editor may wish to convert the dual ascii statement into single line and the logical symbol for assert. ["⊦"](SYMBOL_TABlE.md).
```foolish
{
	⊦ {1,2}less;
	confirm = {1,2}less; !! Confirms that it is true.
}
```

