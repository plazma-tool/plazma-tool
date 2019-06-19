use std::time::Duration;

use actix_web::actix::*;
use actix_web::ws;

use web_view;

pub struct WebviewActor {
    pub writer: ws::ClientWriter,
    pub webview_handle: web_view::Handle<()>,
}

#[derive(Message)]
pub struct ClientMessage {
    pub data: String,
}

impl Actor for WebviewActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopping(&mut self, _: &mut Context<Self>) -> Running {
        info!("🔎 🚔 disconnected, terminating the Webview and stopping the System");

        let res = self.webview_handle.dispatch(move |webview| {
            info!("💬 webview dispatch: server disconnected, terminating.");
            webview.terminate();
            Ok(())
        });

        match res {
            Ok(_) => {}
            Err(e) => error!("🔥 webview_handle.dispatch() {:?}", e),
        }

        info!("System::current().stop()");
        System::current().stop();
        Running::Stop
    }
}

/// Sending a message to the server.
impl Handler<ClientMessage> for WebviewActor {
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

impl WebviewActor {
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
impl StreamHandler<ws::Message, ws::ProtocolError> for WebviewActor {
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
