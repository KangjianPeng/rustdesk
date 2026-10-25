use super::ffi::*;
use super::Display;
use hbb_common::{libc, log};
use std::{io, ptr, slice, time::{Duration, Instant}};

pub struct Capturer {
    display: Display,
    shmid: i32,
    xcbid: u32,
    buffer: *const u8,

    size: usize,
    saved_raw_data: Vec<u8>, // for faster compare and copy
    stats_started: Instant,
    capture_count: u64,
    changed_count: u64,
    unchanged_count: u64,
}

impl Capturer {
    pub fn new(display: Display) -> io::Result<Capturer> {
        // Calculate dimensions.

        let pixel_width = display.pixfmt().bytes_per_pixel();
        let rect = display.rect();
        let size = (rect.w as usize) * (rect.h as usize) * pixel_width;

        // Create a shared memory segment.

        let shmid = unsafe {
            libc::shmget(
                libc::IPC_PRIVATE,
                size,
                // Everyone can do anything.
                libc::IPC_CREAT | 0o777,
            )
        };

        if shmid == -1 {
            return Err(io::Error::last_os_error());
        }

        // Attach the segment to a readable address.

        let buffer = unsafe { libc::shmat(shmid, ptr::null(), libc::SHM_RDONLY) } as *mut u8;

        if buffer as isize == -1 {
            return Err(io::Error::last_os_error());
        }

        // Attach the segment to XCB.

        let server = display.server().raw();
        let xcbid = unsafe { xcb_generate_id(server) };
        unsafe {
            xcb_shm_attach(
                server,
                xcbid,
                shmid as u32,
                0, // False, i.e. not read-only.
            );
        }

        let c = Capturer {
            display,
            shmid,
            xcbid,
            buffer,
            size,
            saved_raw_data: Vec::new(),
            stats_started: Instant::now(),
            capture_count: 0,
            changed_count: 0,
            unchanged_count: 0,
        };
        Ok(c)
    }

    pub fn display(&self) -> &Display {
        &self.display
    }

    fn get_image(&self) {
        let rect = self.display.rect();
        unsafe {
            let request = xcb_shm_get_image_unchecked(
                self.display.server().raw(),
                self.display.root(),
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                !0,
                XCB_IMAGE_FORMAT_Z_PIXMAP,
                self.xcbid,
                0,
            );
            let response =
                xcb_shm_get_image_reply(self.display.server().raw(), request, ptr::null_mut());
            libc::free(response as *mut _);
        }
    }

    pub fn frame<'b>(&'b mut self) -> std::io::Result<&'b [u8]> {
        self.capture_count += 1;
        self.get_image();
        let result = unsafe { slice::from_raw_parts(self.buffer, self.size) };
        match crate::would_block_if_equal(&mut self.saved_raw_data, result) {
            Ok(()) => self.changed_count += 1,
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                self.unchanged_count += 1;
                self.log_stats();
                return Err(err);
            }
            Err(err) => return Err(err),
        }
        self.log_stats();
        Ok(result)
    }

    fn log_stats(&mut self) {
        if self.stats_started.elapsed() < Duration::from_secs(1) {
            return;
        }
        log::debug!(
            "x11_capture_stats: calls={}, changed={}, unchanged={}",
            self.capture_count,
            self.changed_count,
            self.unchanged_count,
        );
        self.stats_started = Instant::now();
        self.capture_count = 0;
        self.changed_count = 0;
        self.unchanged_count = 0;
    }
}

impl Drop for Capturer {
    fn drop(&mut self) {
        unsafe {
            // Detach segment from XCB.
            xcb_shm_detach(self.display.server().raw(), self.xcbid);
            // Detach segment from our space.
            libc::shmdt(self.buffer as *mut _);
            // Destroy the shared memory segment.
            libc::shmctl(self.shmid, libc::IPC_RMID, ptr::null_mut());
        }
    }
}
