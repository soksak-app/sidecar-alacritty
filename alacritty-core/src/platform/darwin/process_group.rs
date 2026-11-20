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
    Ok(listed(
        PROC_PGRP_ONLY,
        group as u32,
        &format!("process group {group}"),
        &format!("group {group}"),
    )?
    .into_iter()
    .map(|(pid, info)| Member {
        pid,
        uid: info.pbsi_uid,
        command: info
            .pbsi_comm
            .iter()
            .take_while(|byte| **byte != 0)
            .map(|byte| *byte as u8 as char)
            .collect(),
        zombie: info.pbsi_status == libc::SZOMB,
        exiting: info.pbsi_flags & PROC_FLAG_INEXIT != 0,
    })
    .collect())
}

// PROC_TTY_ONLY from sys/proc_info.h. The libc crate does not export this value.
const PROC_TTY_ONLY: u32 = 3;

/// One process whose controlling terminal is the terminal of a session, with its group, its parent
/// and its state (core docs/spec/process-list.md#members).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalMember {
    pub member: crate::process_list::Member,
    pub parent: i32,
}

/// Reads the processes whose controlling terminal is the device `device`. A process reaped during
/// the read is not in the result; another failed read fails the list, because the group and the
/// state of that process cannot be stated (core docs/spec/process-list.md#failures).
pub fn terminal(device: u32) -> Result<Vec<TerminalMember>, String> {
    use crate::process_list::{Member, MemberState};
    Ok(listed(
        PROC_TTY_ONLY,
        device,
        &format!("the processes of terminal {device}"),
        &format!("terminal {device}"),
    )?
    .into_iter()
    .map(|(pid, info)| {
        let state = if info.pbsi_status == libc::SZOMB || info.pbsi_flags & PROC_FLAG_INEXIT != 0 {
            MemberState::Ended
        } else if info.pbsi_status == libc::SSTOP {
            MemberState::Stopped
        } else {
            MemberState::Running
        };
        TerminalMember {
            member: Member {
                pid,
                group: info.pbsi_pgid as i32,
                state,
            },
            parent: info.pbsi_ppid as i32,
        }
    })
    .collect())
}

/// Reads the short BSD information of one process: its group, its parent, its user, its command
/// name and its status. PROC_PIDTBSDINFO answers EPERM when the effective user of the process
/// differs from the effective user of the caller: a login shell runs setuid programs in its group,
/// such as /bin/ps, which `brew shellenv` runs, and a terminal runs setuid programs such as `top`
/// (core docs/spec/process-list.md#members). PROC_PIDT_SHORTBSDINFO makes no check of the user.
/// The arg 1 finds a zombie too.
fn short_info(pid: i32) -> Result<libc::proc_bsdshortinfo, std::io::Error> {
    let mut info: libc::proc_bsdshortinfo = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<libc::proc_bsdshortinfo>() as libc::c_int;
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
        Ok(info)
    } else {
        Err(std::io::Error::last_os_error())
    }
}

/// Reads the BSD information of one process, which holds its group, its status, its start time and
/// the device of its controlling terminal; the system refuses it for a process of another effective
/// user.
pub fn info(pid: i32) -> Result<libc::proc_bsdinfo, std::io::Error> {
    let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
    // PROC_PIDTBSDINFO answers ESRCH for a zombie when arg is 0, and finds the zombie too when arg
    // is 1.
    let read = unsafe {
        libc::proc_pidinfo(
            pid,
            libc::PROC_PIDTBSDINFO,
            1,
            (&mut info as *mut libc::proc_bsdinfo).cast(),
            size,
        )
    };
    if read == size {
        Ok(info)
    } else {
        Err(std::io::Error::last_os_error())
    }
}

/// Lists the pids of the kind `kind` with the value `value` and reads the short BSD information of each;
/// `listing` names the list and `owner` the owner of a process in the errors. A process reaped after
/// the list was read gives no information and is not in the result. Any other failure means the
/// state is unknown.
fn listed(
    kind: u32,
    value: u32,
    listing: &str,
    owner: &str,
) -> Result<Vec<(i32, libc::proc_bsdshortinfo)>, String> {
    let size = unsafe { libc::proc_listpids(kind, value, std::ptr::null_mut(), 0) };
    if size < 0 {
        return Err(format!(
            "list {listing}: {}",
            std::io::Error::last_os_error()
        ));
    }
    // Leaves room for members added between the first call and the second call.
    let capacity = size as usize / std::mem::size_of::<i32>() + 16;
    let mut pids = vec![0i32; capacity];
    let filled = unsafe {
        libc::proc_listpids(
            kind,
            value,
            pids.as_mut_ptr().cast(),
            (capacity * std::mem::size_of::<i32>()) as libc::c_int,
        )
    };
    if filled < 0 {
        return Err(format!(
            "list {listing}: {}",
            std::io::Error::last_os_error()
        ));
    }
    pids.truncate(filled as usize / std::mem::size_of::<i32>());
    let mut listed = Vec::new();
    for pid in pids.into_iter().filter(|pid| *pid > 0) {
        match short_info(pid) {
            Ok(info) => listed.push((pid, info)),
            Err(error) if error.raw_os_error() == Some(libc::ESRCH) => {}
            Err(error) => return Err(format!("read process {pid} of {owner}: {error}")),
        }
    }
    Ok(listed)
}
