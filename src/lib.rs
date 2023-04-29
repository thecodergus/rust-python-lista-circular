use pyo3::prelude::*;
use std::sync::{Arc, Mutex};

#[pyclass]
pub struct Circle {
    head: Arc<Mutex<Node>>,
}

struct Node {
    val: PyObject,
    next: Option<Arc<Mutex<Node>>>,
    last: Option<Arc<Mutex<Node>>>,
}

#[pymethods]
impl Circle {
    pub fn remove_current(&mut self) -> PyObject {
        let ret = self.current_value();
        let (next, last) = {
            let head_guard = self.head.lock().unwrap();
            (
                head_guard.next.as_ref().map(Clone::clone),
                head_guard.last.as_ref().map(Clone::clone),
            )
        };
        let mut next = next.unwrap();
        let mut last = last.unwrap();

        Node::combine(&mut last, &mut next);
        self.head = next;
        return ret;
    }

    #[new]
    pub fn new(val: PyObject) -> Circle {
        return Circle {
            head: Node::new(val),
        };
    }

    pub fn current_value(&self) -> PyObject {
        let head_guard = self.head.lock().unwrap();
        return head_guard.value();
    }

    pub fn move_next(&mut self) -> PyObject {
        let next = {
            let head_guard = self.head.lock().unwrap();
            head_guard.next.as_ref().map(Clone::clone)
        };
        match next {
            Some(x) => self.head = x.clone(),
            None => (),
        }
        let head_guard = self.head.lock().unwrap();
        head_guard.value()
    }

    pub fn move_previous(&mut self) -> PyObject {
        let last = {
            let head_guard = self.head.lock().unwrap();
            head_guard.last.as_ref().map(Clone::clone)
        };
        match last {
            Some(x) => self.head = x.clone(),
            None => (),
        }
        let head_guard = self.head.lock().unwrap();
        head_guard.value()
    }

    pub fn insert_after_current(&mut self, val: PyObject) {
        let (node, mut head) = {
            let head_guard = self.head.lock().unwrap();
            (
                head_guard.next.as_ref().map(Clone::clone),
                self.head.clone(),
            )
        };
        let mut new_node = Node::new(val);
        Node::combine(&mut head, &mut new_node);
        Node::combine(&mut new_node, &mut node.unwrap_or(head));
    }

    pub fn insert_after_step(&mut self, val: PyObject) {
        self.insert_after_current(val);
        self.move_next();
    }

    pub fn insert_and_move_next(&mut self, val: PyObject) {
        let (node, mut head) = {
            let head_guard = self.head.lock().unwrap();
            (
                head_guard.last.as_ref().map(Clone::clone),
                self.head.clone(),
            )
        };
        let mut new_node = Node::new(val);
        Node::combine(&mut new_node, &mut head);
        Node::combine(&mut node.unwrap_or(head), &mut new_node);
    }

    pub fn insert_before_current(&mut self, val: PyObject) {
        let (node, mut head) = {
            let head_guard = self.head.lock().unwrap();
            (
                head_guard.last.as_ref().map(Clone::clone),
                self.head.clone(),
            )
        };
        let mut new_node = Node::new(val);
        Node::combine(&mut new_node, &mut head);
        Node::combine(&mut node.unwrap_or(head), &mut new_node);
    }

    pub fn is_empty(&self) -> bool {
        match self.head.lock() {
            Ok(_head_guard) => false,
            Err(_) => true,
        }
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
        let mut node1_guard = node1.lock().unwrap();
        let mut node2_guard = node2.lock().unwrap();
        node1_guard.next = Some(node2.clone());
        node2_guard.last = Some(node1.clone());
    }
}

#[pymodule]
pub fn rust_lista_circular(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Circle>()?;

    Ok(())
}
