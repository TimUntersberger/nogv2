//! Iced currently does not support toolbars.
//!
//! The `run` function in this file is basically a copy of the `iced_winit` run function except
//! that this one will modify the window so that the window is a toolbar.
//!
//! Anything else inside iced_run and iced_run/state was straight up copied from the github
//! repository and modified so the use statements work.

mod state;

pub use state::State;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use iced_winit::clipboard::Clipboard;
use iced_winit::conversion;
use iced_winit::mouse;
use iced_winit::winit;
use iced_winit::{
    application::{build_user_interface, requests_exit, run_command, update},
    Application, Debug, Error, Executor, Proxy, Runtime, Settings,
};
use windows::Windows::Win32::Foundation::HWND;
use windows::Windows::Win32::UI::KeyboardAndMouseInput::keybd_event;
use windows::Windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows::Windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
use windows::Windows::Win32::UI::WindowsAndMessaging::{
    SetWindowLongW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
};

use iced_futures::futures;
use iced_futures::futures::channel::mpsc;
use iced_graphics::window;
use iced_native::Cache;

use std::mem::ManuallyDrop;

pub fn run<A, E, C>(
    settings: Settings<A::Flags>,
    compositor_settings: C::Settings,
) -> Result<(), Error>
where
    A: Application + 'static,
    E: Executor + 'static,
    C: window::Compositor<Renderer = A::Renderer> + 'static,
{
    use futures::task;
    use futures::Future;
    use winit::event_loop::EventLoop;

    let mut debug = Debug::new();
    debug.startup_started();

    let event_loop = EventLoop::with_user_event();
    let mut proxy = event_loop.create_proxy();

    let mut runtime = {
        let proxy = iced_winit::Proxy::new(event_loop.create_proxy());
        let executor = E::new().map_err(Error::ExecutorCreationFailed)?;

        Runtime::new(executor, proxy)
    };

    let (application, init_command) = {
        let flags = settings.flags;

        runtime.enter(|| A::new(flags))
    };

    let subscription = application.subscription();

    //TODO: platform specific
    let prev_hwnd = unsafe { GetForegroundWindow() };

    let window = settings
        .window
        .into_builder(
            &application.title(),
            application.mode(),
            event_loop.primary_monitor(),
            settings.id,
        )
        .build(&event_loop)
        .map_err(Error::WindowCreationFailed)?;

    match &window.raw_window_handle() {
        RawWindowHandle::Windows(win_hndl) => unsafe {
            let hwnd = HWND(win_hndl.hwnd as isize);
            SetWindowLongW(hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE.0 as i32);
            keybd_event(0, 0, Default::default(), 0);
            SetForegroundWindow(prev_hwnd);
        },
        handle => todo!("not supported yet: {:?}", handle),
    }

    let mut clipboard = Clipboard::connect(&window);

    run_command(
        init_command,
        &mut runtime,
        &mut clipboard,
        &mut proxy,
        &window,
    );

    runtime.track(subscription);

    let (compositor, renderer) = C::new(compositor_settings, Some(&window))?;

    let (mut sender, receiver) = mpsc::unbounded();

    let mut instance = Box::pin(run_instance::<A, E, C>(
        application,
        compositor,
        renderer,
        runtime,
        clipboard,
        proxy,
        debug,
        receiver,
        window,
        settings.exit_on_close_request,
    ));

    let mut context = task::Context::from_waker(task::noop_waker_ref());

    event_loop.run(move |event, _, control_flow| {
        use winit::event_loop::ControlFlow;

        if let ControlFlow::Exit = control_flow {
            return;
        }

        let event = match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                window_id,
            } => Some(winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(*new_inner_size),
                window_id,
            }),
            _ => event.to_static(),
        };

        if let Some(event) = event {
            sender.start_send(event).expect("Send event");

            let poll = instance.as_mut().poll(&mut context);

            *control_flow = match poll {
                task::Poll::Pending => ControlFlow::Wait,
                task::Poll::Ready(_) => ControlFlow::Exit,
            };
        }
    });
}

