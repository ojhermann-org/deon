---
concept: broken
subjects:
  thing:
    states:
   - { id: a }
     bad indentation: [unclosed
---

# Broken frontmatter (fixture)

This file opens with a `---` fence, so it is announcing frontmatter — but the
YAML inside does not parse. Whatever state space it meant to declare is
invisible, and without **COVER-4** nothing anywhere says so.
