// Automatically generated rust module for 'p2p_network_floodsub_message.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::borrow::Cow;
use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct RPC<'a> {
    pub subscriptions: Vec<mod_RPC::SubOpts<'a>>,
    pub publish: Vec<Message<'a>>,
}

impl<'a> MessageRead<'a> for RPC<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.subscriptions.push(r.read_message::<mod_RPC::SubOpts>(bytes)?),
                Ok(18) => msg.publish.push(r.read_message::<Message>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for RPC<'a> {
    fn get_size(&self) -> usize {
        0
        + self.subscriptions.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + self.publish.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.subscriptions { w.write_with_tag(10, |w| w.write_message(s))?; }
        for s in &self.publish { w.write_with_tag(18, |w| w.write_message(s))?; }
        Ok(())
    }
}

pub mod mod_RPC {

use std::borrow::Cow;
use super::*;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct SubOpts<'a> {
    pub subscribe: Option<bool>,
    pub topicid: Option<Cow<'a, str>>,
}

impl<'a> MessageRead<'a> for SubOpts<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.subscribe = Some(r.read_bool(bytes)?),
                Ok(18) => msg.topicid = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for SubOpts<'a> {
    fn get_size(&self) -> usize {
        0
        + self.subscribe.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.topicid.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.subscribe { w.write_with_tag(8, |w| w.write_bool(*s))?; }
        if let Some(ref s) = self.topicid { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Message<'a> {
    pub from: Option<Cow<'a, [u8]>>,
    pub data: Option<Cow<'a, [u8]>>,
    pub seqno: Option<Cow<'a, [u8]>>,
    pub topics: Vec<Cow<'a, str>>,
}

impl<'a> MessageRead<'a> for Message<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.from = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(18) => msg.data = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(26) => msg.seqno = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(34) => msg.topics.push(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Message<'a> {
    fn get_size(&self) -> usize {
        0
        + self.from.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.data.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.seqno.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.topics.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.from { w.write_with_tag(10, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.data { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.seqno { w.write_with_tag(26, |w| w.write_bytes(&**s))?; }
        for s in &self.topics { w.write_with_tag(34, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

