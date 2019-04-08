use std::time::Duration;
use std::sync::mpsc;

use actix::*;
use actix_web::ws::{ClientWriter, Message, ProtocolError};

pub struct ClientActor {
    pub writer: ClientWriter,
    pub channel_sender: mpsc::Sender<String>,
}

#[derive(Message)]
pub struct ClientMessage{
    pub data: String,
}

impl Actor for ClientActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopping(&mut self, _: &mut Context<Self>) -> Running {
        info!("🔎 🚔 disconnected, stopping the System");

        info!("Send StopSystem");
        match self.channel_sender.send("StopSystem".to_owned()) {
            Ok(_) => {},
            Err(e) => error!("Can't send StopSystem: {:?}", e),
        };

        info!("System::current().stop()");
        System::current().stop();

        Running::Stop
    }
}

/// Sending a message to the server.
impl Handler<ClientMessage> for ClientActor {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, ctx: &mut Context<Self>) {
        if msg.data == "StopSystem" {
            info!("🔎 🚔 Handler<ClientMessage> handle(): stopping the System");
            ctx.stop();
        } else {
            let m = format!("{}", msg.data.trim());
            self.writer.text(m);
        }
    }
}

impl ClientActor {
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            act.writer.ping("");
            act.hb(ctx);

            // TODO client should also check for a timeout here, similar to the
            // server code
        });
    }
}

/// Handling incoming messages from the server.
impl StreamHandler<Message, ProtocolError> for ClientActor {
    fn handle(&mut self, msg: Message, _: &mut Context<Self>) {

        match msg {
            Message::Text(text) => {
                match self.channel_sender.send(text) {
                    Ok(x) => x,
                    Err(e) => error!("🔥 Can't send on ClientActor.channel_sender: {:?}", e),
                }
            },
            _ => (),
        }
    }

    fn started(&mut self, _: &mut Context<Self>) {
        info!("📡 Connected");
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        info!("📡 Server disconnected");
        ctx.stop()
    }
}

