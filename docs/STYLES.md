Foolish styles are preferences that are not strictly enforced by the language itself. These should be 
chosen for your project to maximize human convenience.

Textual documents taget one hudnred and eight(108) characters width.

Brane depth marker uses tab characters instead of `^[ ]+`, this reduces storage occupancy. Multi-line
statement alignment may use spaces after the same number of tabs as the first line of the statement.

During development, a feature becomes "lexed", which means it parses into AST, but downstream
implementation yields a `???` when it is encountered. A second step during which the feature becomes
"interpreted" which is when we gain the machinery to correctly handle the the feature in the VM. So
good places to be are "lexed and tested" and "interpreted and tested".
