use event::{Action, Event, WindowAction, WorkspaceAction};
use graph::GraphNodeId;
use keybinding_event_loop::KeybindingEventLoop;
use log::{error, info};
use lua::{graph_proxy::GraphProxy, LuaRuntime};
use mlua::FromLua;
use platform::{Window, WindowId, WindowPosition, WindowSize};
use server::Server;
use std::{
    collections::HashMap,
    sync::mpsc::{channel, Sender},
    thread,
};
use window_event_loop::WindowEventLoop;
use workspace::Workspace;

use crate::{
    cleanup::WindowCleanup,
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

mod cleanup;
mod config;
mod direction;
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
mod workspace;

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

            println!("{}[{}]{}", indent, id, tag);

            for child_id in children {
                print_node(depth + 1, child_id, graph);
            }
        }
        GraphNode::Window(win_id) => {
            println!("{}[{}]Win({})", indent, id, win_id);
        }
    }
}

fn print_graph(graph: &Graph) {
    print_node(0, graph.root_node_id, graph)
}

fn call_layout_function<TArgs>(
    rt: &LuaRuntime,
    workspace: &mut Workspace,
    event: String,
    args: TArgs,
) -> mlua::Result<mlua::Value<'static>>
where
    TArgs: mlua::ToLuaMulti<'static>,
{
    // We need to use the scope here to make the rust type system happy.
    // scope drops the userdata when the function has finished.
    let res: mlua::Value = rt.rt.scope(|scope| {
        let ud = scope.create_nonstatic_userdata(GraphProxy(&mut workspace.graph))?;
        mlua::Function::from_lua(rt.rt.load("nog.layout").eval()?, rt.rt)?.call((ud, event, args))
    })?;

    if workspace.graph.dirty {
        info!("Have to rerender!");
        render_graph(&workspace.graph);
        print_graph(&workspace.graph);
        workspace.graph.dirty = false;
    }

    Ok(res)
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
    let mut workspace = Workspace::new(tx.clone());
    let mut window_cleanup: HashMap<WindowId, WindowCleanup> = HashMap::new();

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
            Event::Window(win_event) => match win_event.kind {
                WindowEventKind::FocusChanged => {
                    if workspace.focus_window(win_event.window.get_id()).is_ok() {
                        info!("Focused window with id {}", win_event.window.get_id());
                        win_event.window.focus();
                    }
                }
                WindowEventKind::Created => {
                    let win = win_event.window;
                    let size = win.get_size();
                    let pos = win.get_position();

                    if size.width >= config.min_width && size.height >= config.min_height {
                        info!("'{}' created", win.get_title());

                        let cleanup = window_cleanup.entry(win.get_id()).or_default();

                        cleanup.reset_transform = Some(Box::new(move || {
                            win.reposition(pos);
                            win.resize(size);
                        }));

                        if config.remove_decorations {
                            cleanup.add_decorations = Some(win.remove_decorations())
                        }

                        call_layout_function(
                            &rt,
                            &mut workspace,
                            String::from("created"),
                            win.get_id(),
                        );
                    }
                }
                WindowEventKind::Deleted => {
                    info!("'{}' deleted", win_event.window.get_title());

                    call_layout_function(
                        &rt,
                        &mut workspace,
                        String::from("deleted"),
                        win_event.window.get_id(),
                    );
                }
                WindowEventKind::Minimized => {
                    call_layout_function(
                        &rt,
                        &mut workspace,
                        String::from("minimized"),
                        win_event.window.get_id(),
                    );
                }
            },
            Event::Keybinding(kb) => {
                info!("Received keybinding {}", kb.to_string());

                let cb = rt
                    .rt
                    .named_registry_value::<str, mlua::Function>(&kb.get_id().to_string())
                    .expect("Registry value of a keybinding somehow disappeared?");

                if let Err(e) = cb.call::<(), ()>(()) {
                    error!(
                        "{}",
                        match e {
                            mlua::Error::CallbackError { cause, .. } => cause.to_string(),
                            e => e.to_string(),
                        }
                    );
                }
            }
            Event::Action(action) => match action {
                Action::Window(action) => match action {
                    WindowAction::Focus(win_id) => {
                        let win = Window::new(win_id);
                        win.focus();
                    }
                    WindowAction::Close(maybe_win_id) => {
                        let maybe_win_id = maybe_win_id.or_else(|| {
                            workspace
                                .get_focused_node()
                                .and_then(|n| n.try_get_window_id())
                        });

                        if let Some(id) = maybe_win_id {
                            Window::new(id).close();
                        }
                    }
                    WindowAction::Manage(maybe_id) => {
                        let win = maybe_id
                            .map(|id| Window::new(id))
                            .unwrap_or_else(|| Window::get_foreground_window());

                        if win.exists() && !workspace.has_window(win.get_id()) {
                            info!("'{}' managed", win.get_title());

                            let cleanup = window_cleanup.entry(win.get_id()).or_default();
                            let size = win.get_size();
                            let pos = win.get_position();

                            cleanup.reset_transform = Some(Box::new(move || {
                                win.reposition(pos);
                                win.resize(size);
                            }));

                            if config.remove_decorations {
                                cleanup.add_decorations = Some(win.remove_decorations())
                            }

                            call_layout_function(
                                &rt,
                                &mut workspace,
                                String::from("managed"),
                                win.get_id(),
                            );
                        }
                    }
                    WindowAction::Unmanage(maybe_id) => {
                        let maybe_id = maybe_id.or(workspace
                            .get_focused_node()
                            .and_then(|x| x.try_get_window_id()));

                        if let Some(id) = maybe_id {
                            let win = Window::new(id);
                            if workspace.has_window(win.get_id()) {
                                info!("'{}' unmanaged", win.get_title());

                                if config.remove_decorations {
                                    let cleanup = window_cleanup.get(&win.get_id()).expect("If remove_decorations is enabled there has to be some cleanup function");

                                    if let Some(f) = cleanup.add_decorations.as_ref() {
                                        f();
                                    }

                                    if let Some(f) = cleanup.reset_transform.as_ref() {
                                        f();
                                    }
                                }

                                call_layout_function(
                                    &rt,
                                    &mut workspace,
                                    String::from("unmanaged"),
                                    win.get_id(),
                                );
                            }
                        }
                    }
                },
                Action::Workspace(action) => match action {
                    WorkspaceAction::Focus(maybe_id, dir) => {
                        if let Some(id) = workspace.focus_in_direction(dir) {
                            let win_id = workspace
                                .graph
                                .get_node(id)
                                .expect("The returned node has to exist")
                                .try_get_window_id()
                                .expect("The focused node has to be a window node");

                            tx.send(Event::Action(Action::Window(WindowAction::Focus(win_id))))
                                .unwrap();
                        }
                    }
                    WorkspaceAction::Swap(maybe_id, dir) => {
                        if let Some(id) = workspace.focused_node_id {
                            call_layout_function(
                                &rt,
                                &mut workspace,
                                String::from("swapped"),
                                (id, dir),
                            );
                        }
                    }
                },
                Action::UpdateConfig { key, update_fn } => {
                    update_fn.0(&mut config);
                    info!("Updated config property: {:#?}", key);
                }
                Action::ExecuteLua {
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
                Action::CreateKeybinding {
                    mode,
                    key_combination,
                } => {
                    KeybindingEventLoop::add_keybinding(key_combination.get_id());
                    info!("Created {:?} keybinding: {}", mode, key_combination);
                }
                Action::RemoveKeybinding { key } => {
                    // KeybindingEventLoop::remove_keybinding(key_combination.get_id());
                    info!("Removed keybinding: {}", key);
                }
            },
            Event::RenderGraph => {
                render_graph(&workspace.graph);
            }
        }
    }
}
