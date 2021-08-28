use event::Event;
use graph::GraphNodeId;
use keybinding_event_loop::KeybindingEventLoop;
use log::{error, info};
use lua::graph_proxy::GraphProxy;
use mlua::FromLua;
use platform::{Window, WindowId, WindowPosition, WindowSize};
use server::Server;
use std::{
    sync::mpsc::{channel, Sender},
    thread,
};
use window_event_loop::WindowEventLoop;

use crate::{
    config::Config,
    graph::{Graph, GraphNode, GraphNodeGroupKind},
    platform::NativeWindow,
    window_event_loop::WindowEventKind,
};

/// Responsible for handling events like when a window is created, deleted, etc.
pub trait EventLoop {
    fn run(tx: Sender<Event>);
    fn stop();
    fn spawn(tx: Sender<Event>) {
        thread::spawn(move || {
            Self::run(tx);
        });
    }
}

mod config;
mod event;
mod graph;
mod key;
mod key_combination;
mod keybinding;
mod keybinding_event_loop;
mod logging;
mod lua;
mod modifiers;
mod platform;
mod server;
mod window_event_loop;

enum WindowEventEffect {
    None,
    // the name of the event as string and the id of the window
    Layout(String, WindowId),
}

fn render_node(id: GraphNodeId, graph: &Graph, pos: WindowPosition, size: WindowSize) {
    let node = graph
        .get_node(id)
        .expect("Cannot render a node that doesn't exist");

    match node {
        GraphNode::Group(kind) => {
            let children = graph.get_children(id);

            if children.len() == 0 {
                return;
            }

            match kind {
                GraphNodeGroupKind::Row => {
                    let col_width = size.width / children.len();
                    let mut x = pos.x;
                    for child_id in children {
                        render_node(
                            child_id,
                            graph,
                            WindowPosition::new(x, pos.y),
                            WindowSize::new(col_width, size.height),
                        );
                        x += col_width as isize;
                    }
                }
                GraphNodeGroupKind::Col => {
                    let row_height = size.height / children.len();
                    let mut y = pos.y;
                    for child_id in children {
                        render_node(
                            child_id,
                            graph,
                            WindowPosition::new(pos.x, y),
                            WindowSize::new(size.width, row_height),
                        );
                        y += row_height as isize;
                    }
                }
            }
        }
        GraphNode::Window(win_id) => {
            let win = Window::new(*win_id);
            win.reposition(pos);
            win.resize(size);
        }
    }
}

fn render_graph(graph: &Graph) {
    // - 40 because of taskbar
    render_node(
        graph.root_node_id,
        graph,
        WindowPosition::new(0, 0),
        WindowSize::new(1920, 1040),
    );
}

fn print_node(depth: usize, id: GraphNodeId, graph: &Graph) {
    let node = graph
        .get_node(id)
        .expect("Cannot print a node that doesn't exist");

    let indent = "|   ".repeat(depth);

    match node {
        GraphNode::Group(kind) => {
            let children = graph.get_children(id);

            let tag = match kind {
                GraphNodeGroupKind::Row => "Row",
                GraphNodeGroupKind::Col => "Col",
            };

            println!("{}{}", indent, tag);

            for child_id in children {
                print_node(depth + 1, child_id, graph);
            }
        }
        GraphNode::Window(win_id) => {
            println!("{}Win({})", indent, win_id);
        }
    }
}

fn print_graph(graph: &Graph) {
    print_node(0, graph.root_node_id, graph)
}

