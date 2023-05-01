use pyo3::prelude::*;
use std::sync::{Arc, Mutex, MutexGuard};

// 1. Mudar de "Arc<Mutex<Node>>" para Option<Arc<Mutex<Node>>> para permitir listas vazias
// 2. Uma função para contar o numero de itens presentes na lista circular
// 3. Adicionar um limite de itens na lista circular, e que ao tentar adicionar um novo item, caso o limite seja atingido, o item mais antigo seja removido
// 4. Uma função para retunar todos os itens da lista circular como um vetor (que no Python é uma lista)
// 5. Adicioar as tiagens

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

#[pymethods]
impl Circle {
    #[new]
    pub fn new(max_size: Option<usize>) -> Circle {
        Circle {
            head: None,
            max_size,
        }
    }

    pub fn current_value(&self) -> PyObject {
        match self.head {
            Some(ref x) => {
                let head_guard: MutexGuard<Node> = x.lock().unwrap();

                return head_guard.value();
            }
            None => Python::with_gil(|py| py.None()),
        }
    }
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

    pub fn move_next(&mut self) -> PyObject {
        if let Some(ref head) = self.head {
            let next = {
                let head_guard: MutexGuard<Node> = head.lock().unwrap();
                head_guard.next.as_ref().map(Clone::clone)
            };

            if let Some(next) = next {
                self.head = Some(next);
            }
        }

        return self.current_value();
    }

    pub fn move_previous(&mut self) -> PyObject {
        if let Some(ref head) = self.head {
            let last = {
                let head_guard: MutexGuard<Node> = head.lock().unwrap();
                head_guard.last.as_ref().map(Clone::clone)
            };

            if let Some(last) = last {
                self.head = Some(last);
            }
        }

        return self.current_value();
    }

    pub fn insert_after_current(&mut self, val: PyObject) {
        if let Some(max_size) = self.max_size {
            if self.count() >= max_size {
                self.move_next();
                self.remove_current();
                self.move_previous();
            }
        }

        if let Some(ref head) = self.head {
            let (node, mut head) = {
                let head_guard: MutexGuard<Node> = head.lock().unwrap();
                (head_guard.next.as_ref().map(Clone::clone), head.clone())
            };
            let mut new_node: Arc<Mutex<Node>> = Node::new(val);
            Node::combine(&mut head, &mut new_node);
            Node::combine(&mut new_node, &mut node.unwrap_or(head));
        } else {
            let new_node = Node::new(val);
            self.head = Some(new_node);
        }
    }

    pub fn insert_before_current(&mut self, val: PyObject) {
        if let Some(max_size) = self.max_size {
            if self.count() >= max_size {
                self.move_previous();
                self.remove_current();
            }
        }

        if let Some(ref head) = self.head {
            let (node, mut head) = {
                let head_guard = head.lock().unwrap();
                (head_guard.last.as_ref().map(Clone::clone), head.clone())
            };
            let mut new_node: Arc<Mutex<Node>> = Node::new(val);
            Node::combine(&mut new_node, &mut head);
            Node::combine(&mut node.unwrap_or(head), &mut new_node);
        } else {
            let new_node = Node::new(val);
            self.head = Some(new_node);
        }
    }

    pub fn is_empty(&self) -> bool {
        return self.head.is_none();
    }

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
