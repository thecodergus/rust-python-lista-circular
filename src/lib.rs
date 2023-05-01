use pyo3::prelude::*;
use std::sync::{Arc, Mutex, MutexGuard};

// Definindo a estrutura Circle e Node
#[pyclass]
pub struct Circle {
    head: Option<Arc<Mutex<Node>>>,
    max_size: Option<usize>,
}

struct Node {
    val: PyObject,
    next: Option<Arc<Mutex<Node>>>,
    last: Option<Arc<Mutex<Node>>>,
}

// Implementando métodos para a estrutura Circle
#[pymethods]
impl Circle {
    // Inicializa um objeto Circle com tamanho máximo opcional
    #[new]
    pub fn new(max_size: Option<usize>) -> Circle {
        Circle {
            head: None,
            max_size,
        }
    }

    // Retorna o valor do nó atual (cabeça) da lista circular
    pub fn current_value(&self) -> PyObject {
        match self.head {
            Some(ref x) => {
                let head_guard: MutexGuard<Node> = x.lock().unwrap();

                return head_guard.value();
            }
            None => Python::with_gil(|py| py.None()),
        }
    }

    // Remove o nó atual e retorna seu valor
    pub fn remove_current(&mut self) -> PyObject {
        let ret: PyObject = self.current_value();
        if let Some(ref head) = self.head {
            let (next, last) = {
                let head_guard: MutexGuard<Node> = head.lock().unwrap();
                (
                    head_guard.next.as_ref().map(Clone::clone),
                    head_guard.last.as_ref().map(Clone::clone),
                )
            };

            match (next, last) {
                (Some(mut next), Some(mut last)) => {
                    Node::combine(&mut last, &mut next);
                    self.head = Some(next);
                }
                _ => {
                    self.head = None;
                }
            }
        }
        return ret;
    }

    // Movem a cabeça da lista para o próximo nó e retornam o valor do nó atualizado
    pub fn move_next(&mut self) -> PyObject {
        self.move_to(true)
    }

    // Movem a cabeça da lista para o nó anterior e retornam o valor do nó atualizado
    pub fn move_previous(&mut self) -> PyObject {
        self.move_to(false)
    }

    fn move_to(&mut self, move_to_next: bool) -> PyObject {
        if let Some(ref head) = self.head {
            let adjacent_node = {
                let head_guard: MutexGuard<Node> = head.lock().unwrap();
                if move_to_next {
                    head_guard.next.as_ref().map(Clone::clone)
                } else {
                    head_guard.last.as_ref().map(Clone::clone)
                }
            };

            if let Some(adjacent_node) = adjacent_node {
                self.head = Some(adjacent_node);
            }
        }

        return self.current_value();
    }

    // Insere um novo nó com valor fornecido após o nó atual
    // Se o tamanho máximo for atingido, o nó mais antigo é removido
    pub fn insert_after_current(&mut self, val: PyObject) {
        self.insert_node(val, true);
    }

    // Insere um novo nó com valor fornecido antes do nó atual
    // Se o tamanho máximo for atingido, o nó mais antigo é removido
    pub fn insert_before_current(&mut self, val: PyObject) {
        self.insert_node(val, false);
    }

    fn insert_node(&mut self, val: PyObject, insert_after: bool) {
        if let Some(max_size) = self.max_size {
            if self.count() >= max_size {
                if insert_after {
                    self.move_next();
                } else {
                    self.move_previous();
                }
                self.remove_current();
            }
        }

        if let Some(ref head) = self.head {
            let (adjacent_node, mut head) = {
                let head_guard: MutexGuard<Node> = head.lock().unwrap();
                if insert_after {
                    (head_guard.next.as_ref().map(Clone::clone), head.clone())
                } else {
                    (head_guard.last.as_ref().map(Clone::clone), head.clone())
                }
            };
            let mut new_node: Arc<Mutex<Node>> = Node::new(val);
            Node::combine(&mut head, &mut new_node);
            Node::combine(&mut new_node, &mut adjacent_node.unwrap_or(head));
        } else {
            let new_node = Node::new(val);
            self.head = Some(new_node);
        }
    }

    // Verifica se a lista está vazia
    pub fn is_empty(&self) -> bool {
        return self.head.is_none();
    }

    // Conta o número de itens na lista circular
    pub fn count(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        let mut count: usize = 0;
        let mut current: Option<Arc<Mutex<Node>>> = self.head.clone();
        loop {
            count += 1;
            let next: Option<Arc<Mutex<Node>>> = {
                let current_guard = current.as_ref().unwrap().lock().unwrap();
                current_guard.next.as_ref().map(Clone::clone)
            };
            if let Some(next) = next {
                if Arc::ptr_eq(&next, self.head.as_ref().unwrap()) {
                    break;
                }
                current = Some(next);
            } else {
                break;
            }
        }
        return count;
    }

    // Retorna todos os itens da lista circular como um vetor (lista em Python)
    pub fn to_vec(&self) -> Vec<PyObject> {
        let mut ret: Vec<PyObject> = Vec::new();

        match &self.head {
            Some(head) => {
                let mut current: Option<Arc<Mutex<Node>>> = Some(head.clone());

                loop {
                    let current_value: Py<PyAny> = {
                        let current_guard: MutexGuard<Node> =
                            current.as_ref().unwrap().lock().unwrap();
                        current_guard.value()
                    };

                    ret.push(current_value);

                    let next: Option<Arc<Mutex<Node>>> = {
                        let current_guard: MutexGuard<Node> =
                            current.as_ref().unwrap().lock().unwrap();
                        current_guard.next.as_ref().map(Clone::clone)
                    };

                    if let Some(next) = next {
                        if Arc::ptr_eq(&next, head) {
                            break;
                        }
                        current = Some(next);
                    } else {
                        break;
                    }
                }
            }
            None => {
                // Quando a lista estiver vazia, retorne o vetor vazio.
                return ret;
            }
        }

        return ret;
    }
}

impl Node {
    pub fn new(val: PyObject) -> Arc<Mutex<Node>> {
        Arc::new(Mutex::new(Node {
            val,
            next: None,
            last: None,
        }))
    }
    pub fn value(&self) -> PyObject {
        self.val.clone()
    }

    pub fn combine(node1: &mut Arc<Mutex<Node>>, node2: &mut Arc<Mutex<Node>>) {
        let mut node1_guard: MutexGuard<Node> = node1.lock().unwrap();
        let mut node2_guard: MutexGuard<Node> = node2.lock().unwrap();
        node1_guard.next = Some(node2.clone());
        node2_guard.last = Some(node1.clone());
    }
}

#[pymodule]
pub fn rust_lista_circular(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Circle>()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // use pyo3::types::PyString;

    #[test]
    fn test_circle_empty() {
        //     Python::with_gil(|_py| {
        //         let circle = Circle::new(None);
        //         assert!(circle.is_empty());
        //     });
        let circle = Circle::new(None);
        assert!(circle.is_empty());
    }

    // #[test]
    // fn test_circle_insert_and_count() {
    //     let gil = Python::acquire_gil();
    //     let py = gil.python();
    //     let mut circle = Circle::new(None);

    //     circle.insert_after_current(py.None());
    //     assert_eq!(circle.count(), 1);

    //     circle.insert_after_current(py.None());
    //     assert_eq!(circle.count(), 2);
    // }
}
