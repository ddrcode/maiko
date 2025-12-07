default:
    echo 'Hello, world!'

spell:
    echo 'Spellchecking...'
    cargo spellcheck --cfg spellcheck.toml
