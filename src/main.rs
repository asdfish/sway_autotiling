use swayipc::{Connection, Event, EventType, Fallible, Node, NodeType, WindowChange};

struct Window {
    x: i32,
    pid: i32,
}
impl Window {
    pub fn from_node(node: &Node) -> Window {
        Window {
            x: node.rect.x,
            pid: node.pid.unwrap_or(-1),
        }
    }
}

struct SwayState {
    focused_window: Option<Window>,
    master_window: Option<Window>,
}
impl SwayState {
    pub const fn new() -> SwayState {
        SwayState {
            focused_window: None,
            master_window: None,
        }
    }

    pub fn update(&mut self, root_node: Node) {
        let mut to_walk: Vec<Node> = vec![root_node];

        while let Some(node) = to_walk.pop() {
            let visible: bool = node.visible.unwrap_or(false);
            let pid: i32 = node.pid.unwrap_or(-1);

            if node.node_type == NodeType::Con && visible && pid != -1 {
                if node.focused {
                    self.focused_window = Some(Window::from_node(&node));
                }

                match &self.master_window {
                    Some(master_window) => {
                        if node.rect.x < master_window.x {
                            self.master_window = Some(Window::from_node(&node));
                        }
                    }
                    None => self.master_window = Some(Window::from_node(&node)),
                }
            }

            for child_node in node.nodes {
                to_walk.push(child_node);
            }
        }
    }
    pub fn reset(&mut self) {
        self.focused_window = None;
        self.master_window = None;
    }
}

fn main() -> Fallible<()> {
    let mut sway_state: SwayState = SwayState::new();

    for event in Connection::new()?.subscribe([EventType::Window, EventType::Shutdown])? {
        match event.unwrap() {
            Event::Window(window) => {
                if window.change != WindowChange::Focus {
                    continue;
                }

                let Ok(mut connection) = Connection::new() else {
                    continue;
                };
                let Ok(tree) = connection.get_tree() else {
                    continue;
                };

                sway_state.reset();
                sway_state.update(tree);

                let Some(ref focused_window) = sway_state.focused_window else {
                    continue;
                };
                let Some(ref master_window) = sway_state.master_window else {
                    continue;
                };

                let _ = if focused_window.pid == master_window.pid {
                    connection.run_command("splith")
                } else {
                    connection.run_command("splitv")
                }
                .unwrap();
            }
            Event::Shutdown(_) => {
                panic!("Sway shutdown");
            }
            _ => continue,
        };
    }

    Ok(())
}
