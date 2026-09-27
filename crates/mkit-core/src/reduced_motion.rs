//! Synchronize the operating system's reduced-motion preference with GPUI.
//!
//! Native adapters observe macOS and Windows preference changes. Other
//! platforms currently leave the GPUI preference under application control.

#[cfg(any(target_os = "macos", target_os = "windows", test))]
use futures::StreamExt;
use futures::channel::mpsc;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
use gpui_pre::AsyncApp;
use gpui_pre::{App, Task};

/// Active operating-system preference monitor.
///
/// Keep this value alive to follow OS changes. Dropping it stops monitoring and
/// unregisters the native observer.
#[must_use = "retain the monitor guard while the app should follow system motion preferences"]
pub struct SystemReducedMotionMonitor {
    _task: Task<()>,
    stop: mpsc::UnboundedSender<MonitorMessage>,
}

impl Drop for SystemReducedMotionMonitor {
    fn drop(&mut self) {
        let _ = self.stop.unbounded_send(MonitorMessage::Stop);
    }
}

enum MonitorMessage {
    #[cfg(any(target_os = "macos", target_os = "windows", test))]
    Preference(bool),
    Stop,
}

/// Start monitoring the OS reduced-motion preference where supported.
///
/// The returned monitor guard must be retained for as long as the application should
/// follow OS changes. Dropping it stops the platform listener. macOS and
/// Windows read their native system settings. The current setting is applied
/// immediately and later changes update `App::reduce_motion`, which also
/// redraws open windows. Other platforms return `None`.
pub fn watch_system_reduced_motion(cx: &mut App) -> Option<SystemReducedMotionMonitor> {
    #[cfg(target_os = "macos")]
    {
        let initial = macos::read_preference();
        let (sender, receiver) = mpsc::unbounded();
        let observer = macos::Observer::new(sender);
        let stop = observer.sender();
        Some(start_monitor(cx, initial, receiver, stop, observer))
    }

    #[cfg(target_os = "windows")]
    {
        let initial = windows::read_preference().ok()?;
        let (sender, receiver) = mpsc::unbounded();
        let observer = windows::Observer::new(sender).ok()?;
        let stop = observer.sender();
        Some(start_monitor(cx, initial, receiver, stop, observer))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = cx;
        None
    }
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn start_monitor<K: 'static>(
    cx: &mut App,
    initial: bool,
    mut changes: mpsc::UnboundedReceiver<MonitorMessage>,
    stop: mpsc::UnboundedSender<MonitorMessage>,
    keep_alive: K,
) -> SystemReducedMotionMonitor {
    cx.set_reduce_motion(initial);
    let task = cx.spawn(async move |cx: &mut AsyncApp| {
        let _keep_alive = keep_alive;
        while let Some(message) = changes.next().await {
            match message {
                #[cfg(any(target_os = "macos", target_os = "windows", test))]
                MonitorMessage::Preference(reduce_motion) => {
                    cx.update(|app| app.set_reduce_motion(reduce_motion));
                }
                MonitorMessage::Stop => break,
            }
        }
    });
    SystemReducedMotionMonitor { _task: task, stop }
}

#[cfg(target_os = "windows")]
mod windows {
    use futures::channel::mpsc::UnboundedSender;
    use windows::{
        Foundation::TypedEventHandler,
        UI::ViewManagement::{UISettings, UISettingsAnimationsEnabledChangedEventArgs},
    };

    pub(super) fn read_preference() -> windows::core::Result<bool> {
        UISettings::new()?.AnimationsEnabled().map(|enabled| !enabled)
    }

    pub(super) struct Observer {
        settings: UISettings,
        token: i64,
        sender: UnboundedSender<super::MonitorMessage>,
    }

