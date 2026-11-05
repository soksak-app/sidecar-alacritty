//! Reads the members of a process group and their state.

/// One member of a process group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub pid: i32,
    /// The effective user of the process. A member of another user, such as a setuid program,
    /// refuses signals while it runs.
    pub uid: u32,
    /// The name of the executable, up to 16 bytes.
    pub command: String,
    /// A process that has ended and is not reaped yet.
    pub zombie: bool,
    /// A process that is handling exit(). For example, it waits while a terminal with unread output
    /// left is closed.
    pub exiting: bool,
}

impl Member {
    /// A member with nothing left to end. macOS answers a signal to a group that holds only such
    /// members with EPERM.
    pub fn ended(&self) -> bool {
        self.zombie || self.exiting
    }
}

// PROC_FLAG_INEXIT from sys/proc_info.h. The libc crate does not export this value.
const PROC_FLAG_INEXIT: u32 = 4;

// PROC_PGRP_ONLY from sys/proc_info.h. The libc crate does not export this value.
const PROC_PGRP_ONLY: u32 = 2;

/// Reads the members and state of a process group. A process reaped during the read is not in the
/// result.
pub fn members(group: i32) -> Result<Vec<Member>, String> {
    let size =
        unsafe { libc::proc_listpids(PROC_PGRP_ONLY, group as u32, std::ptr::null_mut(), 0) };
    if size < 0 {
        return Err(format!(
            "list process group {group}: {}",
            std::io::Error::last_os_error()
        ));
    }
    // Leaves room for members added between the first call and the second call.
    let capacity = size as usize / std::mem::size_of::<i32>() + 16;
    let mut pids = vec![0i32; capacity];
    let filled = unsafe {
        libc::proc_listpids(
            PROC_PGRP_ONLY,
            group as u32,
            pids.as_mut_ptr().cast(),
            (capacity * std::mem::size_of::<i32>()) as libc::c_int,
        )
    };
    if filled < 0 {
        return Err(format!(
            "list process group {group}: {}",
            std::io::Error::last_os_error()
        ));
    }
    pids.truncate(filled as usize / std::mem::size_of::<i32>());
    let mut members = Vec::new();
    for pid in pids.into_iter().filter(|pid| *pid > 0) {
        let mut info: libc::proc_bsdshortinfo = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of::<libc::proc_bsdshortinfo>() as libc::c_int;
        // PROC_PIDTBSDINFO answers EPERM when the effective user of the process differs from the
        // effective user of the caller. A login shell runs setuid programs in its group, such as
        // /bin/ps, which `brew shellenv` runs. PROC_PIDT_SHORTBSDINFO makes no check of the user.
        // The arg 1 finds a zombie too.
        let read = unsafe {
            libc::proc_pidinfo(
                pid,
                libc::PROC_PIDT_SHORTBSDINFO,
                1,
                (&mut info as *mut libc::proc_bsdshortinfo).cast(),
                size,
            )
        };
        if read == size {
            let command = info
                .pbsi_comm
                .iter()
                .take_while(|byte| **byte != 0)
                .map(|byte| *byte as u8 as char)
                .collect();
            members.push(Member {
                pid,
                uid: info.pbsi_uid,
                command,
                zombie: info.pbsi_status == libc::SZOMB,
                exiting: info.pbsi_flags & PROC_FLAG_INEXIT != 0,
            });
            continue;
        }
        // A process reaped after the list was read gives no information. Any other failure means
        // the state is unknown.
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(format!("read process {pid} of group {group}: {error}"));
        }
    }
    Ok(members)
}
