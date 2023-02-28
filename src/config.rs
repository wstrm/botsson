use serde::de::{Error as DeError, Unexpected as DeUnexpected, Visitor};
use serde::{self, Deserialize, Deserializer};
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;
use xmpp_parsers::{BareJid as XmppBareJid, Jid as XmppJid};

#[derive(Deserialize)]
pub struct Config {
    pub bot_jid: ConfigXmppJid,
    pub bot_password: String,
    pub bot_nick: String,
    pub muc_jid: ConfigXmppJid,
}

pub struct ConfigXmppJid(XmppJid);

impl Deref for ConfigXmppJid {
    type Target = XmppJid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ConfigXmppJid> for String {
    fn from(cxj: ConfigXmppJid) -> String {
        String::from(cxj.0)
    }
}

impl From<ConfigXmppJid> for XmppBareJid {
    fn from(cxj: ConfigXmppJid) -> XmppBareJid {
        XmppBareJid::from(cxj.0)
    }
}

impl From<ConfigXmppJid> for XmppJid {
    fn from(cxj: ConfigXmppJid) -> XmppJid {
        cxj.0
    }
}

impl fmt::Display for ConfigXmppJid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(&self.0, fmt)
    }
}

impl<'de> Deserialize<'de> for ConfigXmppJid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const EXPECTED_HINT: &str = "jid with the format: name@example.org";

        struct ConfigXmppJidVisitor;

        impl<'de> Visitor<'de> for ConfigXmppJidVisitor {
            type Value = ConfigXmppJid;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(EXPECTED_HINT)
            }

            fn visit_str<E>(self, value: &str) -> Result<ConfigXmppJid, E>
            where
                E: DeError,
            {
                match XmppJid::from_str(value) {
                    Ok(jid) => Ok(ConfigXmppJid(jid)),
                    Err(_) => Err(DeError::invalid_value(
                        DeUnexpected::Str(value),
                        &EXPECTED_HINT,
                    )),
                }
            }
        }

        deserializer.deserialize_str(ConfigXmppJidVisitor)
    }
}