fn main() {
    logging::init().expect("Failed to initialize logging");
    info!("Initialized logging");

    let (tx, rx) = channel::<Event>();
    let mut rt = match lua::init(tx.clone()) {
        Ok(x) => x,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };
    let mut config = Config::default();
    // Graph for managed windows
    let mut graph = Graph::new();

    // lua::repl::spawn(tx.clone());
    // info!("Repl started");

    Server::spawn(tx.clone());
    info!("IPC Server started");

    WindowEventLoop::spawn(tx.clone());
    info!("Window event loop spawned");

    KeybindingEventLoop::spawn(tx.clone());
    info!("Keybinding event loop spawned");

    info!("Starting main event loop");
    while let Ok(event) = rx.recv() {
        match event {
            Event::Window(win_event) => {
                let effect = match win_event.kind {
                    WindowEventKind::Created => {
                        let size = win_event.window.get_size();

                        if size.width >= config.min_width && size.height >= config.min_height {
                            info!("'{}' created", win_event.window.get_title());

                            WindowEventEffect::Layout(
                                String::from("created"),
                                win_event.window.get_id(),
                            )
                        } else {
                            WindowEventEffect::None
                        }
                    }
                    WindowEventKind::Deleted => {
                        info!("'{}' deleted", win_event.window.get_title());
                        WindowEventEffect::Layout(
                            String::from("deleted"),
                            win_event.window.get_id(),
                        )
                    }
                    WindowEventKind::Minimized => WindowEventEffect::Layout(
                        String::from("minimized"),
                        win_event.window.get_id(),
                    ),
                };

                match effect {
                    WindowEventEffect::None => {}
                    WindowEventEffect::Layout(event, win_id) => {
                        // We need to use the scope here to make the rust type system happy.
                        // scope drops the userdata when the function has finished.
                        let res = rt.rt.scope(|scope| {
                            let ud = scope.create_nonstatic_userdata(GraphProxy(&mut graph))?;
                            mlua::Function::from_lua(rt.rt.load("nog.layout").eval()?, rt.rt)?
                                .call((ud, event, win_id))?;
                            Ok(())
                        });

                        if let Err(e) = res {
                            error!("{}", e);
                        }

                        if graph.dirty {
                            info!("Have to rerender!");
                            render_graph(&graph);
                            print_graph(&graph);
                            graph.dirty = false;
                        }
                    }
                };
            }
            Event::Keybinding(kb) => {
                info!("Received keybinding {}", kb.to_string());

                let cb = rt
                    .rt
                    .named_registry_value::<str, mlua::Function>(&kb.get_id().to_string())
                    .expect("Registry value of a keybinding somehow disappeared?");

                if let Err(e) = cb.call::<(), ()>(()) {
                    error!("{}", e);
                }
            }
            Event::Action(action) => match action {
                event::Action::UpdateConfig { key, update_fn } => {
                    update_fn.0(&mut config);
                    info!("Updated config property: {:#?}", key);
                }
                event::Action::ExecuteLua {
                    code,
                    capture_stdout,
                    cb,
                } => {
                    if capture_stdout {
                        rt.eval(
                            r#"
                            _G.__stdout_buf = ""
                            _G.__old_print = print
                            _G.print = function(...)
                                if _G.__stdout_buf ~= "" then
                                    _G.__stdout_buf = _G.__stdout_buf .. "\n"
                                end
                                local outputs = {}
                                for _,x in ipairs({...}) do
                                    table.insert(outputs, tostring(x))
                                end
                                local output = table.concat(outputs, "\t")
                                _G.__stdout_buf = _G.__stdout_buf .. output
                            end
                                    "#,
                        )
                        .unwrap();

                        let code_res = rt.eval(&code);

                        let stdout_buf =
                            String::from_lua(rt.eval("_G.__stdout_buf").unwrap(), rt.rt).unwrap();

                        cb.0(code_res.map(move |x| {
                            if stdout_buf.is_empty() {
                                format!("{:?}", x)
                            } else {
                                format!("{}\n{:?}", stdout_buf, x)
                            }
                        }));

                        rt.eval(
                            r#"
                            _G.print = _G.__old_print
                            _G.__stdout_buf = nil
                            _G.__old_print = nil
                                    "#,
                        )
                        .unwrap();
                    } else {
                        cb.0(rt.eval(&code).map(|x| format!("{:?}", x)));
                    }
                }
                event::Action::CreateKeybinding {
                    mode,
                    key_combination,
                } => {
                    KeybindingEventLoop::add_keybinding(key_combination.get_id());
                    info!("Created {:?} keybinding: {:#?}", mode, key_combination);
                }
                event::Action::RemoveKeybinding { key } => {
                    // KeybindingEventLoop::remove_keybinding(key_combination.get_id());
                    info!("Removed keybinding: {:#?}", key);
                }
            },
            Event::RenderGraph => {
                render_graph(&graph);
            }
        }
    }
}
