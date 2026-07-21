---
concept: stateless
subjects:
  widget:
    note: this subject was declared, but its `states:` list never was
---

# Subject with no states list (fixture)

`widget` reads as a declared subject, but there is no `states:` list, so it
yields an empty state space — **COVER-4**. Coverage would have nothing to check
a `covers: ...` claim against, and would report the subject as undeclared, which
is true of the bundle but misleading about the intent.
