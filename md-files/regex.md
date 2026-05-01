# Allowed Regex Character Set

The regular expression syntax used throughout the project is based on the printable ASCII character set.

We allow all printable ASCII characters in the range:

```text
0x20 (space) through 0x7E (~)
```

with the exceptions and control characters described below.

## Disallowed Characters

The following printable ASCII characters are reserved and therefore not allowed in regular expressions:

```text
" $ . ? [ ] \ ^ { }
```

## Control Characters

The following characters are treated as control/meta characters by the parser:

```text
| + * ( )
```

These characters have special meaning in the grammar and are therefore interpreted differently from normal literal characters.

## Allowed Literal Characters

Any printable ASCII character not listed above may be used as a literal character.

Examples of allowed literal characters include:

```text
a b c d
A B C D
0 1 2 3
! # % & ' , - / : ; < = > @
_ ` ~
```

## Notes

- The character set is restricted to printable ASCII only.
- Non-printable ASCII characters are not allowed unless explicitly handled by the implementation.
- Control/meta characters must follow the grammar rules of the language.
- Reserved characters are invalid and may not appear in expressions.
