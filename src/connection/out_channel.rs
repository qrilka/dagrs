use std::{collections::HashMap, sync::Arc};

use tokio::sync::{broadcast, mpsc};

use crate::node::node::NodeId;

use super::information_packet::Content;

/// # Output Channels
/// A hash-table mapping `NodeId` to `OutChannel`. In **Dagrs**, each `Node` stores output
/// channels in this map, enabling `Node` to send information packets to other `Node`s.
#[derive(Default)]
pub struct OutChannels(HashMap<NodeId, Arc<OutChannel>>);

impl OutChannels {
    /// Perform a blocking send on the outcoming channel from `NodeId`.
    pub fn blocking_send_to(&self, id: &NodeId, content: Content) -> Result<(), SendErr> {
        match self.get(id) {
            Some(channel) => channel.blocking_send(content),
            None => Err(SendErr::NoSuchChannel),
        }
    }

    /// Perform a asynchronous send on the outcoming channel from `NodeId`.
    pub async fn send_to(&self, id: &NodeId, content: Content) -> Result<(), SendErr> {
        match self.get(id) {
            Some(channel) => channel.send(content).await,
            None => Err(SendErr::NoSuchChannel),
        }
    }

    /// Close the channel by the given `NodeId`, and remove the channel in this map.
    pub fn close(&mut self, id: &NodeId) {
        if let Some(_) = self.get(id) {
            self.0.remove(id);
        }
    }

    fn get(&self, id: &NodeId) -> Option<Arc<OutChannel>> {
        match self.0.get(id) {
            Some(c) => Some(c.clone()),
            None => None,
        }
    }
}

/// # Output Channel
/// Wrapper of senderrs of `tokio::sync::mpsc` and `tokio::sync::broadcast`. **Dagrs** will
/// decide the inner type of channel when building the graph.
/// Learn more about [Tokio Channels](https://tokio.rs/tokio/tutorial/channels).
enum OutChannel {
    /// Sender of a `tokio::sync::mpsc` channel.
    Mpsc(mpsc::Sender<Content>),
    /// Sender of a `tokio::sync::broadcast` channel.
    Bcst(broadcast::Sender<Content>),
}

impl OutChannel {
    /// Perform a blocking send on this channel.
    fn blocking_send(&self, value: Content) -> Result<(), SendErr> {
        match self {
            OutChannel::Mpsc(sender) => match sender.blocking_send(value) {
                Ok(_) => Ok(()),
                Err(e) => Err(SendErr::ClosedChannel(e.0)),
            },
            OutChannel::Bcst(sender) => match sender.send(value) {
                Ok(_) => Ok(()),
                Err(e) => Err(SendErr::ClosedChannel(e.0)),
            },
        }
    }

    /// Perform a asynchronous send on this channel.
    async fn send(&self, value: Content) -> Result<(), SendErr> {
        match self {
            OutChannel::Mpsc(sender) => match sender.send(value).await {
                Ok(_) => Ok(()),
                Err(e) => Err(SendErr::ClosedChannel(e.0)),
            },
            OutChannel::Bcst(sender) => match sender.send(value) {
                Ok(_) => Ok(()),
                Err(e) => Err(SendErr::ClosedChannel(e.0)),
            },
        }
    }
}

/// # Output Channel Error Types
/// - NoSuchChannel: try to get a channel with an invalid `NodeId`.
/// - ClosedChannel: the channel is closed alredy.
///
/// In cases of getting errs of type `MpscError` and `BcstError`, the sender
/// will find there are no active receivers left, so try to send messages is
/// meaningless for now.
#[derive(Debug)]
pub enum SendErr {
    NoSuchChannel,
    ClosedChannel(Content),
}