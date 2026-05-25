import re

with open('cloak-admin/src/App.tsx', 'r') as f:
    content = f.read()

# We will just write a python script to do the replacement safely.
# Actually, since it's just generating code, it's easier to do it via python string replacements.

