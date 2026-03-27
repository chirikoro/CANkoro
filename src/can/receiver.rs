/// CAN receive thread management
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam_channel::Sender;

use crate::can::frame::CanFrame;
use crate::vector::channel::CanPort;

/// Manages a CAN receive thread
pub struct CanReceiver {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl CanReceiver {
    /// Start receiving CAN frames on the given port
    /// Frames are sent to the provided channel sender
    pub fn start(
        port: Arc<parking_lot::Mutex<CanPort>>,
        tx: Sender<CanFrame>,
        poll_interval: Duration,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let handle = thread::Builder::new()
            .name("can-receiver".to_string())
            .spawn(move || {
                Self::receive_loop(port, tx, running_clone, poll_interval);
            })
            .expect("Failed to spawn CAN receiver thread");

        Self {
            running,
            handle: Some(handle),
        }
    }

    fn receive_loop(
        port: Arc<parking_lot::Mutex<CanPort>>,
        tx: Sender<CanFrame>,
        running: Arc<AtomicBool>,
        poll_interval: Duration,
    ) {
        #[cfg(target_os = "windows")]
        {
            // On Windows, use WaitForSingleObject for efficient waiting
            let notification_handle = port.lock().notification_handle();
            while running.load(Ordering::Relaxed) {
                // Wait for notification event
                unsafe {
                    winapi::um::synchapi::WaitForSingleObject(
                        notification_handle as *mut _,
                        poll_interval.as_millis() as u32,
                    );
                }

                let port_guard = port.lock();
                if let Ok(frames) = port_guard.receive() {
                    for frame in frames {
                        if tx.send(frame).is_err() {
                            return; // Channel closed
                        }
                    }
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On non-Windows, poll with sleep
            while running.load(Ordering::Relaxed) {
                {
                    let port_guard = port.lock();
                    if let Ok(frames) = port_guard.receive() {
                        for frame in frames {
                            if tx.send(frame).is_err() {
                                return;
                            }
                        }
                    }
                }
                thread::sleep(poll_interval);
            }
        }
    }

    /// Stop the receiver thread
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Drop for CanReceiver {
    fn drop(&mut self) {
        self.stop();
    }
}
