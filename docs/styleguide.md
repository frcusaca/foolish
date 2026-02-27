# StyleGuide.md — Foolish Language Documentation Style Guide

> Version: 0.4.0-draft · Date: 2026-02-27
> Audience: Human writers and LLM agents that author, review, or lint
> documentation for the Foolish programming language.
> License: To be determined.

---

## References

This guide synthesizes the following six authoritative style guides. Where
guidelines conflict, Section 2 documents each source's position followed by the
Foolish Decision.

| Label | Source | URL | Description |
|-------|--------|-----|-------------|
| Google | Google Developer Documentation Style Guide | https://developers.google.com/style | Comprehensive open-source technical writing guide maintained by Google; the most widely referenced documentation style guide in the industry. |
| Microsoft | Microsoft Writing Style Guide | https://learn.microsoft.com/en-us/style-guide/welcome/ | Enterprise-focused style guide emphasizing scannable, accessible, globally friendly content for developer and end-user documentation. |
| Chicago | The Chicago Manual of Style (18th ed.) | https://www.chicagomanualofstyle.org/home.html | Authoritative American English publishing reference for grammar, punctuation, capitalization, and usage since 1906. |
| Apple | Apple Style Guide | https://help.apple.com/pdf/applestyleguide/en_US/apple-style-guide.pdf | Prescriptive guide for all Apple-branded materials with detailed terminology, capitalization, and product-name conventions. |
| GitLab | GitLab Documentation Style Guide | https://docs.gitlab.com/development/documentation/styleguide/ | Open-source, docs-as-code style guide enforced by automated linting (Vale, markdownlint) with strict rules for translatability. |
| Stylepedia | Stylepedia (Red Hat Technical Writing Style Guide) | https://stylepedia.net/ | Enterprise open-source style guide (CC-BY-SA 3.0) with measurable readability targets and an extensive A–Z usage dictionary. |

Supplementary references: Merriam-Webster Dictionary (spelling); The Chicago
Manual of Style (grammar fallback for Google, Apple, Stylepedia).

---


## Core Principles

### The C's

1. *Clarity* — Every sentence communicates exactly one idea with no ambiguity.
2. *Conciseness* — Remove every word that does not add meaning.
3. *Correctness* — Technical facts, grammar, and terminology are accurate.
4. *Consistency* — One term, one format, one pattern for each concept across
   all documents.
5. *Completeness* — Documentation covers every user-facing behavior;
   undocumented features do not exist.
6. *Communication* — Documentation has a singular purpose of communication,
   this is not abstract expression for expression sake.

### Writing and Formatting Standards

Active voice (with permitted passive patterns defined below), audience
awareness, structural hierarchy, visual aids.

### Open-Source Principles

Plain language and global accessibility, task-oriented content, docs-as-code
workflows, inclusivity, minimum viable documentation.

### LLM Agent Calibration

AI agents as of early 2026 tend to over-emphasize
— bolding topic sentences, key phrases, and anything they consider
significant. In a technical document nearly everything is significant, and
the result is a wall of bold that human readers find exhausting and
patronizing. It is the typographic equivalent of shouting. The conventions
in this guide on emphasis, structure, and formatting exist in part to
counteract this tendency. The standard is simple: write the way a thoughtful
engineer writes for other engineers. The reader is paying attention. Trust
document and sentence structure as as each word to express the full meaning.
Reserve emphasis for the rare cases where it genuinely helps. Take a cue
from Unix man pages, IEEE standards, the Chicago Manual of Style, or any
well-edited technical reference — none of them bold every other phrase, and
all of them communicate complex material clearly through structure and
sentence craft alone.

---


## Latin Abbreviation Conversion Table

Latin abbreviations are not used in Foolish documentation. The following
conversion table is provided for editors who encounter them in source material
or inherited text.

| Latin Abbreviation | Replace With |
|--------------------|--------------|
| e.g. | for example |
| i.e. | that is |
| etc. | (end the list, or open it with "including" or "such as") |
| vs. | versus |
| cf. | compare, see also |
| et al. | and others |
| ibid. | (use a full citation) |
| op. cit. | (use a full citation) |
| viz. | namely |
| N.B. | Note: |
| ca. / circa | approximately |
| fl. | active around (for dates) |

---


## Pronoun Conventions for Non-Personal Entities

Foolish documentation uses *it / they / them / their* for all non-personal
entities, including but not limited to: AI systems, classes, branes, modules,
functions, types, and values. Don't use "he" or "she" for any non-personal
entity. Don't anthropomorphize by assigning personal gendered identity to
language constructs.

Examples:

- "A brane receives a message; it processes the message asynchronously."
- "Branes are immutable once created. They can't be modified after initialization."
- "The class exposes two methods. They're described in the reference below."

For human individuals whose pronouns are unknown, use singular "they."

---


## DM — The "Don't Miss" Marker

Foolish documentation uses a single admonition type: *DM*.

DM stands for "Don't Miss." It's a reading instruction — a flag placed inline
or at the start of a passage to signal that the following content requires
particular attention while reading the document. It's not a call to perform
an action outside the document.

Format:

> DM: The message search operator `$` is case-sensitive.

DM is not used for warnings about system behavior, errors, or operational
risks. Those are explained in plain prose using words like "this operation
can't be undone" or "this setting affects all branes in the cluster." The DM
marker is exclusively a reading-comprehension aid.

Use DM sparingly. If every paragraph carries a DM, none do.

---


## Directory Structure and Documentation Types

Foolish documentation is organized into directories that carry semantic meaning.
The directory a file lives in determines its expected content type, audience,
and tooling treatment.

`howto/` — Literate code. Files in this directory are meant to read as
documentation first and executable code second. They follow a literate
programming style: prose narrates the code, and the code is woven into the
prose. A reader can read a `howto/` document straight through without running
anything. These are the primary reference for humans learning how to use
Foolish. A table of contents is maintained for all files in this directory.

`why/` — Design rationale and motivation. Files in this directory are plain
documentation with minimal code. They explain the reasoning behind Foolish's
design decisions: why a feature exists, what problems it solves, what
alternatives were considered. This is the territory of higher-level reasoning.
A table of contents is maintained for all files in this directory.

`how/` — Engineering documentation. Files in this directory describe how
Foolish itself is built: compiler internals, runtime architecture, contributor
guides, and implementation notes. These are written for contributors and
toolchain authors, not end users.

*Machine-targeted directories* — Directories intended for tools, agents, or
automated pipelines (for example, spec files, grammar definitions, or
agent-facing prompt documents) do not require a maintained table of contents.
Agents are expected to read the full document. Maintaining a TOC for these
files would create unnecessary overhead without reader benefit.

DM: The directory a document lives in is part of its specification.
Moving a document changes its implied contract with readers and tools.

---


## Rendering Target

All markdown files in Foolish documentation are written to be rendered by a
markdown engine and displayed in a browser — typically GitHub-flavored
markdown. The source text is not the final form; the rendered HTML is. This
means HTML entities (such as `&#124;` for a pipe character inside a table
cell, or `&lt;` for a literal angle bracket) are acceptable when markdown
escaping alone cannot produce the correct rendering. Use HTML entities
sparingly and only when needed; prefer markdown's own escaping mechanisms
first.

---


# Section 1: Universal Consensus — Best Practices

---

## 1. Voice and Tone


1.1 Use a tone that is professional, direct, and approachable. Avoid both
stiff formality and excessive casualness.

1.2 Write as if speaking to a knowledgeable peer who is new to the specific
topic. Don't assume expertise beyond what the document's stated audience
requires.