    impl Observer {
        pub(super) fn new(
            sender: UnboundedSender<super::MonitorMessage>,
        ) -> windows::core::Result<Self> {
            let settings = UISettings::new()?;
            let callback_sender = sender.clone();
            let handler = TypedEventHandler::<
                UISettings,
                UISettingsAnimationsEnabledChangedEventArgs,
            >::new(move |settings, _| {
                if let Ok(settings) = settings.ok() {
                    if let Ok(animations_enabled) = settings.AnimationsEnabled() {
                        let _ = callback_sender
                            .unbounded_send(super::MonitorMessage::Preference(!animations_enabled));
                    }
                }
                Ok(())
            });
            let token = settings.AnimationsEnabledChanged(&handler)?;
            Ok(Self { settings, token, sender })
        }

        pub(super) fn sender(&self) -> UnboundedSender<super::MonitorMessage> {
            self.sender.clone()
        }
    }

    impl Drop for Observer {
        fn drop(&mut self) {
            let _ = self.settings.RemoveAnimationsEnabledChanged(self.token);
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::ptr::NonNull;

    use block2::RcBlock;
    use futures::channel::mpsc::UnboundedSender;
    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, ProtocolObject};
    use objc2_app_kit::{NSWorkspace, NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification};
    use objc2_foundation::{
        NSNotification, NSNotificationCenter, NSObjectProtocol, NSOperationQueue,
    };

    pub(super) fn read_preference() -> bool {
        NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion()
    }

    pub(super) struct Observer {
        center: Retained<NSNotificationCenter>,
        token: Retained<ProtocolObject<dyn NSObjectProtocol>>,
        _block: RcBlock<dyn Fn(NonNull<NSNotification>)>,
        sender: UnboundedSender<super::MonitorMessage>,
    }

    impl Observer {
        pub(super) fn new(sender: UnboundedSender<super::MonitorMessage>) -> Self {
            let workspace = NSWorkspace::sharedWorkspace();
            let center = workspace.notificationCenter();
            let callback_sender = sender.clone();
            let block = RcBlock::new(move |_: NonNull<NSNotification>| {
                let _ = callback_sender
                    .unbounded_send(super::MonitorMessage::Preference(read_preference()));
            });
            // AppKit delivers this workspace notification on the main operation
            // queue, where NSWorkspace's preference accessor is valid to call.
            let token = unsafe {
                center.addObserverForName_object_queue_usingBlock(
                    Some(NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification),
                    Some(&*workspace as &AnyObject),
                    Some(&NSOperationQueue::mainQueue()),
                    &block,
                )
            };

            Self { center, token, _block: block, sender }
        }

        pub(super) fn sender(&self) -> UnboundedSender<super::MonitorMessage> {
            self.sender.clone()
        }
    }

    impl Drop for Observer {
        fn drop(&mut self) {
            // SAFETY: `token` is the observer token returned by this center's
            // block-based registration and is still retained here.
            unsafe { self.center.removeObserver(self.token.as_ref()) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::start_monitor;
    use futures::channel::mpsc;
    use gpui_pre::TestAppContext;
    use std::{cell::Cell, rc::Rc};

    struct DropMarker(Rc<Cell<bool>>);

    impl Drop for DropMarker {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    #[gpui_pre::test]
    fn preference_changes_update_the_gpui_motion_flag(cx: &mut TestAppContext) {
        let (sender, receiver) = mpsc::unbounded();
        let dropped = Rc::new(Cell::new(false));
        let stop = sender.clone();
        let task =
            cx.update(|app| start_monitor(app, false, receiver, stop, DropMarker(dropped.clone())));
        assert!(!cx.read(|app| app.reduce_motion()));

        sender.unbounded_send(super::MonitorMessage::Preference(true)).unwrap();
        cx.run_until_parked();
        assert!(cx.read(|app| app.reduce_motion()));

        sender.unbounded_send(super::MonitorMessage::Preference(false)).unwrap();
        cx.run_until_parked();
        assert!(!cx.read(|app| app.reduce_motion()));

        drop(task);
        cx.run_until_parked();
        assert!(dropped.get(), "dropping the monitor must release its observer");
    }
}
