use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use wbdl::Date;
use wimcm::presets::{cleanup, echo, get, ping, query, store};
use wimcm::WIMCMethods::Remove;
use wimcm::{WIMCError, WIMCInput, WIMCOutput};
use wjp::{Deserialize, Serialize};

use crate::r#const::{ADDRESS, DOUBLE_COLON, PORT};

mod r#const;
pub struct Provider;

impl Provider {
    fn stream() -> Result<TcpStream, WIMCError> {
        TcpStream::connect(format!("{}{}{}", ADDRESS, DOUBLE_COLON, PORT)).map_err(|_err| WIMCError)
    }
    pub fn store<T: Serialize>(
        ser: T,
        till: Option<Date>,
        params: Vec<String>,
    ) -> Result<u128, WIMCError> {
        Self::stream()
            .map(|mut stream| {
                let _ = stream.write_ser(store(ser, params, till));
                stream
            })
            .map(|mut stream| {
                let out: Result<WIMCOutput, WIMCError> = stream.read_ser();
                out
            })?
            .map(|val| val.map_ok(u128::try_from))??
            .map_err(|_err| WIMCError)
    }
    pub fn echo(msg: &str) -> Result<String, WIMCError> {
        Self::stream()
            .map(|mut stream| {
                let _ = stream.write_ser(echo(msg));
                stream
            })
            .map(|mut stream| {
                let out: Result<WIMCOutput, WIMCError> = stream.read_ser();
                out
            })?
            .map(|out| out.map_ok(String::try_from))??
            .map_err(|_err| WIMCError)
    }
    pub fn ping() -> bool {
        Self::stream()
            .map(|mut stream| stream.write_ser(ping()))
            .map(|val| val.is_ok())
            .is_ok_and(|val| val)
    }
    pub fn get<T: Deserialize>(id: u128) -> Result<T, WIMCError> {
        Self::stream()
            .map(|mut stream| {
                let _ = stream.write_ser(get(id as usize));
                stream
            })
            .map(|mut stream| {
                let out: Result<WIMCOutput, WIMCError> = stream.read_ser();
                out
            })?
            .map(|val| val.map_ok(|val| T::try_from(val)))??
            .map_err(|_err| WIMCError)
    }
    pub fn query<T: Deserialize>(vec: Vec<String>) -> Result<Vec<T>, WIMCError> {
        Self::stream()
            .map(|mut stream| {
                let _ = stream.write_ser(query(vec));
                stream
            })
            .map(|mut stream| {
                let out: Result<WIMCOutput, WIMCError> = stream.read_ser();
                out
            })?
            .map(|val| val.map_ok(Vec::try_from))??
            .map_err(|_err| WIMCError)
    }
    pub fn remove(id: u128) -> Result<(), WIMCError> {
        Self::stream().map(|mut stream| stream.write_ser(Self::rm(id)))?
    }
    pub fn cleanup() -> Result<(), WIMCError> {
        Self::stream().map(|mut stream| stream.write_ser(cleanup()))?
    }
    fn rm(id: u128) -> WIMCInput {
        WIMCInput::default()
            .set_method(Remove)
            .set_payload(id.serialize())
            .clone()
    }
}

trait Readwrite {
    fn write_ser<T: Serialize>(&mut self, obj: T) -> Result<(), WIMCError>;
    fn read_ser<T: Deserialize>(&mut self) -> Result<T, WIMCError>;
}

impl Readwrite for TcpStream {
    fn write_ser<T: Serialize>(&mut self, obj: T) -> Result<(), WIMCError> {
        self.write_all(obj.json().as_bytes())
            .map_err(|_err| WIMCError)
    }
    fn read_ser<T: Deserialize>(&mut self) -> Result<T, WIMCError> {
        T::deserialize_str(
            String::from_utf8(
                BufReader::new(self)
                    .fill_buf()
                    .map(|s| s.to_vec())
                    .map_err(|_err| WIMCError)?,
            )
            .map_err(|_err| WIMCError)?
            .as_str(),
        )
        .map_err(|_err| WIMCError)
    }
}
