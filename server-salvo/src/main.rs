mod db;
mod points;
//mod transport_h3;
mod channels;
mod mail_send;
mod paths;
mod redis;
mod session;
mod tasks;
mod transport_h2;
mod types;
mod utils;

use std::env;

//use salvo::conn::rustls::{Keycert, RustlsConfig};
use salvo::prelude::*;
use shared::{URL_SSE, URL_TRANSPORT};

use crate::tasks::{mail_watcher, run_tasks, task_init};

#[cfg(target_os = "freebsd")]
const PORT: u16 = 3035;
#[cfg(any(target_os = "linux", target_os = "macos"))]
const PORT: u16 = 8443;
//const PORT: u16 = 443;
//Io(Os { code: 13, kind: PermissionDenied, message: "Permission denied"

#[cfg(target_os = "freebsd")]
const DIST_PATH: &str = "/mnt/data/rs-app-mail-canvas/client-egui/release";
// const DIST_PATH: &str = "client/release";
#[cfg(any(target_os = "linux", target_os = "macos"))]
//const DIST_PATH: &str = "client-iced/release";
const DIST_PATH: &str = "client-egui/dist";

#[tokio::main(worker_threads = 2)]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let cmd = args[1].as_str();
        println!("cmd: {cmd}");
        if cmd == "move" {
            //db_messages_move().await;
            println!("MOVED");
        }
        return;
    }

    // #[cfg(any(target_os = "linux", target_os = "macos"))]
    let with_ansi = cfg!(any(target_os = "linux", target_os = "macos"));
    tracing_subscriber::fmt().with_ansi(with_ansi).init();

    tokio::task::spawn(async {
        task_init().await;
        mail_watcher().await;
    });

    tokio::task::spawn(async {
        run_tasks().await;
    });

    let router = Router::new()
        //.push(Router::with_path(URL_WEB_TRANSPORT).goal(transport_h3::transport_data))
        .push(Router::with_path(URL_SSE).get(transport_h2::transport_sse))
        .push(Router::with_path(URL_TRANSPORT).post(transport_h2::transport_data))
        //.push(Router::with_path(format!("{API_DOWNLOAD}{{id}}")).get(download_file))
        .push(
            Router::with_path("{*path}").get(
                StaticDir::new([DIST_PATH])
                    .defaults("index.html")
                    .fallback("index.html"),
            ),
        );

    let acceptor = TcpListener::new(("0.0.0.0", PORT)).bind().await;
    Server::new(acceptor).serve(router).await;
}
