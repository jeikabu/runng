extern crate thrift;

use thrift::{
    protocol::{
        TInputProtocol,
        TOutputProtocol,
        TMessageIdentifier,
        TStructIdentifier,
        TFieldIdentifier,
        TSetIdentifier,
        TListIdentifier,
        TMapIdentifier,
    },
    transport::{
        TReadTransport,
        TWriteTransport,
    }
};

use super::*;

pub struct TNngInputProtocol<T>
where
    T: TReadTransport
{
    protocol: thrift::protocol::TBinaryInputProtocol<T>,
}

impl<T> TNngInputProtocol<T>
where
    T: TReadTransport
{
    pub fn new (transport: T) -> TNngInputProtocol<T> {
        TNngInputProtocol {
            protocol: thrift::protocol::TBinaryInputProtocol::new(transport, false),
        }
    }
}

impl<T> TInputProtocol for TNngInputProtocol<T>
where
    T: TReadTransport
{
    fn read_message_begin(&mut self) -> thrift::Result<TMessageIdentifier> {
        self.protocol.read_message_begin()
    }

    fn read_message_end(&mut self) -> thrift::Result<()> {
        self.protocol.read_message_end()
    }

    fn read_struct_begin(&mut self) -> thrift::Result<Option<TStructIdentifier>> {
        self.protocol.read_struct_begin()
    }

    fn read_struct_end(&mut self) -> thrift::Result<()> {
        self.protocol.read_struct_end()
    }

    fn read_field_begin(&mut self) -> thrift::Result<TFieldIdentifier> {
        self.protocol.read_field_begin()
    }

    fn read_field_end(&mut self) -> thrift::Result<()> {
        self.protocol.read_field_end()
    }

    fn read_bool(&mut self) -> thrift::Result<bool> {
        self.protocol.read_bool()
    }

    fn read_bytes(&mut self) -> thrift::Result<Vec<u8>> {
        self.protocol.read_bytes()
    }

    fn read_i8(&mut self) -> thrift::Result<i8> {
        self.protocol.read_i8()
    }

    fn read_i16(&mut self) -> thrift::Result<i16> {
        self.protocol.read_i16()
    }

    fn read_i32(&mut self) -> thrift::Result<i32> {
        self.protocol.read_i32()
    }

    fn read_i64(&mut self) -> thrift::Result<i64> {
        self.protocol.read_i64()
    }

    fn read_double(&mut self) -> thrift::Result<f64> {
        self.protocol.read_double()
    }

    fn read_string(&mut self) -> thrift::Result<String> {
        self.protocol.read_string()
    }

    fn read_list_begin(&mut self) -> thrift::Result<TListIdentifier> {
        self.protocol.read_list_begin()
    }

    fn read_list_end(&mut self) -> thrift::Result<()> {
        self.protocol.read_list_end()
    }

    fn read_set_begin(&mut self) -> thrift::Result<TSetIdentifier> {
        self.protocol.read_set_begin()
    }

    fn read_set_end(&mut self) -> thrift::Result<()> {
        self.protocol.read_set_end()
    }

    fn read_map_begin(&mut self) -> thrift::Result<TMapIdentifier> {
        self.protocol.read_map_begin()
    }

    fn read_map_end(&mut self) -> thrift::Result<()> {
        self.protocol.read_map_end()
    }

    fn read_byte(&mut self) -> thrift::Result<u8> {
        self.protocol.read_byte()
    }
}

pub struct TNngOutputProtocol<T>
where
    T: TWriteTransport
{
    protocol: thrift::protocol::TBinaryOutputProtocol<T>,
}

impl<T> TNngOutputProtocol<T>
where
    T: TWriteTransport
{
    pub fn new(transport: T) -> TNngOutputProtocol<T> {
        TNngOutputProtocol {
            protocol: thrift::protocol::TBinaryOutputProtocol::new(transport, false),
        }
    }
}

impl<T> TOutputProtocol for TNngOutputProtocol<T>
where
    T: TWriteTransport
{

    fn write_message_begin(&mut self, identifier: &TMessageIdentifier) -> thrift::Result<()> {
        self.protocol.write_message_begin(identifier)
    }

    fn write_message_end(&mut self) -> thrift::Result<()> {
        self.protocol.write_message_end()
    }

    fn write_struct_begin(&mut self, identifier: &TStructIdentifier) -> thrift::Result<()> {
        self.protocol.write_struct_begin(identifier)
    }

    fn write_struct_end(&mut self) -> thrift::Result<()> {
        self.protocol.write_struct_end()
    }

    fn write_field_begin(&mut self, identifier: &TFieldIdentifier) -> thrift::Result<()> {
        self.protocol.write_field_begin(identifier)
    }

    fn write_field_end(&mut self) -> thrift::Result<()> {
        self.protocol.write_field_end()
    }

    fn write_field_stop(&mut self) -> thrift::Result<()> {
        self.protocol.write_field_stop()
    }

    fn write_bool(&mut self, b: bool) -> thrift::Result<()> {
        self.protocol.write_bool(b)
    }

    fn write_bytes(&mut self, b: &[u8]) -> thrift::Result<()> {
        self.protocol.write_bytes(b)
    }

    fn write_i8(&mut self, i: i8) -> thrift::Result<()> {
        self.protocol.write_i8(i)
    }

    fn write_i16(&mut self, i: i16) -> thrift::Result<()> {
        self.protocol.write_i16(i)
    }

    fn write_i32(&mut self, i: i32) -> thrift::Result<()> {
        self.protocol.write_i32(i)
    }

    fn write_i64(&mut self, i: i64) -> thrift::Result<()> {
        self.protocol.write_i64(i)
    }

    fn write_double(&mut self, d: f64) -> thrift::Result<()> {
        self.protocol.write_double(d)
    }

    fn write_string(&mut self, s: &str) -> thrift::Result<()> {
        self.protocol.write_string(s)
    }

    fn write_list_begin(&mut self, identifier: &TListIdentifier) -> thrift::Result<()> {
        self.protocol.write_list_begin(identifier)
    }

    fn write_list_end(&mut self) -> thrift::Result<()> {
        self.protocol.write_list_end()
    }

    fn write_set_begin(&mut self, identifier: &TSetIdentifier) -> thrift::Result<()> {
        self.protocol.write_set_begin(identifier)
    }

    fn write_set_end(&mut self) -> thrift::Result<()> {
        self.protocol.write_set_end()
    }

    fn write_map_begin(&mut self, identifier: &TMapIdentifier) -> thrift::Result<()> {
        self.protocol.write_map_begin(identifier)
    }

    fn write_map_end(&mut self) -> thrift::Result<()> {
        self.protocol.write_map_end()
    }

    fn flush(&mut self) -> thrift::Result<()> {
        self.protocol.flush()
    }

    fn write_byte(&mut self, b: u8) -> thrift::Result<()> {
        self.protocol.write_byte(b)
    }
}