1.3 Prefer active voice in all sentences. Make the actor (subject)
explicit. For the full set of cases where passive voice is permitted and
preferred, see [the Passive Voice Permitted Uses section of the Foolish Style Guide](StyleGuide.md#1a-passive-voice--permitted-uses).

1.4 Don't use language that trivializes complexity. Avoid "simply," "just,"
"easy," "obviously," and similar words that presume the reader's skill level.

1.5 Use exclamation points extremely sparingly — only when the content
genuinely warrants it and the writer is confident the moment is important
enough to justify one. A document may use at most one exclamation point. The
budget is not additive across documents.

1.6 Don't use "please" in procedural instructions. Write direct imperatives
instead.

1.7 Don't use humor, pop-culture references, slang, idioms, or culturally
specific metaphors. These impair translation and global comprehension.

1.8 Maintain a consistent voice across all documents in the project.

---

## 1A. Passive Voice — Permitted Uses

Active voice is the default. The following patterns are situations where
passive voice is not merely acceptable but preferred.

1A.1 Unknown, Irrelevant, or Unimportant Actor

When the agent performing an action is unknown, genuinely irrelevant to the
reader's purpose, or too obvious to name, passive voice eliminates false
precision and noise.

> "The file was corrupted during transmission."
>
> "The configuration is loaded at startup."
>
> "Unused variables are flagged with a warning."

Don't force an actor into subject position when none is meaningfully present.
"Something corrupted the file" is awkward and imprecise.

1A.2 Subject Foregrounding for Emphasis

When a sentence or paragraph significantly changes subject or shifts in mood,
passive voice can promote the semantically important entity to subject position.
Use this when the *what* matters more than the *who did it*, or when the writer
wants the reader to register the subject itself as the primary point.

> "Girls can't be bribed."
>
> "Expression proto-brane members cannot be searched."
>
> "That special message, once sent, can't be recalled."

This pattern is especially effective at the opening of a new section or
immediately after a transition, where it announces the new topic with emphasis.
It's appropriate wherever a sentence or paragraph marks a notable change of
subject or mood and the new subject deserves to land with force.

1A.3 Error Reporting — Foregrounding the Condition, Not the Cause

Error messages and troubleshooting text use passive to foreground what went
wrong rather than who caused it, and to avoid placing the user in subject
position for a mistake.

> "An invalid value was entered for `timeout`."
>
> "The connection was refused."

Don't write "You entered an invalid value" in error contexts. The condition
is what the reader needs to act on; the actor is already implied.

1A.4 Topic-Chain Establishment

When a long passage centers on a single subject — particularly in conceptual
and reference sections that require extended setup before analysis can begin —
passive voice can introduce and promote that subject at the head of the chain.
Once established, the promoted subject carries forward through subsequent active
sentences, creating coherence and reading rhythm.

This pattern is encouraged when the chain is long and a consistent grammatical
subject helps the reader maintain orientation. Uniform syntactic structure
across a passage creates a reading rhythm that makes dense material easier
to follow.

> In the constanic months, the brane's scheduling behavior is constrained by resource limits. It can't allocate new threads. It can't spawn child branes. It must complete queued work before accepting new messages."

The passive opening ("is constrained") places "the brane's scheduling behavior"
in subject position and launches a chain of active sentences that all refer
back to it. Use this pattern freely when the chain is long enough that
restarting from the actor each sentence would fragment the prose.

1A.5 Declaring Global Definitions and Formal Conclusions

When documentation states a formal definition, principle, or design decision
that follows from preceding explanation or derivation, passive voice can mark
the transition from description to declaration. The passive creates a slight
increase in register that signals the conclusion is authoritative rather than
incidental, and that it's distinct from the derivation or argumentation that
preceded it.

> "…and from these properties it is determined that Foolish shall have the
> power of creation."
> "…thus, the callers-are-choosers principle is established as the foundational
> access model for all brane interactions."

This pattern is appropriate for glossary definitions, formal language
specifications, and design rationale sections. Don't overuse it in procedural
or tutorial content.

---

## 2. Grammar and Mechanics


2.1 Use present tense for all descriptions of system behavior, UI
states, and general truths. Avoid future tense ("will") unless describing
scheduled or planned changes.

2.2 Address the reader in second person ("you"). Use imperative mood
for procedural steps (the "you" is implied). Don't use "the user" or "one"
to refer to the reader.

2.3 Use imperative mood for all instructional steps. Begin each step with
an action verb: "Click," "Enter," "Run," "Select."

2.4 Place conditions before instructions, not after. Write "If you want to
export data, click **Export**" rather than "Click **Export** if you want to
export data."

2.5 Sentence length is context-dependent. Introductory sentences can be
short and declarative. Concluding sentences may be longer, gathering the
threads of a section into a final statement. As a general guideline, aim for
sentences in the range of 5 to 24 words. Reconsider rephrasing if a sentence
falls outside that range — though a comma-separated enumeration may legitimately
exceed it, and a single short emphatic sentence is always valid.

2.6 Use "must" for required actions. Use "can" for optional actions. Avoid
"should" (ambiguous between recommendation and requirement). For recommendations,
write "we recommend" or "recommended:".

2.7 Include the conjunction "that" in subordinate clauses for translation
clarity. Write "Verify that the file exists" rather than "Verify the file
exists."

2.8 Avoid noun stacks (chains of three or more nouns used as modifiers).
Rewrite "data migration process status report" as "status report for the data
migration process."

2.9 Don't start a sentence with a numeral. Spell out the number or
restructure the sentence.

2.10 Use parallel grammatical structure for all list items at the same level
and all headings at the same level.

2.11 First-person pronouns ("I," "we," "our") are not used in Foolish
documentation. Address the reader as "you." Refer to the Foolish project,
language, or toolchain by name. When speaking on behalf of the documentation
itself, restructure to avoid a personal subject.

---

## 3. Punctuation


3.1 Use the serial (Oxford) comma before the final conjunction in a list
of three or more items. This is required in all Foolish documentation.

3.2 Use one space between sentences (`sentence_spacing`).

3.3 Follow American convention for punctuation placed adjacent to closing
quotation marks. Periods and commas always go inside the closing quotation
mark, regardless of whether they are logically part of the quotation. Colons
and semicolons always go outside. Question marks and exclamation points follow
the logic of the situation: if the punctuation belongs to the quoted material,
it goes inside; if it belongs to the surrounding sentence, it goes outside.
When both the quotation and the surrounding sentence end with a question mark
or exclamation point, include only one mark — the one inside the closing quote.

3.4 Don't end headings, subheadings, or UI titles with a period.

3.5 Use a comma before a coordinating conjunction (and, but, or, nor, for,
yet, so) that joins two independent clauses.

3.6 Use straight quotation marks and apostrophes in source files. Don't
use typographic ("curly") quotes in documentation source.

3.7 Use hyphens (-) for compound modifiers that precede a noun
("well-known method"). Don't hyphenate after adverbs ending in -ly
("frequently used option").

3.8 Use an em dash (—) for parenthetical insertions and breaks in thought,
with no spaces on either side. For ranges (pages, versions, numbers), use the
word "to" in prose: "versions 3 to 7," "pages 14 to 22." For hyphenated
compound words, use a hyphen. Em dashes are not used for ranges. En dashes
are not used in Foolish documentation.

3.9 Foolish documentation is open to characters from the full Unicode range
where their meaning is unambiguous and they're unlikely to cause rendering or
comprehension difficulty. Mathematical symbols, arrows, and typographic marks
with clear conventional meanings are welcome. Characters whose meaning is
obscure or context-dependent should be accompanied by a definition on first use.

---

## 4. Terminology and Word Choice


4.1 Use plain language. Choose common, everyday words over technical jargon,
formal vocabulary, or bureaucratic language. Define necessary technical terms
on first use.

4.2 Don't use Latin abbreviations in running text. See the
[Latin Abbreviation Conversion Table in the Foolish Style Guide](StyleGuide.md#latin-abbreviation-conversion-table)
for replacements.

4.3 Use one term for one concept consistently. Define the canonical term
and use it everywhere.

4.4 Spell out acronyms and abbreviations on first use in each document.
Place the abbreviation in parentheses immediately after: "Continuous Integration
(CI)." After the first use, use only the abbreviation.

4.5 Avoid "and/or." Rewrite the sentence to clarify the logical relationship.

4.6 Avoid anthropomorphism. Don't attribute human qualities to software.
Use precise verbs: "the system requires," "the function returns."

4.7 Avoid phrasal verbs when a single, precise verb exists. Prefer
"determine" over "figure out," "delete" over "get rid of."

4.8 Use American English spelling as the project standard. Follow
Merriam-Webster for spelling questions.

---

## 5. Formatting


5.1 Use bold for UI element labels visible on screen: button names,
menu items, tab labels, field names, page titles. Match the text and
capitalization displayed in the UI.

5.2 Use inline code font (backticks in Markdown) for all code-related
text appearing in prose: commands, filenames, file extensions, class names,
method names, function names, variable names, parameter names, property names,
attribute names, constant names, data types, environment variables, registry
keys, HTTP methods, HTTP status codes, markup tags, keyboard input the user
types in a terminal or code editor, short error messages, and Boolean values
(`true`, `false`).

5.3 Use italic for the first occurrence of a new term being defined, for
titles of standalone works, and for mathematical variables. Use bold only for
small, easily misread words whose weight matters — "this does **not** apply"
or "the filter is active **only** during BRANING." Bold draws attention to a
word that might otherwise be skimmed past. A document where every third
phrase is bolded teaches the reader to ignore bold entirely.

5.4 Don't use ALL CAPS for emphasis. Reserve all-caps only for acronyms
or constants that are conventionally all-caps in code.

5.5 Use numbered (ordered) lists for sequential procedures. Use
bulleted (unordered) lists for sets of items with no required order. Use
description lists for term-definition pairs.

5.6 Begin each list item with a capital letter. Use consistent end
punctuation: periods after items that are complete sentences; no punctuation
after sentence fragments, single words, or items entirely in code font.

5.7 Introduce every list with a complete introductory sentence or clause
ending in a colon.

5.8 Reserve underlining exclusively for hyperlinks.

5.9 Use `indentation_depth` spaces for indentation in Markdown source
(see Conventions Registry). Don't use tabs.

---

## 6. Document Structure and Organization


6.1 Every page has exactly one level-1 heading (H1). The H1 serves as the
document or page title.

6.2 Use heading levels hierarchically. Don't skip levels. Headings from
H1 through H7 are all permitted in Foolish documentation.

6.3 Blank lines between sections follow the heading depth. After an H1,
leave 4 blank lines before the next content. After an H2, leave 3 blank lines.
After any heading H3 through H7, leave 2 blank lines. Between body paragraphs,
leave 1 blank line. This visual rhythm helps both human readers and parsing
tools identify section boundaries.

6.4 Don't place two headings consecutively without intervening body text.

6.5 Write heading text that functions as a self-contained label. A reader
scanning only headings should understand the document's structure and coverage.

6.6 For task-oriented headings, use a verb phrase in the base form: "Create
an Instance," "Configure Logging." For conceptual headings, use noun phrases:
"Authentication Methods," "System Requirements." All headings follow Title Case
as defined in the
[Title Case — Foolish Definition section of the Foolish Style Guide](StyleGuide.md#title-case--foolish-definition).

6.7 Don't include links in headings. Don't include inline code in headings
unless unavoidable. Don't include numbers that may change, as they break anchor
links.

6.8 Organize content into typed topics, following the directory conventions
defined in the
[Directory Structure and Documentation Types section of the Foolish Style Guide](StyleGuide.md#directory-structure-and-documentation-types):
*Concept* (explains what something is), *Task* (explains how to do
something), *Reference* (provides lookup data), or *Troubleshooting*
(diagnoses and resolves errors).

6.9 Place the most important information first — both within documents
(inverted pyramid) and within paragraphs (topic sentence first).

6.10 Keep paragraphs in the range of 5 to 10 sentences. Consider
restructuring if a paragraph falls outside that range — either by splitting
an overlong paragraph into two or by merging an underdeveloped one with
adjacent content.

6.11 A table of contents is maintained for all files in `howto/` and
`why/` directories. Files in machine-targeted directories don't require a
table of contents. For the full policy, see the
[Directory Structure and Documentation Types section of the Foolish Style Guide](StyleGuide.md#directory-structure-and-documentation-types).

6.12 The following table lists every structural element used in Foolish
documentation, with its markdown trigger. These are confirmed to render
correctly in GitHub-flavored markdown and in most documentation viewers.
Use these structural tools — headings, lists, tables — to organize ideas
rather than relying on bold or italic to do the work of guiding the reader.

| Structure | Markdown | When to use |
|-----------|----------|-------------|
| Chapter heading | `# Title` | One per file, the document title |
| Section | `## Section` | Major topic divisions |
| Subsection | `### Subsection` | Divisions within a section |
| Sub-subsection | `#### Sub-subsection` | Addressable parts within a subsection; use instead of bolding a sentence inline |
| Bulleted list | `- item` | Unordered sets of related items, options, properties |
| Nested sublist | `  - sub-item` | Grouping details under a parent bullet (indent 2 spaces) |
| Numbered list | `1. step` | Sequential steps, ordered procedures |
| Table | pipe-delimited rows | Parallel structure, comparisons, reference data |
| Blockquote | `> text` | Callouts, quoted material, invariants, formal statements |
| Fenced code block | `` ``` `` + lang tag | Code examples, output samples, ASCII diagrams |
| Inline code | `` ` `` around text | Method names, file paths, short expressions in running prose |
| Horizontal rule | `---` | Section separators (use sparingly; headings usually suffice) |
| Italic | `*word*` | Term definitions on first use |
| Bold | `**word**` | Small, easily misread words only |

---

## 7. Visual Aids and Media


7.1 Provide alt text for every image. Alt text must convey the same
information the image communicates.

7.2 Always introduce an image with a complete sentence before the image
appears. Don't let images carry information absent from the text.

7.3 Use screenshots to supplement text, never to replace it.

7.4 Prefer text-based diagram formats (Mermaid, PlantUML) over binary
image files when the toolchain supports them.

7.5 Compress images to a reasonable file size. Use PNG for screenshots
with text; use SVG for diagrams and icons.

7.6 Use sentence case for figure captions and diagram labels.

7.7 Don't rely solely on color to convey meaning. Supplement with labels,
patterns, or shapes.

---

## 8. Accessibility and Inclusivity


8.1 Use gender-neutral language. Don't use "he," "him," "his," "she,"
or "her" as generic pronouns. Use singular "they/them/their," second person
"you," plural nouns, or rephrase to avoid pronouns.

8.2 Use inclusive terminology:

- "master/slave" → "primary/replica" or "main/secondary"
- "blacklist/whitelist" → "denylist/allowlist"
- "sanity check" → "validity check" or "confidence check"
- "dummy" → "placeholder" or "sample"
- "kill" → "stop," "terminate," or "end"
- "grandfathered" → "legacy" or "preexisting"

8.3 Don't use ableist language. Use precise, neutral alternatives.

8.4 Use people-first language by default, and respect community preferences
for identity-first language where applicable.

8.5 Use diverse and culturally varied names in examples and sample data.
Use `example.com` for example email addresses and domains.

8.6 Write for a global audience. Assume many readers are non-native English
speakers and that some are learning both Foolish and English simultaneously.
Foolish documentation makes no assumptions about the social, economic, or
educational background of its readers. The goal is clarity for anyone who
wants to learn.

8.7 Use "before" and "after" rather than "above" and "below" when referring
to content position within a document. "Before" means content that appears at
a smaller index position in the current section or in an earlier section at
the next heading level up. "After" means content at a larger index position
or in a later section. This convention carries a temporal meaning — the
sequence in which a reader encounters content — and also has a precise
offset meaning for tools that process documents as ordered data.

8.8 Use semantic heading levels (H1–H7) rather than visual formatting to
communicate document hierarchy. Screen readers rely on semantic markup.

---

## 9. Code and Technical Notation


9.1 Use inline code font (single backticks) for all code elements
appearing within prose text. See guideline 5.2 for the comprehensive list.

9.2 Use code blocks (triple backticks) for multi-line code samples,
command-line examples, configuration files, and terminal output. Any code
that is significant or longer than 5 tokens belongs in a fenced code block,
not inline backticks. Always specify a language identifier so that syntax
highlighting and tooling work correctly. Use `plaintext` when no language
applies. Common language tags in Foolish documentation: `foolish`, `java`,
`scala`, `python`, `bash`. For conceptual pseudocode or ASCII diagrams, use
a bare three-backtick fence with no language tag.

9.3 Insert a blank line before and after every code block in Markdown source.

9.4 Use `placeholder_format` for placeholder values the reader must replace
(see Conventions Registry).

9.5 Don't put UI element names in code font. UI elements use bold.

9.6 When referencing a code element in prose, add a qualifying noun for
clarity: "the `config.yaml` file," "the `getUserName()` method."

9.7 Follow the capitalization and naming conventions of the programming
language when writing code elements in text.

9.8 For command-line examples, show the command without a shell prompt
character unless the guide explicitly configures a prompt convention to
distinguish input from output.

9.9 Coding style standards — including naming conventions, identifier
capitalization, and formatting of Foolish-specific constructs — are defined
in `DOC_AGENTS.md`. When `DOC_AGENTS.md` and this guide conflict on a
code-formatting question, `DOC_AGENTS.md` takes precedence.

---

## 10. Linking and Cross-References


10.1 Use descriptive link text that makes sense when read out of context
and that would, if encountered again in the same paragraph, distinctly and fully
evoke the memory of reading the linked content. Link text is not a label — it
is a condensed description of the destination's substance.

Correct: [the passive voice permitted-uses section of the Foolish Style Guide](StyleGuide.md#1a-passive-voice--permitted-uses)

Incorrect: "click here," "this page," "learn more"

10.2 Every cross-reference must include a call to action — a phrase that
tells the reader why they should follow the link. For the full detail, read
[the Cross-References and Linking section of the Foolish Style Guide](StyleGuide.md#10-linking-and-cross-references).
* Format for backward references: "For [purpose], review [descriptive link text](url)."
* Format for forward references: "For [purpose], read [descriptive link text](url)."

purose is often "complete detail", but can be "cross tabulated table", "explanation of..."

10.3 Don't use raw URLs as link text. Embed the URL behind descriptive text.

10.4 Don't force links to open in a new tab or window.

10.5 Use relative file paths for links within the same repository or
documentation set. Use absolute URLs for external links.

10.6 Minimize external links. When linking externally, prefer canonical,
stable URLs.

10.7 Don't duplicate the same link target on a single page. Link only on
the first or most prominent mention.

10.8 Don't place links inside headings.

---

## Conventions Registry

Configurable parameters are listed below. Parameters with an assigned Foolish
value are marked [DECIDED]. Parameters deferred to `DOC_AGENTS.md` are
marked [DEFERRED]. Undecided parameters remain open for resolution.

| Parameter | Description | Google | Microsoft | Chicago | Apple | GitLab | Stylepedia | Common Range | Foolish Value |
|-----------|-------------|--------|-----------|---------|-------|--------|------------|--------------|---------------|
| `quotation_punctuation` | Punctuation placement relative to closing quotation marks | American | American | American | American | American | American | American | [DECIDED] American: periods and commas inside; colons and semicolons outside; ?! follow logic of the situation |
| `serial_comma` | Serial comma in lists of 3+ | Required | Required | Required | Required | Required | Required | Required | [DECIDED] Required |
| `locale` | Language/region variant | US English | US English | US English | US English | US English | US English | US English | [DECIDED] US English (Merriam-Webster) |
| `sentence_spacing` | Spaces between sentences | 1 | 1 | 1 | 1 | 1 | 1 | 1 | [DECIDED] 1 |
| `heading_capitalization` | Title case or sentence case | Sentence | Sentence | Title | Title | Sentence | Title | Both common | [DECIDED] Title Case — see full definition below |
| `latin_abbreviations` | Permitted in text | Forbidden | Avoid | Footnotes only | Forbidden | Forbidden | Forbidden | Forbidden in body | [DECIDED] Forbidden; see conversion table |
| `admonition_types` | Recognized admonition markers | Note/Caution/Warning | Tip/Note | N/A | Not specified | Note/Warning/Flag | Note/Important/Warning | Varies | [DECIDED] DM only |
| `pronoun_non_personal` | Pronoun for classes, branes, AI | Varies | Varies | Varies | Varies | Varies | Varies | it/they | [DECIDED] it / they / them / their |
| `link_text_policy` | Link text convention | Descriptive | Descriptive | N/A | Implied | Descriptive | Descriptive | Descriptive | [DECIDED] Full evocative description; cross-references require a call to action |
| `passive_voice_policy` | Passive voice use | Contextual | Contextual | Balanced | Restrictive | Contextual | Contextual | Active default | [DECIDED] Active default; 5 permitted patterns defined |
| `first_person_pronouns` | Use of I, we, our | Avoid | Limited | No prohibition | Avoid | Avoid | Limited | Avoid | [DECIDED] Forbidden |
| `exclamation_points` | Exclamation point use | Avoid | Use sparingly | Permitted | Implied restraint | Not addressed | Avoid | Avoid | [DECIDED] At most one per document; use only when genuinely important |
| `em_dash_policy` | Em dash use in prose | Used, no spaces | Used, no spaces | Used, no spaces | Used, no spaces | Forbidden | Defers to Chicago | Used or forbidden | [DECIDED] Used, no spaces, for parenthetical and breaks in thought |
| `en_dash_policy` | En dash for ranges | Forbidden | Used | Used | Used | Forbidden | Defers to Chicago | Used or forbidden | [DECIDED] Not used; "to" for ranges in prose |
| `range_notation` | How to express ranges | Hyphen or "to" | En dash | En dash | En dash | Not addressed | Defers | Various | [DECIDED] "to" in prose ("versions 3 to 7") |
| `heading_depth_limit` | Max heading level permitted | H1–H3+ | H4 practical | H3 rec. | N/A | H5 | ~H4 | H3–H5 | [DECIDED] H1 through H7 are all permitted |
| `toc_policy` | Table of contents required | Not prescribed | Longer docs | Front matter | Not addressed | Sidebar auto | Book-level | Varies | [DECIDED] Required in howto/ and why/; omitted in machine-targeted directories |
| `contraction_policy` | Contractions permitted | Encouraged | Encouraged | Permitted | Encouraged | With exceptions | Forbidden | Permitted or forbidden | [DECIDED] Permitted and encouraged |
| `numbers_threshold` | Threshold: spell out vs. numerals | 0–9 spelled | 0–9 spelled | 0–100 spelled | 0–9 spelled | 0–9 spelled | 0–9 spelled | 0–9 / 10+ | [DECIDED] Use numerals throughout: 1, 2, 3, 10, 20. No spell-out threshold. |
| `unicode_policy` | Use of non-ASCII characters | Not addressed | Not addressed | Not addressed | Not addressed | Not addressed | Not addressed | Restrict to ASCII | [DECIDED] Full Unicode permitted where meaning is unambiguous and rendering is reliable |
| `directional_reference` | "above/below" vs. other terms | Avoid | Avoid | Not addressed | Conditional | Implied avoidance | Implied avoidance | Avoid | [DECIDED] Use "before" and "after"; carries temporal and index-offset meaning |
| `whitespace_after_headings` | Blank lines after headings in source | Not specified | Not specified | N/A | Not specified | 1 blank line | Not specified | 1 | [DECIDED] H1: 4 blank lines; H2: 3 blank lines; H3–H7: 2 blank lines; between paragraphs: 1 blank line |
| `sentence_length_limit` | Recommended sentence length range | 26 max | Qualitative | N/A | N/A | N/A | 30 max | 20–30 | [DECIDED] 5 to 24 words; reconsider outside this range; enumerations may exceed upper bound |
| `paragraph_length_limit` | Recommended paragraph length range | 5–6 sentences | 3–7 lines | N/A | N/A | N/A | N/A | 3–7 | [DECIDED] 5 to 10 sentences; consider restructuring outside this range |
| `docs_as_code_workflow` | Documentation authoring workflow | Advocates | Not mandated | N/A | Internal systems | Git + CI/CD lint | GitHub/AsciiDoc | Varies | [DECIDED] Semantically segregated by directory; howto/ is literate code; why/ is prose; how/ is engineering docs |
| `semicolon_policy` | Semicolons in prose | Permitted | Permitted | Permitted | Permitted | Forbidden | Limit | Permitted | [DECIDED] Permitted; if two or three appear in succession, consider separate sentences instead |
| `topic_type_taxonomy` | Formal topic-type taxonomy | Informal | Informal | N/A | Informal | CTRT explicit | Book-level | Varies | [DECIDED] Semantically segregated by directory; why/ for motivation, howto/ for procedure, how/ for engineering |
| `cross_reference_phrasing` | Standard cross-reference phrasing | "see [title]" | "See also" | Not addressed | Not specified | "see [title]" | "refer to" | Varies | [DECIDED] "For [purpose], read [descriptive link](url)." |
| `readability_target` | Target readability score | Qualitative | Qualitative | N/A | Qualitative | Qualitative | Flesch-Kincaid 60–70 | FK 60–70 | [DECIDED] No readability grade level target — see decision below |
| `version_notation` | How to express version references | "later versions" | Not specified | Not applicable | Not specified | History shortcodes | Not specified | Various | [DECIDED] Follow Google convention: "version X or later," "later versions" |
| `code_style` | Identifier naming, capitalization conventions | N/A | N/A | N/A | N/A | N/A | N/A | Per language | [DEFERRED] Imported from DOC_AGENTS.md |
| `bold_emphasis` | Bold use beyond UI labels | Restricted | Restricted | Not addressed | Restricted | Restricted | Restricted | Restricted | [DECIDED] Bold for small, easily misread words only; see guideline 5.3 |
| `feature_name_capitalization` | Capitalize feature names as proper nouns | Conservative | Conservative | Per usage | Aggressive | Lowercase | Per vendor | Varies | [DEFERRED] Imported from DOC_AGENTS.md |
| `indentation_depth` | Spaces per indent level (Markdown) | N/A | N/A | N/A | N/A | Spaces, no tabs | N/A | 2 or 4 | _(to be decided)_ |
| `line_length_limit` | Max chars per Markdown source line | N/A | N/A | N/A | N/A | ~100 | N/A | 80–120 | _(to be decided)_ |
| `placeholder_format` | Format for code placeholder values | `UPPER_SNAKE_CASE` | Italic or `<angle>` | N/A | Per format | `<angle_brackets>` | Monospace | `<PLACEHOLDER>` | _(to be decided)_ |
| `list_marker` | Unordered list character in Markdown | N/A | N/A | N/A | N/A | Dash `-` | N/A | `-` or `*` | _(to be decided)_ |
| `ordered_list_numbering` | Numbering scheme in Markdown source | N/A | N/A | N/A | N/A | All `1.` | N/A | `1.` or sequential | _(to be decided)_ |

---

## Title Case — Foolish Definition

All headings and document titles in Foolish documentation use Title Case,
following the convention shared by Chicago Manual of Style and Apple Style Guide.

### Capitalize

- The first word of the heading, always.
- The last word of the heading, always.
- All nouns, pronouns, verbs (including infinitive "to"), adjectives, and adverbs.
- Prepositions of five or more letters (for example: "About," "Between,"
  "Through," "Without," "Against").
- Subordinating conjunctions (for example: "Because," "Although," "That,"
  "When," "While").

### Do Not Capitalize

- Articles: "a," "an," "the" (unless first or last word).
- Coordinating conjunctions: "and," "but," "or," "nor," "for," "yet," "so"
  (unless first or last word).
- Short prepositions of four or fewer letters: "at," "by," "for," "from,"
  "in," "into," "of," "off," "on," "onto," "out," "to," "up," "with"
  (unless first or last word).
- The word "to" in infinitives: "How to Configure a Brane" — "to" is lowercase.

### Special Cases

- Inline code identifiers within headings retain their code capitalization.
  Don't re-case identifiers to satisfy title case rules.
- Hyphenated compounds: capitalize both elements if each would be capitalized
  standing alone ("Self-Referential Pattern"), but lowercase the second element
  if it is an article, preposition, or coordinating conjunction
  ("End-to-End Encryption").

### Examples

- Correct: "Configuring a Brane for Message Passing"
- Correct: "How to Set Up the Development Environment"
- Correct: "The Callers-Are-Choosers Access Model"
- Incorrect: "Configuring A Brane For Message Passing" (articles and short
  prepositions capitalized)
- Incorrect: "configuring a brane for message passing" (sentence case)

Figure captions, table captions, diagram labels, and list items use sentence
case, not title case.

---


# Section 2: Points of Disagreement or Genuine Variation

Undecided items appear first. Items with Foolish Decisions appear at the end
of this section.

---

## Undecided


### Monospace Formatting: Boundary of Inline Code

What's Contested: The precise boundary of what content receives inline code
formatting versus bold, italic, or plain text.

Positions:
- Google: Code font for filenames, class names, method names, HTTP status
  codes, console output, placeholders, attribute names and values, data types,
  environment variables, DNS record types, API names, command-line utilities,
  user input. UI elements get bold.
- Microsoft: Code font for attributes, classes, code samples, command-line
  commands and options, constants, data types, environment variables, event
  names, fields, functions, keywords, macros, markup tags, methods, operators,
  parameters, properties, statements, structures, values, and variables.
  Exceptions: database names (bold), error messages (quotation marks), math
  variables (italic), new terms (italic).
- Chicago: Not applicable.
- Apple: Code font for code elements, commands, parameter names, code
  values, filenames, filename extensions.
- GitLab: Backticks for text users enter in a UI, short inputs and outputs,
  filenames, configuration parameters, keywords, short error messages, API and
  HTTP methods, HTTP status codes, HTML elements. UI labels get bold.
- Stylepedia: Monospace for commands, filenames, directory paths, code
  elements, package names. Don't include code elements in headings.

Foolish Decision: _(to be decided)_

---


## Decided


### Quotation Mark Punctuation Placement

What's Contested: Whether terminal punctuation goes inside or outside
closing quotation marks, and whether the rule varies by punctuation type.

Positions:
- Google: American convention (commas and periods inside).
- Microsoft: American convention.
- Chicago: American convention, with detailed treatment of edge cases for
  ?! based on sentence logic.
- Apple: American convention.
- GitLab: American convention.
- Stylepedia: American convention.

Foolish Decision: Foolish documentation follows American punctuation
conventions for all quotation mark interactions. Periods and commas always
go inside the closing quotation mark, regardless of whether they are logically
part of the quotation. Colons and semicolons always go outside. Question marks
and exclamation points follow the logic of the situation: if the punctuation
belongs to the quoted material, it goes inside; if it belongs to the surrounding
sentence, it goes outside. When both the quotation and the surrounding sentence
end with a question mark or exclamation point, include only one mark — the one
inside the closing quote. For the guideline in full, read
[guideline 3.3 of the Foolish Style Guide Punctuation section](StyleGuide.md#3-punctuation).

---


### Oxford / Serial Comma

What's Contested: Whether to use a comma before the final conjunction in a
list of three or more items.

Positions:
- Google: Required.
- Microsoft: Required. Listed as a Top 10 tip.
- Chicago: Strongly recommended since 1906.
- Apple: Required. Follows Chicago Manual of Style.
- GitLab: Required. Enforced via automated Vale rule.
- Stylepedia: Required. Used consistently in all examples.

Foolish Decision: Required in all Foolish documentation. Every list of three
or more items uses a serial comma before the final conjunction. This is not
configurable. Linting tools must enforce this rule.

---


### US English vs. UK English

What's Contested: Whether to use American English or British English
conventions for spelling and punctuation.

Positions:
- Google: US English. Follows Merriam-Webster.
- Microsoft: US English. References The American Heritage Dictionary and
  Merriam-Webster.
- Chicago: US English. Follows Merriam-Webster.
- Apple: US English.
- GitLab: US English. Enforced via Vale rule.
- Stylepedia: US English (implicit).

Foolish Decision: American English in all Foolish documentation.
Merriam-Webster is the dictionary of record for spelling disputes. When
Merriam-Webster lists multiple acceptable spellings, use the first entry. All
punctuation follows American conventions: commas and periods are placed inside
closing quotation marks; colons and semicolons are placed outside.

---


### Heading Capitalization: Title Case vs. Sentence Case

What's Contested: Whether headings and titles use title case or sentence case.

Positions:
- Google: Sentence case.
- Microsoft: Sentence case.
- Chicago: Title case (headline style / title case in the 18th edition).
- Apple: Title case as primary heading style.
- GitLab: Sentence case. Enforced in linting.
- Stylepedia: Title case for chapter and section headings.

Foolish Decision: Title Case for all headings and document titles, following
the conventions of Chicago Manual of Style and Apple Style Guide. Specifically:
capitalize the first and last words always; capitalize all nouns, pronouns, verbs,
adjectives, and adverbs; capitalize prepositions of five or more letters and all
subordinating conjunctions; don't capitalize articles (a, an, the), coordinating
conjunctions (and, but, or, nor, for, yet, so), or short prepositions of four or
fewer letters, unless they are the first or last word. Infinitive "to" is
lowercase. For the full definition with examples, read the
[Title Case — Foolish Definition section of the Foolish Style Guide](StyleGuide.md#title-case--foolish-definition).
Figure captions, table captions, diagram labels, and list items use sentence case.

---


### Latin Abbreviations

What's Contested: Whether Latin abbreviations may appear in documentation text.

Positions:
- Google: Avoid.
- Microsoft: Implicitly avoid.
- Chicago: Permitted in parentheses, footnotes, endnotes, and tables.
- Apple: Avoid.
- GitLab: Explicitly prohibited. Enforced via Vale rule.
- Stylepedia: Explicitly forbidden.

Foolish Decision: Forbidden in all Foolish documentation. No Latin
abbreviations appear in body text, headings, captions, code comments, or any
other documentation element. Authors must replace them using the
[Latin Abbreviation Conversion Table in the Foolish Style Guide](StyleGuide.md#latin-abbreviation-conversion-table).
Linting tools must flag Latin abbreviations as errors.

---


### Admonition Types

What's Contested: Which admonition types exist, what they are called, and
how they are formatted.

Positions:
- Google: Note, Caution, Warning, Success.
- Microsoft: Tip, Note, See also.
- Chicago: Not applicable.
- Apple: Not explicitly addressed.
- GitLab: Note, Warning, Flag, Disclaimer.
- Stylepedia: Note, Important, Warning, Danger.

Foolish Decision: Foolish documentation uses a single admonition marker:
*DM* ("Don't Miss"). DM is a reading-comprehension instruction — a flag that
signals content requires particular attention while reading the document. It's
not a warning about system behavior, a deprecation notice, or a tip about an
action outside the document. Operational risks, irreversible operations, and
deprecations are explained in plain prose. For the full definition and usage
rules, read the
[DM — The "Don't Miss" Marker section of the Foolish Style Guide](StyleGuide.md#dm--the-dont-miss-marker).

---


### Pronoun for Non-Personal Entities

What's Contested: Which pronoun to use for software entities such as
classes, branes, AI systems, and functions.

Positions: All six sources vary or are silent on this specific question for
language-level constructs.

Foolish Decision: Foolish documentation uses *it / they / them / their*
for all non-personal entities, including AI systems, classes, branes, modules,
functions, types, and values. Don't use "he" or "she" for any non-personal
entity. For the full convention with examples, read the
[Pronoun Conventions for Non-Personal Entities section of the Foolish Style Guide](StyleGuide.md#pronoun-conventions-for-non-personal-entities).

---


### Link Text Policy

What's Contested: How link text should be written; whether "click here" or
raw URLs are acceptable.

Positions:
- Google: Descriptive. Prohibits "click here," "this document," and raw URLs.
- Microsoft: Descriptive. "Write brief but meaningful link text."
- Chicago: Not specifically addressed for technical documentation.
- Apple: Implied descriptive from general writing principles.
- GitLab: Descriptive. Standard pattern enforced. Never "here" or "this page."
- Stylepedia: Descriptive cross-references with evaluation tests for clarity.

Foolish Decision: Link text in Foolish documentation must be a full,
evocative description of the linked content — specific enough that if
encountered again in the same paragraph, the phrase would distinctly recall
the linked material without the reader needing to follow the link again.
Every cross-reference must include a call to action telling the reader why to
follow it. Format: "For [purpose], read [descriptive link text](url)."
Raw URLs, "click here," "this page," "learn more," and "read more" are forbidden.
For the full policy and examples, read
[guideline 10.1 and 10.2 of the Foolish Style Guide Linking and Cross-References section](StyleGuide.md#10-linking-and-cross-references).

---


### Passive Voice Policy

What's Contested: Whether passive voice should be categorically avoided or
used selectively.

Positions:
- Google: Contextual use. Active preferred; passive acceptable when actor
  is irrelevant or unknown.
- Microsoft: Contextual use. Exceptions for awkward active constructions and
  error messages.
- Chicago: Balanced. Recommends regular use of passive when appropriate.
- Apple: Strong preference for active. Restructure sentences to make the
  reader the subject.
- GitLab: Contextual use. Active preferred; exceptions for awkward
  system-as-actor cases.
- Stylepedia: Contextual use. Passive acceptable for release notes and
  foregrounding key concepts.

Foolish Decision: Active voice is the default for all Foolish documentation.
Passive voice is not merely permitted but preferred in five specific situations:
when the actor is unknown, irrelevant, or unimportant; when subject foregrounding
is needed to emphasize a new or contrasting topic at a transition; when reporting
errors, to foreground the condition rather than the cause; when establishing a
topic chain where consistent subject position creates reading rhythm; and when
declaring formal global definitions or design conclusions in a register distinct
from the preceding derivation. For the full definitions, rationale, and examples,
read the
[Passive Voice — Permitted Uses section of the Foolish Style Guide](StyleGuide.md#1a-passive-voice--permitted-uses).

---


### Sentence Length Recommendations

What's Contested: Whether a specific word-count limit per sentence should be
enforced, and what that limit is.

Positions:
- Google: Fewer than 26 words per sentence.
- Microsoft: Qualitative: "Write short, simple sentences." No specific word count.
- Chicago: Not addressed.
- Apple: Not explicitly addressed.
- GitLab: Not explicitly addressed with a number.
- Stylepedia: Don't exceed 30 words in a sentence.

Foolish Decision: Sentence length is context-dependent. Introductory
sentences can be short and declarative. Concluding sentences may be longer,
gathering the threads of a section. The preferred range is 5 to 24 words.
Reconsider rephrasing if a sentence falls outside that range. A
comma-separated enumeration may legitimately exceed the upper bound, and a
single emphatic short sentence is always valid.

---


### Paragraph Length Recommendations

What's Contested: Whether a specific sentence count per paragraph should
be enforced.

Positions:
- Google: "A paragraph longer than 5 or 6 sentences is often an indication
  that the paragraph is trying to convey too much information."
- Microsoft: "Three to seven lines is about the right length for a paragraph."
- Chicago: Not addressed.
- Apple: Not explicitly addressed.
- GitLab: Not explicitly addressed with a number.
- Stylepedia: Not explicitly addressed.

Foolish Decision: The preferred paragraph length is 5 to 10 sentences.
Consider restructuring if a paragraph falls outside that range — either by
splitting an overlong paragraph into two focused ones or by merging an
underdeveloped paragraph with adjacent content.

---


### Heading Depth Limits

What's Contested: The maximum number of heading levels permitted in a
single document.

Positions:
- Google: No explicit maximum. Practical usage: H1–H3 or H1–H4.
- Microsoft: Practical maximum of 4 levels. No hard cap.
- Chicago: No more than 3 levels recommended; Turabian extends to 5.
- Apple: No explicit limit stated.
- GitLab: Avoid heading levels greater than H5.
- Stylepedia: No explicit numeric limit; approximately 4 levels in practice.

Foolish Decision: Heading levels H1 through H7 are all permitted in Foolish
documentation. There is no enforced maximum. The constraint on heading depth
is structural coherence, not a numeric cap. Don't add heading levels solely to
create visual hierarchy; add them when the content genuinely branches into a
distinct sub-topic.

---


### Table of Contents: Required vs. Optional

What's Contested: Whether every document must include a table of contents,
and whether it should be auto-generated or manually curated.

Positions:
- Google: Not explicitly prescribed for per-page TOC.
- Microsoft: Recommended for longer documents.
- Chicago: Rules for book-level TOC in front matter; no prescription for
  general documents.
- Apple: Not explicitly addressed.
- GitLab: Auto-generated sidebar navigation for headings up to H4. Tutorials
  use a manually curated ordered list at the top.
- Stylepedia: Book-level TOC follows formal conventions; no universal rule.

Foolish Decision: A table of contents is maintained for all files in
`howto/` and `why/` directories, where documents are primarily human-targeted
and readers benefit from navigational structure. Files in machine-targeted
directories — where tools and agents are expected to read the entire document —
don't require a table of contents, and maintaining one would create unnecessary
overhead without reader benefit. For the directory semantics that govern this
rule, read the
[Directory Structure and Documentation Types section of the Foolish Style Guide](StyleGuide.md#directory-structure-and-documentation-types).

---


### Use of Contractions

What's Contested: Whether contractions are appropriate in technical
documentation.

Positions:
- Google: Encouraged. Common two-word contractions recommended.
- Microsoft: Encouraged. Listed under "Project friendliness."
- Chicago: Permitted, context-dependent.
- Apple: Encouraged. The guide uses contractions extensively.
- GitLab: Encouraged with exceptions (not in reference docs or error messages).
- Stylepedia: Forbidden. Rationale: informal register and translation difficulty.

Foolish Decision: Contractions are permitted and encouraged throughout
Foolish documentation. They make prose more natural and reduce the cognitive
distance between the writer and the reader. Don't avoid contractions out of
a misplaced sense of formality.

---


### Em Dash and En Dash Usage

What's Contested: Whether to use em dashes and en dashes at all, and if
so, whether spaces surround them.

Positions:
- Google: Em dash (—) used for breaks in thought, no spaces. En dash:
  forbidden; use a hyphen or the word "to" instead.
- Microsoft: Em dash (—) for parenthetical phrases, no spaces. En dash (–)
  for ranges and compound adjectives with open elements.
- Chicago: Em dash (—) for parenthetical insertions, no spaces. En dash (–)
  for ranges and open-element compounds.
- Apple: Em dash per Chicago, no spaces. En dash for ranges and
  multi-word compound adjectives.
- GitLab: Don't use em dashes or en dashes.
- Stylepedia: Defers to Chicago.

Foolish Decision: The em dash (—) is used in Foolish documentation for
parenthetical insertions and breaks in thought, with no spaces on either side,
following Google's convention. En dashes are not used. For ranges in prose,
use the word "to": "versions 3 to 7," "pages 14 to 22." For hyphenated
compound words, use a hyphen.

---


### Numbers: Spell Out vs. Numerals

What's Contested: The threshold at which writers switch from spelled-out
numbers to numerals.

Positions:
- Google: Spell out zero through nine; numerals for 10 and above.
- Microsoft: Spell out zero through nine; numerals for 10 and above.
- Chicago: General rule: spell out zero through one hundred.
- Apple: Spell out zero through nine; numerals for 10 and above.
- GitLab: Spell out zero through nine; numerals for 10 and above.
- Stylepedia: Spell out numbers below ten; numerals for 10 and above.

Foolish Decision: Foolish documentation uses numerals throughout, without
a spell-out threshold. Write 1, 2, 3, 10, 20 — not "one," "two," "three."
Numbers in prose are numerals. The one exception is a sentence-initial
position: don't begin a sentence with a numeral; restructure the sentence
instead.

---


### Readability Grade Level Targets

What's Contested: Whether plain language should be measured by a specific
readability metric, and what the target score or grade level should be.

Positions:
- Google: No specific metric. Qualitative guidance only.
- Microsoft: No specific metric. "Simpler is better."
- Chicago: Not in scope.
- Apple: No specific metric.
- GitLab: No specific metric.
- Stylepedia: Flesch-Kincaid reading ease score of 60–70. Gunning Fog
  index of 9–12. Target: 8th-grade comprehension level.

Foolish Decision: Foolish documentation deliberately does not adopt a
Flesch-Kincaid or American grade-level readability target. American reading
grade level tests—including the F&P—were calibrated
on a corpus of intentionally simplified writing, and the grade levels they
produce reflect social and economic biases in American society rather than
universal clarity. Writing toward a specific American literacy score would mean writing
toward that corpus of oversimplified English. The higher the target grade
level, the more abstruse and obscurantist the writing becomes; the lower
the target level, the more it strips language of precision.

Foolish documentation aims to be understandable by anyone who wants to learn —
both Foolish and English. Writing difficulty should vary with importance and
access frequency. Material that is critical and frequently referenced should
be written as clearly as possible. More advanced material may use more precise
or technically dense language where precision genuinely requires it. The
documentation corpus should be self-sufficient, providing the user with all
necessary information to understand the language used throughout. As a rough
calibration, the 800-level vocabulary of the 2026 SAT is a reasonable starting
point.

---


### Version Notation

What's Contested: How to express version references and feature availability.

Positions:
- Google: "version X or later," "later versions." Always use numerals.
- Microsoft: Not specified.
- Chicago: Not applicable.
- Apple: Not explicitly addressed.
- GitLab: History shortcodes after headings: "Introduced in GitLab 16.3."
  Follows Keep a Changelog principles.
- Stylepedia: Per-version release notes in the guide's preface.

Foolish Decision: Foolish documentation follows Google's convention for
version references: "version X or later," "later versions." Version numbers
always use numerals. Don't use "above," "higher," or "newer" — use "later."

---


### Docs-as-Code Workflow and Directory Semantics

What's Contested: Whether documentation must follow docs-as-code practices
or may use other authoring systems.

Positions:
- Google: Advocates for docs-as-code but doesn't prescribe a universal system.
- Microsoft: Doesn't mandate a specific authoring system.
- Chicago: Not applicable.
- Apple: Not applicable.
- GitLab: Strongly advocates docs-as-code. Markdown, Git, Vale, markdownlint,
  CI/CD pipelines.
- Stylepedia: Open-source, GitHub-hosted. Historically DocBook XML; newer
  Red Hat docs use AsciiDoc.

Foolish Decision: Foolish documentation follows a docs-as-code workflow
with semantic directory segregation. The `howto/` directory contains literate
code — documentation meant to be read as prose, with code woven in. The `why/`
directory contains plain documentation explaining design rationale and
motivation, with minimal code. The `how/` directory contains engineering
documentation for contributors and toolchain authors. Machine-targeted
directories contain specification files, grammar definitions, and agent-facing
documents that tools read in full; they don't carry the same prose-first
requirement. For the full directory conventions, read the
[Directory Structure and Documentation Types section of the Foolish Style Guide](StyleGuide.md#directory-structure-and-documentation-types).

---


### Topic Type Taxonomy and Minimum Document Structure

What's Contested: Whether documentation must follow a formal topic-type
taxonomy and what constitutes minimum viable documentation.

Positions:
- Google: Informal organization into conceptual, procedural, and API
  reference content. "If a feature is not documented, it doesn't exist."
- Microsoft: Content types (conceptual, procedural, reference) without a
  rigid taxonomy.
- Chicago: Not applicable.
- Apple: Informal structure.
- GitLab: Explicit CTRT taxonomy: Concept, Task, Reference, Troubleshooting.
  Each type has specific structural requirements.
- Stylepedia: Book-level structure (preface, chapters, appendixes).

Foolish Decision: Topic type in Foolish documentation is governed by
directory, not by formal per-file taxonomy. The `why/` directory holds
high-level reasoning and motivation. The `howto/` directory holds procedural
manuals. The `how/` directory holds engineering documents. This segregation
replaces the need for per-file topic-type labels while making content type
immediately apparent from the file's location.

---


### Cross-Reference Introductory Phrasing

What's Contested: The standard phrase used to introduce a cross-reference
link.

Positions:
- Google: "For more information, see [title]."
- Microsoft: "See also" and "Learn more" as run-in headings.
- Chicago: Not specifically addressed.
- Apple: No specific mandated phrasing.
- GitLab: "For more information, see [title]." Never "Learn more about..."
- Stylepedia: Version 7.0 onward: use "refer to" instead of "see."

Foolish Decision: Every cross-reference in Foolish documentation includes
a call to action that tells the reader why to follow the link. The standard
format is: "For [purpose], read [descriptive title of target content](url)."
The phrase "read" is used in preference to "see," "refer to," or "go to,"
because reading is the actual action and the word is unambiguous in both
human and machine contexts.

---


### First-Person Pronouns in Documentation

What's Contested: Whether first-person pronouns ("I," "we") are permitted.

Positions:
- Google: Generally avoid. "We" may represent the authoring organization.
- Microsoft: "We" acceptable to represent the app or company in UI text.
- Chicago: No prohibition.
- Apple: Avoid. Documentation doesn't use first person.
- GitLab: "I" explicitly prohibited. "We" acceptable for the organization.
- Stylepedia: Uses "we" when speaking as Red Hat.

Foolish Decision: First-person pronouns are not used in Foolish
documentation. Address the reader as "you." Refer to the Foolish project,
language, or toolchain by name. When the documentation itself speaks —
for example, in rationale or explanation — restructure to avoid a first-person
subject: "This document covers..." rather than "We cover..." or
"The following section explains..." rather than "I explain..."

---


### Whitespace Between Sections in Markdown Source

What's Contested: How many blank lines to place between sections,
paragraphs, and headings in Markdown source files.

Positions:
- Google: Not explicitly addressed.
- Microsoft: Use heading styles to control spacing.
- Chicago: Not applicable.
- Apple: Not explicitly addressed.
- GitLab: One blank line before and after headings, paragraphs, and code
  blocks. Enforced via markdownlint.
- Stylepedia: Handled by templates and tooling.

Foolish Decision: Foolish Markdown source files use a graduated blank-line
convention that mirrors heading depth. After an H1, leave 4 blank lines before
the next content. After an H2, leave 3 blank lines. After any heading H3
through H7, leave 2 blank lines. Between body paragraphs, leave 1 blank line.
This rhythm helps both human readers scanning source and parsing tools
identifying section boundaries.

---


### "Above" and "Below" for Cross-References

What's Contested: Whether spatial references ("as described above," "see
below") are acceptable or should be replaced with explicit cross-references.

Positions:
- Google: Avoid. Use cross-references with heading titles or link text.
- Microsoft: Avoid. Screen readers and responsive layouts may reorder content.
- Chicago: Not specifically addressed.
- Apple: Conditional. "Above" and "below" can describe content in close
  proximity; otherwise use formal cross-references.
- GitLab: Implies avoidance through emphasis on descriptive link text.
- Stylepedia: Implies avoidance.

Foolish Decision: Foolish documentation uses "before" and "after" rather
than "above" and "below." This convention carries both a temporal meaning —
the sequence in which a reader encounters content — and a precise index-offset
meaning for tools that process documents as ordered data. "Before" refers to
content at a smaller index position in the current section or in an earlier
section at the next heading level up. "After" refers to content at a larger
index position or in a later section. In flat text, "before" and "after" also
describe position relative to a specific anchor: "before" means a smaller
offset, "after" means a larger offset.

---


### Exclamation Point Usage

What's Contested: Whether exclamation points are ever acceptable in
documentation.

Positions:
- Google: Generally avoid. Never in concept or reference docs.
- Microsoft: Use sparingly.
- Chicago: Standard usage; no prohibition.
- Apple: Implied restraint.
- GitLab: Not explicitly addressed.
- Stylepedia: Avoid in technical writing.

Foolish Decision: A single exclamation point may be used in a Foolish
document — but it had better be genuinely important. The budget is one per
document. Use it when the content is critical enough that the writer would
regret not marking it, not merely when enthusiasm is present. Exclamation
points are not used in headings, code, reference material, or error messages.

---


### Semicolons in Documentation Prose

What's Contested: Whether semicolons are permitted or should be replaced
with shorter sentences.

Positions:
- Google: Permitted but not highlighted.
- Microsoft: Permitted between independent clauses and complex list items.
- Chicago: Permitted and covered in detail.
- Apple: Permitted. No specific restriction.
- GitLab: Prohibited. "Use two sentences instead."
- Stylepedia: Limit use.

Foolish Decision: Semicolons are permitted in Foolish documentation.
They're useful for closely related independent clauses and for maintaining
parallel structure in complex lists. However, if two or three semicolons
appear in succession within a passage, consider whether those sentences
would read more clearly as a short paragraph of separate statements.

---


### Capitalization of Product and Feature Names

What's Contested: Whether internal product features and components are
capitalized as proper nouns or treated as common nouns in lowercase.

Positions:
- Google: Capitalize product names as officially styled. Don't capitalize
  general references unnecessarily.
- Microsoft: Product and service names are proper nouns; capitalize them.
  Technology concepts and generic features are common nouns; lowercase them.
- Chicago: Capitalize proper nouns; follow the entity's own usage.
- Apple: Follow official Apple styling exactly. Feature names are
  capitalized as proper nouns.
- GitLab: Feature names are generally lowercase.
- Stylepedia: Capitalize product names exactly as the vendor specifies.

Foolish Decision: _(Deferred — imported from DOC_AGENT.md)_

---


### Bold for General Emphasis

What's Contested: Whether bold may be used for general emphasis in body text
or only for specific elements like UI labels.

Positions:
- Google: Bold only for UI elements and run-in headings. Use italics for emphasis.
- Microsoft: Bold for UI elements, user input, and run-in headings.
- Chicago: Italic is the standard for emphasis in body text.
- Apple: Bold for UI elements. Italic for emphasis.
- GitLab: Bold only for UI elements. "Don't use bold for emphasis or keywords."
- Stylepedia: Bold for GUI elements. Emphasis handled by content clarity.

Foolish Decision: Bold in Foolish documentation is reserved for small,
easily misread words whose weight matters — words like "not," "only," "all,"
or "never" that a reader might skim past with consequences. Bold is not used
for topic sentences, key phrases, or general emphasis. A heading, a list
item, or a sub-subsection serves better than inline bold for introducing
concepts. If the structure of the document cannot carry the meaning, the
structure needs work — not more bold. For the complete rule, read
[guideline 5.3 of the Formatting section](styleguide.md#5-formatting).

---

*End of StyleGuide.md version 0.4.0-draft.*

---

Implementation Notes:

1. Parameters marked _(to be decided)_ in the Conventions Registry must be
   resolved before this guide is published as version 1.0.
2. Parameters marked [DEFERRED] will be resolved when `DOC_AGENTS.md` is
   imported and integrated.
3. All decided rules should be encoded as Vale rules or markdownlint
   configuration and enforced in CI/CD.
4. This guide is version-controlled alongside the Foolish language source code.
   Changes follow the same review process as code changes.
5. Revisit annually.
