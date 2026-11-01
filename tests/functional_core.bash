# The readings of the bash's tools, read by tests/functional_core.rs: each is defined in
# src/bash.rs and nowhere else in src/, by its name, a method included. One name per line.

# Lines as awk, `while IFS= read -r`, and `sed` split a file.
awk_lines
read_lines
sed_lines
# The space class of POSIX, of a byte and of a character, and the bytes without it.
is_space
is_space_char
trim
# The first word `read -r word _` splits off with the default IFS.
first_word

# `find` and `contains` of src/bash.rs are no reader: they read no byte otherwise than Rust, and
# the names are taken by `rules::find` and a method of src/glob.rs. The test reads names only, so
# a copy under another name, or a closure, passes it; that is review.
