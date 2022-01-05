# Regex Replacer

This repository contains rust code to quickly parse replacement and removal
regexes from a YAML configuration file, and apply them to multiple files in
parallel.

## Configuration

The input to the binary is a list of files without the language suffix.

The patterns are read from a YAML file specified via the `-p` or `--pattern` flag.

The `remove` key removes all lines which match a regex in the list, and the
`replace` key contains all the replacements that should be performed.

```YAML
remove:
    # If a line starts with a hyphen, skip it.
  - '^-'

    # Match any repeated punctuation with or without a space.
  - '([?!,."_]\s*){2,}'

    # Match any sort of parenthesis or bracket.
  - '[<>{}()\[\]]'

    # Match double forward slash.
  - '//'

    # Single underscore.
  - '_'

    # Double backslash.
  - '\\'

    # Any line starting with a number followed by a period and then a space or word.
  - '^\d*\.[\w\s]'

replace:
    # Match language tags '- (EN)'
  - regex: '.*-\s*\([A-Z]{2}\)\s*'
    replacement: ''

    # Match hyphens surrounded by non word characters
  - regex: '(\s-\s)|(\s-\w)'
    replacement: ' – '

    # Match double hyphens surrounded by word characters 'a--b'
  - regex: '\w--\w'
    replacement: '-'

    # Match one or more space characters
  - regex: '\s{2,}'
    replacement: ' '
```

