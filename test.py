x = """\
zaaaaaaaaaaaaaaaigiurhuihziuhziu
azzz/b/c
"""


lines = x.splitlines()

lines.sort()

x = "\n".join(lines)

print(x)
