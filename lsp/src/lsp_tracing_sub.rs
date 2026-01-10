// use tracing_subscriber::fmt::{FormatEvent, format::Full};
//
// use std::io::BufWriter;
//
// use tower_lsp_server::{Client, ls_types::MessageType};
// use tracing::Subscriber;
// use tracing_subscriber::Layer;
//
// pub struct LspLayer {
//     client: Client,
// }
//
// impl LspLayer {
//     pub fn new(client: Client) -> Self {
//         Self { client }
//     }
// }
//
// fn level_to_message_type(level: &tracing::Level) -> MessageType {
//     match *level {
//         tracing::Level::ERROR => MessageType::ERROR,
//         tracing::Level::WARN => MessageType::WARNING,
//         tracing::Level::INFO => MessageType::INFO,
//         _ => MessageType::LOG,
//     }
// }
//
// impl<S: Subscriber> Layer<S> for LspLayer {
//     fn on_event(
//         &self,
//         event: &tracing::Event<'_>,
//         _ctx: tracing_subscriber::layer::Context<'_, S>,
//     ) {
//         self.client.log_messageas
//         let message_type = level_to_message_type(event.metadata().level());
//         let client = self.client.clone();
//
//         tokio::spawn(async move {
//             let _ = client.log_message(message_type, msg).await;
//         });
//     }
// }
