use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, SystemTime};

use multimap::MultiMap;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;

pub struct RemoteAddress(IpAddr);

impl RemoteAddress {
    pub fn ip(&self) -> IpAddr {
        self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for RemoteAddress {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        return if let Some(addr) = request.client_ip() {
            Outcome::Success(RemoteAddress { 0: addr })
        } else {
            Outcome::Forward(())
        };
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BotInstance {
    pub address: SocketAddr,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AddressEntry {
    address: SocketAddr,
    updated: SystemTime,
    name: String,
}

impl AddressEntry {
    fn new(address: SocketAddr, name: String) -> AddressEntry {
        AddressEntry {
            address,
            updated: SystemTime::now(),
            name,
        }
    }

    pub fn is_entry_valid(&self, ttl: Duration) -> bool {
        if let Ok(t) = self.updated.elapsed() {
            return t < ttl;
        }
        return false;
    }
}

impl PartialEq for AddressEntry {
    fn eq(&self, other: &Self) -> bool {
        self.address.eq(&other.address)
    }
}

impl PartialEq<SocketAddr> for AddressEntry {
    fn eq(&self, other: &SocketAddr) -> bool {
        self.address.eq(other)
    }
}

#[derive(Debug)]
pub struct Registry {
    multimap: MultiMap<IpAddr, AddressEntry>,
    ttl: Duration,
}

impl Registry {
    pub fn create(capacity: usize, ttl: Duration) -> Registry {
        Registry {
            multimap: MultiMap::with_capacity(capacity),
            ttl,
        }
    }

    fn insert_unchecked(&mut self, key: IpAddr, value: SocketAddr, name: String) -> () {
        if let Some(vec) = self.multimap.get_vec_mut(&key) {
            match { vec.iter().position(|e| e == &value) } {
                Some(pos) => {
                    if let Some(entry) = vec.get_mut(pos) {
                        entry.updated = SystemTime::now()
                    }
                }
                None => vec.push(AddressEntry::new(value, name)),
            }
        } else {
            self.multimap.insert(key, AddressEntry::new(value, name));
        }
    }

    pub fn insert_struct(&mut self, key: IpAddr, instance: BotInstance) -> bool {
        self.insert(key, instance.address, instance.name)
    }

    pub fn insert(&mut self, key: IpAddr, value: SocketAddr, name: String) -> bool {
        if self.multimap.len() < self.multimap.capacity() {
            self.insert_unchecked(key, value, name);
            return true;
        } else {
            self.clean();
            if self.multimap.len() < self.multimap.capacity() {
                self.insert_unchecked(key, value, name);
                return true;
            }
        }
        return false;
    }

    pub fn get(&self, key: &IpAddr) -> Option<(Vec<AddressEntry>, bool)> {
        let ttl = self.ttl;
        let result = self.multimap.get_vec(&key);
        return match result {
            Some(vec) => {
                let size = vec.len();
                let cleaned: Vec<AddressEntry> = vec
                    .iter()
                    .filter(|item| item.is_entry_valid(ttl))
                    .cloned()
                    .collect();
                let cleaned_size = cleaned.len();
                Some((cleaned, size != cleaned_size))
            }
            _ => None,
        };
    }

    pub fn clean_key(&mut self, key: &IpAddr) {
        let ttl = self.ttl;
        let opt_vec = self.multimap.get_vec_mut(&key);
        if let Some(vec) = opt_vec {
            let mut i = 0;
            while i != vec.len() {
                if !(&mut vec[i]).is_entry_valid(ttl) {
                    vec.remove(i);
                } else {
                    i += 1;
                }
            }
            if vec.is_empty() {
                self.multimap.remove(&key);
            }
        }
    }

    pub fn clean(&mut self) -> () {
        let ttl = self.ttl;
        self.multimap.retain(|_, v| v.is_entry_valid(ttl))
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;
    use std::thread;

    #[test]
    fn insert_and_get() -> () {
        let mut reg = Registry::create(5, Duration::from_secs(300));
        reg.insert(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            "Test".parse().unwrap(),
        );
        assert_eq!(1, reg.multimap.len());
        let (vec, dirty) = reg.get(&IpAddr::V4(Ipv4Addr::LOCALHOST)).unwrap();
        assert_eq!(1, vec.len());
        assert!(!dirty);
        assert_eq!(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            vec[0].address
        )
    }

    #[test]
    fn test_ttl() -> () {
        let mut reg = Registry::create(5, Duration::from_secs(4));
        reg.insert(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            "Test".parse().unwrap(),
        );
        thread::sleep(Duration::from_secs(2));
        reg.insert(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
            "Test".parse().unwrap(),
        );
        thread::sleep(Duration::from_secs(2));
        let (vec, dirty) = reg.get(&IpAddr::V4(Ipv4Addr::LOCALHOST)).unwrap();
        assert_eq!(1, vec.len());
        assert!(dirty);
        reg.clean_key(&IpAddr::V4(Ipv4Addr::LOCALHOST));
        let (vec, dirty) = reg.get(&IpAddr::V4(Ipv4Addr::LOCALHOST)).unwrap();
        assert_eq!(1, vec.len());
        assert!(!dirty);
        thread::sleep(Duration::from_secs(2));
        reg.clean_key(&IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(None, reg.get(&IpAddr::V4(Ipv4Addr::LOCALHOST)))
    }
}
