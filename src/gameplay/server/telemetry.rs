//! Bounded public gateway measurements; credentials/URLs/raw headers never enter logs.

/// Server work and read kind, distinct from client queue/admission/HTTP time.
#[derive(Debug,Clone,Default,PartialEq,Eq)]
pub(super) struct Metadata {
    /// Allowlisted server stage plus finite nonnegative millisecond duration.
    pub(super) server_timing:Option<String>,
    /// Explicit current replication versus ongoing event subscription.
    pub(super) read_mode:Option<String>,
}
impl Metadata {
    /// Retain only the two public contract headers, never authorization/cookies.
    pub(super) fn parse(headers:&str)->Self {
        let mut result=Self::default();
        for line in headers.lines() {
            let Some((name,value))=line.split_once(':') else {continue;};let value=value.trim();
            if name.eq_ignore_ascii_case("X-Gameplay-Read-Mode")&&matches!(value,"current"|"stream"){result.read_mode=Some(value.to_owned());}
            if name.eq_ignore_ascii_case("Server-Timing")&&value.len()<=64_usize {
                if let Some((stage,duration))=value.split_once(";dur=") {
                    if matches!(stage,"gameplay"|"snapshot"|"initial_read")&&duration.bytes().all(|byte|byte.is_ascii_digit()||byte==b'.')&&duration.parse::<f64>().is_ok_and(f64::is_finite){result.server_timing=Some(value.to_owned());}
                }
            }
        }
        result
    }
}
