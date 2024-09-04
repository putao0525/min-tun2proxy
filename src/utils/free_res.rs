// use std::sync::Arc;
// use log::info;
// use tokio::{signal, task};
// use crate::platform::RouteTable;
//
// pub fn free_resource(route_table: Arc<dyn RouteTable>) -> anyhow::Result<()> {
//     // 创建一个任务来处理信号
//     let signal_task = task::spawn(async {
//         let route_table = Arc::clone(&route_table);
//         let sigint = signal::ctrl_c();
//         let sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap().recv();
//
//         tokio::select! {
//             _ = sigint => {
//                 info!("Received SIGINT, performing cleanup...");
//             },
//             _ = sigterm => {
//                 info!("Received SIGTERM, performing cleanup...");
//             },
//         }
//
//         route_table.free_route_table();
//     });
//
//     tokio::spawn(async {
//         signal_task.await.unwrap();
//     });
//     Ok(())
// }
