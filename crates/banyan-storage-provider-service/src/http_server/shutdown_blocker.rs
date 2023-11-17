





use tokio::sync::watch;
use tokio::task::JoinHandle;









/// Follow k8s signal handling rules for these different signals. The order of shutdown events are:
///
/// 1. Pod is set to the "Terminating" state and removed from the endpoints list of all services,
///    new traffic should stop appearing
/// 2. The preStop Hook is executed if configured, can send a command or an http request. Should be
///    implemented if SIGTERM doesn't gracefully shutdown your app. Simultaneously k8s will start
///    issuing endpoint update commands indicating the service should be removed from load
///    balancers.
/// 3. SIGTERM signal is sent to the pod, your service should start shutting down cleanly, service
///    has 30 seconds to perform any clean up, shutdown, and state saving. The service may still
///    receive requests for up to 10 seconds on GKE according to some blog post. This would make
///    sense as the event time needs to propagate through the system and is supported by this quote
///    about service meshes:
///
///    > Since the components might be busy doing something else, there is no guarantee on how
///    > long it will take to remove the IP address from their internal state.
///
///    I've seen recommendations that the readiness probe should start failing here and others
///    reporting that won't do anything. As far as I can tell failing the readiness probe here
///    makes sense and at worse will do nothing.
///
///    It seems that the common recommendation here is to wait for 10-15 seconds in the
///    graceperiod, with readiness failing, then exit
/// 4. If the container doesn't exit on its own after 30 seconds it will receive a SIGKILL which we
///    can't respond to, we just get killed.
///
/// This also handles SIGINT which K8s doesn't issue, those will be coming from users running the
/// server locally and should shut the server down immediately.
pub async fn graceful_shutdown_blocker() -> (JoinHandle<()>, watch::Receiver<()>) {
    use tokio::signal::unix;

    let mut sig_int_handler =
        unix::signal(unix::SignalKind::interrupt()).expect("to be able to install signal handler");
    let mut sig_term_handler =
        unix::signal(unix::SignalKind::terminate()).expect("to be able to install signal handler");

    let (tx, rx) = tokio::sync::watch::channel(());
    let handle = tokio::spawn(async move {
        // TODO: need to follow k8s signal handling rules for these different signals
        tokio::select! {
            _ = sig_int_handler.recv() => tracing::debug!("gracefully exiting on an interrupt signal"),
            _ = sig_term_handler.recv() => tracing::debug!("gracefully exiting on an terminate signal"),
        }

        let _ = tx.send(());
    });

    (handle, rx)
}
