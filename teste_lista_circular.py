import rust_lista_circular as rlc

if __name__ == "__main__":
    lista = rlc.Circle(10)
    for i in range(30):
        lista.insert_after(i)

    for i in range(30):
        print(lista.last())
