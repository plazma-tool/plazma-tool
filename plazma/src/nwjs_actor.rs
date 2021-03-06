use std::time::Duration;

use actix_web::actix::*;
use actix_web::ws;

pub struct NwjsActor {
    pub writer: ws::ClientWriter,
}

#[derive(Message)]
pub struct ClientMessage {
    pub data: String,
}

impl Actor for NwjsActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopping(&mut self, _: &mut Context<Self>) -> Running {
        info!("🔎 🚔 disconnected, stopping the System");
        info!("System::current().stop()");
        System::current().stop();
        Running::Stop
    }
}

/// Sending a message to the server.
impl Handler<ClientMessage> for NwjsActor {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, ctx: &mut Context<Self>) {
        if msg.data == "StopSystem" {
            info!("🔎 🚔 Handler<ClientMessage> handle(): stopping the System");
            ctx.stop();
        } else {
            let m = String::from(msg.data.trim());
            self.writer.text(m);
        }
    }
}

impl NwjsActor {
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
impl StreamHandler<ws::Message, ws::ProtocolError> for NwjsActor {
    fn handle(&mut self, _msg: ws::Message, _ctx: &mut Context<Self>) {
    }

    fn started(&mut self, _: &mut Context<Self>) {
        info!("📡 Connected");
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        info!("📡 Server disconnected");
        ctx.stop()
    }
}
