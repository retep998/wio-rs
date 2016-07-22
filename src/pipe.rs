// Copyright © 2016, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
use handle::{Handle};

pub struct NamedPipe(Handle);
impl NamedPipe {
    //fn create(name: &[u16], access: Access, )
}
pub enum Access {
    Inbound,
    Outbound,
    Duplex,
}