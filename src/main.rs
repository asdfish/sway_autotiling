use swayipc::{
    Connection,
    Event,
    EventType,
    Fallible,
    Node,
    NodeType,
    WindowChange,
};

use std::option::Option;

macro_rules! unsafe_set {
    ($var:ident, $val:expr) => {
        unsafe {
            $var = $val;
        }
    }
}
macro_rules! unwrap_or_continue_option {
    (mut $new:ident, $orig:expr) => {
        if $orig.is_none() {
            continue;
        }
        let mut $new = $orig.unwrap();
    };
    ($new:ident, $orig:expr) => {
        if $orig.is_none() {
            continue;
        }
        let $new = $orig.unwrap();
    };
}
macro_rules! unwrap_or_continue_result {
    (mut $new:ident, $orig:expr) => {
        if $orig.is_err() {
            continue;
        }
        let mut $new = $orig.unwrap();
    };
    ($new:ident, $orig:expr) => {
        if $orig.is_err() {
            continue;
        }
        let $new = $orig.unwrap();
    };
}

static mut FOCUSED_NODE_PID: Option<i32> = None;
static mut MASTER_NODE_PID: Option<i32> = None;
static mut MASTER_NODE_X: Option<i32> = None;

fn set_master_node(node: &Node) -> &Node {
    unsafe_set!(MASTER_NODE_PID, node.pid);
    unsafe_set!(MASTER_NODE_X, Some(node.rect.x));
    return node;
}

fn node_callback(node: &Node) -> &Node {
    let visible = node.visible.or(Some(false)).unwrap();
    let pid: i32 = node.pid.or(Some(-1)).unwrap();

    if ! visible || node.node_type != NodeType::Con || pid == -1 {
        return node;
    }

    if node.focused {
        unsafe_set!(FOCUSED_NODE_PID, Some(pid));
    }

    unsafe {
        match MASTER_NODE_X {
            Some(master_node_x) => {
                if node.rect.x < master_node_x {
                    return set_master_node(node);
                }
            },
            None => return set_master_node(node),
        }
    }

    return node;
}
fn node_walk(root: &Node, callback: &dyn Fn(&Node) -> &Node) {
    let mut to_walk: Vec<&Node> = vec![root];

    while let Some(node) = to_walk.pop() {
        callback(node);

        for child_node in &node.nodes {
            to_walk.push(child_node);
        }
    }
}

fn main() -> Fallible<()> {
    for event in Connection::new()?.subscribe([ EventType::Window ])? {
        unwrap_or_continue_result!(event, event);

        match event {
            Event::Window(window) => {
                if window.change != WindowChange::Focus {
                    continue;
                }

                let connection = Connection::new();
                unwrap_or_continue_result!(mut connection, connection);

                let tree = connection.get_tree();
                unwrap_or_continue_result!(tree, tree);

                unsafe_set!(FOCUSED_NODE_PID, None);
                unsafe_set!(MASTER_NODE_PID, None);
                unsafe_set!(MASTER_NODE_X, None);
                node_walk(&tree, &node_callback);

                unsafe {
                    unwrap_or_continue_option!(master_node_pid, MASTER_NODE_PID);
                    unwrap_or_continue_option!(focused_node_pid, FOCUSED_NODE_PID);

                    if master_node_pid == focused_node_pid {
                        let _ = connection.run_command("splith");
                    } else {
                        let _ = connection.run_command("splitv");
                    }
                }
            },
            _ => continue,
        }
    }

    return Ok(());
}
