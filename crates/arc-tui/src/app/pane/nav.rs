//! Shared list navigation. Story and todo panes group their items into ordered
//! sections but navigate across section boundaries as one flat sequence.

/// Step to the next (or previous) id in a flat ordered list, wrapping at the
/// ends. Returns `None` — leave the selection unchanged — when `current` is not
/// present in `ids`.
pub fn step_wrapping<Id: PartialEq + Copy>(ids: &[Id], current: Id, forward: bool) -> Option<Id> {
    let pos = ids.iter().position(|&x| x == current)?;
    let len = ids.len();
    let next = if forward {
        (pos + 1) % len
    } else if pos == 0 {
        len - 1
    } else {
        pos - 1
    };
    Some(ids[next])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_and_bails_on_missing() {
        let ids = [10, 20, 30];
        assert_eq!(step_wrapping(&ids, 10, true), Some(20));
        assert_eq!(step_wrapping(&ids, 30, true), Some(10)); // wrap forward
        assert_eq!(step_wrapping(&ids, 10, false), Some(30)); // wrap back
        assert_eq!(step_wrapping(&ids, 20, false), Some(10));
        assert_eq!(step_wrapping(&ids, 99, true), None); // not found
        assert_eq!(step_wrapping::<i32>(&[], 1, true), None); // empty
    }
}
