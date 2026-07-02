
## 2026-06-15: SF constanic-clone implementation

- **StayFoolishFir Braning arm**: When expr settles, SF clones the result into ubc_children.
  - If expr has ubc_children (e.g. Operator, Search), clone ubc_children[0]
  - If expr has no ubc_children (leaf like ConstantInt, Nk), clone the expr itself
  - SF's NYES is set to the cloned result's NYES
- **Foolishly flag**: Documented but not yet used — will be used when Scope is implemented
- **Full constanic-clone**: Currently just Rc::clone. Parent rewiring comes later.
- **get_value behavior change**: SF now has ubc_children after settling, so get_value recurses through them (returns inner value, not SF itself)