#[allow(clippy::too_many_arguments)]
async fn run_instance<A, E, C>(
    mut application: A,
    mut compositor: C,
    mut renderer: A::Renderer,
    mut runtime: Runtime<E, Proxy<A::Message>, A::Message>,
    mut clipboard: Clipboard,
    mut proxy: winit::event_loop::EventLoopProxy<A::Message>,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<winit::event::Event<'_, A::Message>>,
    window: winit::window::Window,
    exit_on_close_request: bool,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: window::Compositor<Renderer = A::Renderer> + 'static,
{
    use futures::stream::StreamExt;
    use winit::event;

    let mut surface = compositor.create_surface(&window);

    let mut state = State::new(&application, &window);
    let mut viewport_version = state.viewport_version();

    let physical_size = state.physical_size();

    compositor.configure_surface(&mut surface, physical_size.width, physical_size.height);

    let mut user_interface = ManuallyDrop::new(build_user_interface(
        &mut application,
        Cache::default(),
        &mut renderer,
        state.logical_size(),
        &mut debug,
    ));

    let mut primitive = user_interface.draw(&mut renderer, state.cursor_position());
    let mut mouse_interaction = mouse::Interaction::default();

    let mut events = Vec::new();
    let mut messages = Vec::new();

    debug.startup_finished();

    while let Some(event) = receiver.next().await {
        match event {
            event::Event::MainEventsCleared => {
                if events.is_empty() && messages.is_empty() {
                    continue;
                }

                debug.event_processing_started();

                let statuses = user_interface.update(
                    &events,
                    state.cursor_position(),
                    &renderer,
                    &mut clipboard,
                    &mut messages,
                );

                debug.event_processing_finished();

                for event in events.drain(..).zip(statuses.into_iter()) {
                    runtime.broadcast(event);
                }

                if !messages.is_empty() {
                    let cache = ManuallyDrop::into_inner(user_interface).into_cache();

                    // Update application
                    update(
                        &mut application,
                        &mut runtime,
                        &mut clipboard,
                        &mut proxy,
                        &mut debug,
                        &mut messages,
                        &window,
                    );

                    // Update window
                    state.synchronize(&application, &window);

                    let should_exit = application.should_exit();

                    user_interface = ManuallyDrop::new(build_user_interface(
                        &mut application,
                        cache,
                        &mut renderer,
                        state.logical_size(),
                        &mut debug,
                    ));

                    if should_exit {
                        break;
                    }
                }

                debug.draw_started();
                primitive = user_interface.draw(&mut renderer, state.cursor_position());
                debug.draw_finished();

                window.request_redraw();
            }
            event::Event::PlatformSpecific(event::PlatformSpecific::MacOS(
                event::MacOS::ReceivedUrl(url),
            )) => {
                use iced_native::event;
                events.push(iced_native::Event::PlatformSpecific(
                    event::PlatformSpecific::MacOS(event::MacOS::ReceivedUrl(url)),
                ));
            }
            event::Event::UserEvent(message) => {
                messages.push(message);
            }
            event::Event::RedrawRequested(_) => {
                let physical_size = state.physical_size();

                if physical_size.width == 0 || physical_size.height == 0 {
                    continue;
                }

                debug.render_started();
                let current_viewport_version = state.viewport_version();

                if viewport_version != current_viewport_version {
                    let logical_size = state.logical_size();

                    debug.layout_started();
                    user_interface = ManuallyDrop::new(
                        ManuallyDrop::into_inner(user_interface)
                            .relayout(logical_size, &mut renderer),
                    );
                    debug.layout_finished();

                    debug.draw_started();
                    primitive = user_interface.draw(&mut renderer, state.cursor_position());
                    debug.draw_finished();

                    compositor.configure_surface(
                        &mut surface,
                        physical_size.width,
                        physical_size.height,
                    );

                    viewport_version = current_viewport_version;
                }

                match compositor.draw(
                    &mut renderer,
                    &mut surface,
                    state.viewport(),
                    state.background_color(),
                    &primitive,
                    &debug.overlay(),
                ) {
                    Ok(new_mouse_interaction) => {
                        debug.render_finished();

                        if new_mouse_interaction != mouse_interaction {
                            window.set_cursor_icon(conversion::mouse_interaction(
                                new_mouse_interaction,
                            ));

                            mouse_interaction = new_mouse_interaction;
                        }

                        // TODO: Handle animations!
                        // Maybe we can use `ControlFlow::WaitUntil` for this.
                    }
                    Err(error) => match error {
                        // This is an unrecoverable error.
                        window::SurfaceError::OutOfMemory => {
                            panic!("{}", error);
                        }
                        _ => {
                            debug.render_finished();

                            // Try rendering again next frame.
                            window.request_redraw();
                        }
                    },
                }
            }
            event::Event::WindowEvent {
                event: event::WindowEvent::MenuEntryActivated(entry_id),
                ..
            } => {
                if let Some(message) = conversion::menu_message(state.menu(), entry_id) {
                    messages.push(message);
                }
            }
            event::Event::WindowEvent {
                event: window_event,
                ..
            } => {
                if requests_exit(&window_event, state.modifiers()) && exit_on_close_request {
                    break;
                }

                state.update(&window, &window_event, &mut debug);

                if let Some(event) =
                    conversion::window_event(&window_event, state.scale_factor(), state.modifiers())
                {
                    events.push(event);
                }
            }
            _ => {}
        }
    }

    // Manually drop the user interface
    drop(ManuallyDrop::into_inner(user_interface));
}
