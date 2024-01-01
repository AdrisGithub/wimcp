use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use wbdl::Date;
use wimcm::{ADDRESS, DOUBLE_COLON, PORT, WIMCError, WIMCOutput};
use wimcm::presets::{cleanup, echo, get, ping, query, remove, store};
use wjp::{Deserialize, Serialize, Values};

pub struct Provider;

impl Provider {
    fn stream() -> Result<TcpStream, WIMCError> {
        TcpStream::connect(format!("{}{}{}", ADDRESS, DOUBLE_COLON, PORT)).map_err(|_err| WIMCError)
    }
    pub fn store<T: Serialize>(
        ser: T,
        till: Option<Date>,
        params: Vec<&str>,
    ) -> Result<u128, WIMCError> {
        let params = params.iter().map(|&v| String::from(v)).collect();
        Self::stream()
            .map(|mut stream| stream.write_ser(store(ser, params, till)).map(|_| stream))?
            .map(|mut stream| stream.read_ser())?
            .map(|out: WIMCOutput| out.map_ok(u128::try_from))??
            .map_err(|_err| WIMCError)
    }
    pub fn echo(msg: &str) -> Result<String, WIMCError> {
        Self::stream()
            .map(|mut stream| stream.write_ser(echo(msg)).map(|_| stream))?
            .map(|mut stream| stream.read_ser())?
            .map(|out: WIMCOutput| out.map_ok(String::try_from))??
            .map_err(|_err| WIMCError)
    }
    pub fn ping() -> bool {
        Self::internal_ping().is_ok()
    }
    fn internal_ping() -> Result<bool, WIMCError> {
        Ok(Self::stream()
            .map(|mut stream| stream.write_ser(ping()).map(|_| stream))?
            .map(|mut stream| stream.read_ser())?
            .map(|val: WIMCOutput| val.is_okay())
            .is_ok_and(|val| val))
    }
    pub fn get<T: Deserialize>(id: u128) -> Result<T, WIMCError> {
        Self::stream()
            .map(|mut stream| stream.write_ser(get(id)).map(|_| stream))?
            .map(|mut stream| stream.read_ser())?
            .map(|out: WIMCOutput| out.map_ok(|val| T::try_from(val)))??
            .map_err(|_err| WIMCError)
    }
    pub fn query<T: Deserialize>(vec: Vec<&str>) -> Result<Vec<T>, WIMCError> {
        Ok(Self::query_raw(vec)?
            .into_iter()
            .flat_map(|val: Values|
                T::try_from(val).map_err(|_err| WIMCError)
            )
            .collect::<Vec<T>>())
    }
    pub fn query_raw(vec: Vec<&str>) -> Result<Vec<Values>, WIMCError> {
        let vec = vec.iter().map(|&v| String::from(v)).collect();
        Self::stream()
            .map(|mut stream| stream.write_ser(query(vec)).map(|_| stream))?
            .map(|mut stream| stream.read_ser())?
            .map(|out: WIMCOutput| out.map_ok(Vec::try_from))??
            .map_err(|_err| WIMCError)
            .map_err(|_err| WIMCError)
    }
    pub fn remove(id: u128) -> Result<(), WIMCError> {
        Self::stream().map(|mut stream| stream.write_ser(remove(id)))?
    }
    pub fn cleanup() -> Result<(), WIMCError> {
        Self::stream().map(|mut stream| stream.write_ser(cleanup()))?
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

#[cfg(test)]
mod tests {
    use wbdl::Date;

    use crate::Provider;

    #[test]
    pub fn test() {
        println!("{:?}", Provider::store("Hello", Some(Date::now_unchecked().add_year()), vec!["Hello"]));

        println!("{:?}", Provider::get::<String>(1));

        println!("{:?}", Provider::get::<String>(1));
    }
    #[test]
    pub fn remove() {
        println!("{:?}", Provider::store("Hello", Some(Date::now_unchecked().add_year()), vec!["Hello"]));
        println!("{:?}", Provider::get::<String>(0));
        println!("{:?}", Provider::remove(0));
        println!("{:?}", Provider::get::<String>(0));
    }
    #[test]
    pub fn query() {
        println!("{:?}", Provider::query::<String>(vec!["Hello"]));
    }

    #[test]
    pub fn echo() {
        println!("{:?}", Provider::echo("Hello"));
    }

    #[test]
    pub fn ping() {
        println!("{:?}", Provider::ping());
    }
}
