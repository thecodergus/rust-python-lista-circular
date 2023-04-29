import time

lista = [*range(10_000_000)]

# Filter
a = time.time()
f_filtro = filter(lambda x: x & 1 == 0, lista)
[a for a in f_filtro]
b = time.time()

c = time.time()
f_lcompre = [x for x in lista if x & 1 == 0]
[a for a in f_lcompre]
d = time.time()


# Map
e = time.time()
m_map = map(lambda x: x + 2, lista)
[a for a in m_map]
f = time.time()

g = time.time()
m_lcompre = [x + 2 for x in lista]
[a for a in m_lcompre]
h = time.time()

print(f"filter: {(b - a):.4f} s\nlist compre: {(d - c):.4f} s")
print(f"map: {(f - e):.4f} s\nlist compre: {(h - g):.4f} s")
