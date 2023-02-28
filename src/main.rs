mod config;

use crossbeam_channel::{unbounded, RecvError, Sender, Receiver};
use env_logger;
use futures::stream::StreamExt;
use futures::SinkExt;
use log::{debug, info, error};
use tokio_xmpp::{AsyncClient as XmppClient, Event as XmppEvent, Packet as XmppPacket};
use std::fs;
use toml;
use xmpp_parsers::presence::{Presence as XmppPresence, Type as XmppPresenceType};
use xmpp_parsers::muc::{Muc as XmppMuc};
use xmpp_parsers::message::{Message as XmppMessage, MessageType as XmppMessageType};
use std::convert::Into;
use xmpp_parsers::{Element as XmppElement, Jid as XmppJid, BareJid as XmppBareJid};
use config::Config;
use std::thread;
use chrono::{DateTime, FixedOffset};

// TODO: Could use something like this for the event loop.
enum Direction {
    Incoming,
    Outoging,
}

// TODO: Could use something like this for the event loop.
enum Event {
    Join{
        account: XmppJid,
        channel: XmppJid,
    },
    Message{
        from: XmppJid,
        to: XmppJid,
        direction: Direction,
        body: String,
        timestamp: DateTime<FixedOffset>,
    },
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("main: started");

    let config = "./config.toml";
    let config: Config = {
        let txt = match fs::read_to_string(&config) {
            Ok(txt) => txt,
            Err(e) => {
                error!("config: cannot read {config}: {:?}", e);
                return;
            }
        };

        match toml::from_str(&txt) {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("config: error reading stream of {config}: {:?}", e);
                return;
            }
        }
    };

    let mut xmpp_client = match XmppClient::new(&String::from(config.bot_jid.to_owned()), &config.bot_password) {
        Ok(client) => client,
        Err(err) => {
            error!("xmpp: cannot connect: {err}");
            return;
        }
    };
    xmpp_client.set_reconnect(true);

    let (mut xmpp_writer, mut xmpp_reader) = xmpp_client.split();

    // TODO: Create an event loop that works with xmpp_reader + xmpp_writer.
    let (event_sender, event_receiver): (Sender<Event>, Receiver<Event>) = unbounded();

    thread::spawn(move || {
        match event_receiver.recv() {
            Ok(e) => {},
            Err(e) => {}
        }
    });

    while let Some(event) = xmpp_reader.next().await {
        match event {
            XmppEvent::Disconnected(e) => {
                error!("xmpp: disconnected: {:?}", e);
                break;
            }
            XmppEvent::Online {
                bound_jid,
                resumed: _resumed,
            } => {
                info!("xmpp: connected");

                let muc_payload = XmppElement::from(XmppMuc::new());
                let muc_jid = XmppBareJid::from(config.muc_jid.to_owned())
                    .with_resource(config.bot_nick.to_owned());

                let bot_jid = bound_jid.to_owned();

                let presence = XmppPresence::new(XmppPresenceType::None)
                    .with_to(muc_jid)
                    .with_from(bot_jid)
                    .with_payloads(vec![muc_payload]);

                if let Err(e) = xmpp_writer.send(XmppPacket::Stanza(presence.into())).await {
                    log::error!("xmpp: cannot send stanza: {}", e);
                };

                info!("xmpp: joined: {}", config.muc_jid);
            },
            XmppEvent::Stanza(stanza) => {
                debug!("xmpp: recv stanza");

                if let Ok(message) = XmppMessage::try_from(stanza.clone()) {
                    info!("xmpp: message: {:?}", message);

                    if message.type_ == XmppMessageType::Groupchat {
                        if let Some((_, body)) = message.get_best_body(vec![]) {
                            if let Some(command) = body.0.strip_prefix(&config.bot_nick) {
                                let command = command.trim_matches(|c: char| c.is_whitespace() || c == ',' || c == ':');
                                info!("xmpp: command: {}", command)
                            }
                        }
                    }
                }
            },
        }
    }

    info!("main: exited");
}
