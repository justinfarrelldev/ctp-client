//! Worker-owned IO. SDL and authoritative render state remain on the UI thread.
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool,Ordering};
use std::sync::mpsc::{self,Receiver,SyncSender};
use std::thread;
use std::time::Duration;
use crate::gameplay::server::{ServerClient,ServerGameplaySession,ServerProjection,envelope::PreparedCommand};

/// Worker messages contain complete viewer state, never canonical patches.
#[derive(Debug)]
pub(super) enum Outcome {
    /// Validated command receipt or current-resync snapshot.
    Snapshot(Box<ServerProjection>),
    /// Acknowledges the one submitted user operation, not a background read.
    Receipt(Box<ServerProjection>),
    /// A discrepancy requires explicit resynchronizing presentation.
    Recovering(String),
    /// Current read failed; input stays blocked until a later successful read.
    Failure(String),
}

/// Injectable service boundary used by pure retry/recovery tests.
pub(super) trait Transport {
    /// Submit precisely the captured operation, without regenerating its key.
    fn submit(&mut self,command:&PreparedCommand)->Result<ServerProjection,String>;
    /// Obtain the complete current authorized viewer model.
    fn resync(&mut self)->Result<ServerProjection,String>;
}
/// Production transport; owns no SDL resource.
struct HttpTransport {
    /// Endpoint and request settings.
    client:ServerClient,
    /// Game membership and session IDs.
    session:ServerGameplaySession,
}
impl Transport for HttpTransport {
    fn submit(&mut self,command:&PreparedCommand)->Result<ServerProjection,String> {
        command.execute(&self.client).map_err(|error| error.to_string())?.viewer_projection.ok_or_else(|| "receipt has no viewer projection".to_owned())
    }
    fn resync(&mut self)->Result<ServerProjection,String> {
        self.client.fetch_resync_projection(self.session.game_id,self.session.session_id).map_err(|error| error.to_string())
    }
}

/// One command's retry/recovery policy, independent of any thread or real service.
pub(super) fn execute(transport:&mut impl Transport, command:&PreparedCommand, emit:&mut impl FnMut(Outcome)) {
    let mut result=transport.submit(command);
    if result.as_ref().is_err_and(|error| error.contains("failed to read") || error.contains("failed to write") || error.contains("could not connect")) {
        // The first execution may have committed. Reuse the exact body/key.
        result=transport.submit(command);
    }
    match result {
        Ok(snapshot)=>emit(Outcome::Receipt(Box::new(snapshot))),
        Err(error)=>{
            emit(Outcome::Recovering(error));
            refresh(transport,emit);
        }
    }
}

/// Full current-model recovery, never a local fallback or partial event patch.
fn refresh(transport:&mut impl Transport, emit:&mut impl FnMut(Outcome)) {
    match transport.resync() {
        Ok(snapshot)=>emit(Outcome::Snapshot(Box::new(snapshot))),
        Err(error)=>emit(Outcome::Failure(error)),
    }
}

/// Bounded worker channels; at most one user operation is outstanding.
#[derive(Debug)]
pub(super) struct Network {
    /// Exactly captured jobs; bounded to avoid input flooding.
    jobs:SyncSender<PreparedCommand>,
    /// Complete results/recovery states consumed by the UI thread.
    results:Receiver<Outcome>,
    /// Cooperative cancellation; read timeouts bound stream shutdown latency.
    stop:Arc<AtomicBool>,
}
impl Network {
    /// Starts independent command/current-read and invalidation workers.
    pub(super) fn start(client:ServerClient,session:ServerGameplaySession)->Self {
        let (jobs,requests)=mpsc::sync_channel::<PreparedCommand>(1);
        let (result_sink,results)=mpsc::sync_channel::<Outcome>(8);
        let (invalid_sink,invalidations)=mpsc::sync_channel::<u64>(1);
        let stop=Arc::new(AtomicBool::new(false));
        let watcher_stop=Arc::clone(&stop);
        let watcher_client=client.clone();
        let watcher_session=session.clone();
        let _watcher=thread::spawn(move || {
            let mut cursor=0_u64;
            while !watcher_stop.load(Ordering::Relaxed) {
                if let Err(error)=crate::gameplay::server::stream::watch(&watcher_client,&watcher_session,&mut cursor,&watcher_stop,&invalid_sink) { log::warn!("Gameplay stream reconnect: {error}"); }
                // Also refresh after disconnect; this recovers missed/expired events.
                let _invalidation=invalid_sink.try_send(cursor);
                for _ in 0_u8..4_u8 { if watcher_stop.load(Ordering::Relaxed) { break; } thread::sleep(Duration::from_millis(100)); }
            }
        });
        let command_stop=Arc::clone(&stop);
        let _commands=thread::spawn(move || {
            let mut transport=HttpTransport {client,session};
            let mut needs_recovery=false;
            while !command_stop.load(Ordering::Relaxed) {
                let recovering=needs_recovery;
                let mut emit=|outcome| {
                    needs_recovery=matches!(outcome,Outcome::Failure(_));
                    // UI departure disconnects the receiver, so a worker cannot
                    // retain/mutate a later SDL session or block shutdown forever.
                    let _delivery=result_sink.send(outcome);
                };
                match requests.recv_timeout(Duration::from_millis(100)) {
                    Ok(command)=>execute(&mut transport,&command,&mut emit),
                    Err(mpsc::RecvTimeoutError::Disconnected)=>break,
                    Err(mpsc::RecvTimeoutError::Timeout)=>{
                        if invalidations.try_recv().is_ok() || recovering {
                            // Coalesce wakeups; current read covers every newer event.
                            while invalidations.try_recv().is_ok() {}
                            refresh(&mut transport,&mut emit);
                        }
                    }
                }
            }
        });
        Self {jobs,results,stop}
    }

    /// Nonblocking enqueue; caller disables further authoritative input until receipt.
    pub(super) fn submit(&self,command:PreparedCommand)->Result<(),String> {
        self.jobs.try_send(command).map_err(|error| error.to_string())
    }

    /// One result per frame leaves recovery presentation visible before replacement.
    pub(super) fn poll(&self)->Option<Outcome> {
        self.results.try_recv().ok()
    }
}
impl Drop for Network {
    fn drop(&mut self) {
        self.stop.store(true,Ordering::Relaxed);
        // Workers hold only owned transport data; bounded socket timeouts and
        // receiver disconnection release them without joining from the SDL thread.
    }
}
