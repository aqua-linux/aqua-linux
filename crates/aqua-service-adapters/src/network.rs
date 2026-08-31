use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io;
use std::net::IpAddr;
use std::path::Path;

pub const MAX_NETWORK_INTERFACES: usize = 8;
pub const MAX_DNS_SERVERS: usize = 3;
const MAX_INTERFACE_DIRECTORY_ENTRIES: usize = 32;
const MAX_ROUTE_BYTES: u64 = 64 * 1024;
const MAX_RESOLVER_BYTES: u64 = 4 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkServiceHealth {
    Unavailable,
    Offline,
    Configuring,
    Online,
    Degraded,
}

impl NetworkServiceHealth {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::Offline => "offline",
            Self::Configuring => "configuring",
            Self::Online => "online",
            Self::Degraded => "degraded",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkInterfaceKind {
    Wired,
    Wireless,
    Other,
}

impl NetworkInterfaceKind {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Wired => "wired",
            Self::Wireless => "wireless",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkLinkState {
    Up,
    Down,
    Dormant,
    LowerLayerDown,
    NotPresent,
    Testing,
    Unknown,
}

impl NetworkLinkState {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Dormant => "dormant",
            Self::LowerLayerDown => "lowerlayerdown",
            Self::NotPresent => "notpresent",
            Self::Testing => "testing",
            Self::Unknown => "unknown",
        }
    }

    fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "up" => Self::Up,
            "down" => Self::Down,
            "dormant" => Self::Dormant,
            "lowerlayerdown" => Self::LowerLayerDown,
            "notpresent" => Self::NotPresent,
            "testing" => Self::Testing,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInterface {
    name: String,
    kind: NetworkInterfaceKind,
    link: NetworkLinkState,
}

impl NetworkInterface {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn kind(&self) -> NetworkInterfaceKind {
        self.kind
    }

    pub const fn link(&self) -> NetworkLinkState {
        self.link
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkAuthoritativeState {
    health: NetworkServiceHealth,
    interfaces: Vec<NetworkInterface>,
    default_route: Option<String>,
    dns_servers: Vec<IpAddr>,
}

impl Default for NetworkAuthoritativeState {
    fn default() -> Self {
        Self::unavailable()
    }
}

impl NetworkAuthoritativeState {
    pub fn unavailable() -> Self {
        Self {
            health: NetworkServiceHealth::Unavailable,
            interfaces: Vec::new(),
            default_route: None,
            dns_servers: Vec::new(),
        }
    }

    pub const fn health(&self) -> NetworkServiceHealth {
        self.health
    }

    pub fn interfaces(&self) -> &[NetworkInterface] {
        &self.interfaces
    }

    pub fn default_route(&self) -> Option<&str> {
        self.default_route.as_deref()
    }

    pub fn dns_servers(&self) -> &[IpAddr] {
        &self.dns_servers
    }

    pub fn primary_interface(&self) -> Option<&NetworkInterface> {
        self.default_route
            .as_deref()
            .and_then(|route| {
                self.interfaces
                    .iter()
                    .find(|interface| interface.name == route)
            })
            .or_else(|| {
                self.interfaces
                    .iter()
                    .find(|interface| interface.link == NetworkLinkState::Up)
            })
            .or_else(|| self.interfaces.first())
    }

    pub const fn status_available(&self) -> bool {
        !matches!(self.health, NetworkServiceHealth::Unavailable)
    }
}

#[derive(Debug)]
pub enum NetworkSnapshotError {
    Interfaces(io::Error),
}

impl fmt::Display for NetworkSnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Interfaces(error) => {
                write!(formatter, "network interface snapshot failed: {error}")
            }
        }
    }
}

impl std::error::Error for NetworkSnapshotError {}

pub fn read_network_snapshot(
    class_net: &Path,
    ipv4_route: &Path,
    resolver: &Path,
) -> Result<NetworkAuthoritativeState, NetworkSnapshotError> {
    let interfaces =
        read_network_interfaces(class_net).map_err(NetworkSnapshotError::Interfaces)?;
    if !interfaces
        .iter()
        .any(|interface| interface.link == NetworkLinkState::Up)
    {
        return Ok(NetworkAuthoritativeState {
            health: NetworkServiceHealth::Offline,
            interfaces,
            default_route: None,
            dns_servers: Vec::new(),
        });
    }

    let default_route = match read_default_ipv4_route(ipv4_route) {
        Ok(Some(default_route)) => default_route,
        Ok(None) => {
            return Ok(NetworkAuthoritativeState {
                health: NetworkServiceHealth::Configuring,
                interfaces,
                default_route: None,
                dns_servers: Vec::new(),
            });
        }
        Err(_) => {
            return Ok(NetworkAuthoritativeState {
                health: NetworkServiceHealth::Degraded,
                interfaces,
                default_route: None,
                dns_servers: Vec::new(),
            });
        }
    };
    let route_is_up = interfaces
        .iter()
        .any(|interface| interface.name == default_route && interface.link == NetworkLinkState::Up);
    if !route_is_up {
        return Ok(NetworkAuthoritativeState {
            health: NetworkServiceHealth::Degraded,
            interfaces,
            default_route: Some(default_route),
            dns_servers: Vec::new(),
        });
    }

    let dns_servers = read_dns_servers(resolver).unwrap_or_default();
    let health = if dns_servers.is_empty() {
        NetworkServiceHealth::Degraded
    } else {
        NetworkServiceHealth::Online
    };
    Ok(NetworkAuthoritativeState {
        health,
        interfaces,
        default_route: Some(default_route),
        dns_servers,
    })
}

pub fn read_network_interfaces(class_net: &Path) -> io::Result<Vec<NetworkInterface>> {
    let mut interfaces = Vec::new();
    for entry in fs::read_dir(class_net)?.take(MAX_INTERFACE_DIRECTORY_ENTRIES) {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name == "lo" || !valid_interface_name(&name) {
            continue;
        }
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let link = fs::read_to_string(path.join("operstate"))
            .map(|state| NetworkLinkState::parse(&state))
            .unwrap_or(NetworkLinkState::Unknown);
        let kind = if path.join("wireless").is_dir() {
            NetworkInterfaceKind::Wireless
        } else if path.join("device").exists() {
            NetworkInterfaceKind::Wired
        } else {
            NetworkInterfaceKind::Other
        };
        interfaces.push(NetworkInterface { name, kind, link });
        if interfaces.len() == MAX_NETWORK_INTERFACES {
            break;
        }
    }
    interfaces.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(interfaces)
}

fn valid_interface_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 15
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn read_default_ipv4_route(path: &Path) -> io::Result<Option<String>> {
    let contents = read_bounded_text(path, MAX_ROUTE_BYTES)?;
    for line in contents.lines().skip(1) {
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        if fields.len() < 4 || fields[1] != "00000000" {
            continue;
        }
        let Ok(flags) = u16::from_str_radix(fields[3], 16) else {
            continue;
        };
        if flags & 1 != 0 && valid_interface_name(fields[0]) {
            return Ok(Some(fields[0].to_string()));
        }
    }
    Ok(None)
}

fn read_dns_servers(path: &Path) -> io::Result<Vec<IpAddr>> {
    let contents = read_bounded_text(path, MAX_RESOLVER_BYTES)?;
    let mut servers = Vec::new();
    let mut seen = HashSet::new();
    for line in contents.lines() {
        let mut fields = line.split_ascii_whitespace();
        if fields.next() != Some("nameserver") {
            continue;
        }
        let Some(address) = fields.next().and_then(|value| value.parse::<IpAddr>().ok()) else {
            continue;
        };
        if seen.insert(address) {
            servers.push(address);
            if servers.len() == MAX_DNS_SERVERS {
                break;
            }
        }
    }
    Ok(servers)
}

fn read_bounded_text(path: &Path, limit: u64) -> io::Result<String> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "network status source exceeds size limit",
        ));
    }
    fs::read_to_string(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(1);

    struct Fixture {
        root: std::path::PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "aqua-network-adapter-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("clock")
                    .as_nanos(),
                NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(root.join("class-net/lo")).expect("loopback fixture");
            fs::write(root.join("class-net/lo/operstate"), "unknown\n").expect("loopback state");
            Self { root }
        }

        fn interface(&self, name: &str, state: &str, wireless: bool) {
            let path = self.root.join("class-net").join(name);
            fs::create_dir_all(&path).expect("interface fixture");
            fs::write(path.join("operstate"), format!("{state}\n")).expect("interface state");
            if wireless {
                fs::create_dir(path.join("wireless")).expect("wireless marker");
            } else {
                fs::create_dir(path.join("device")).expect("device marker");
            }
        }

        fn snapshot(&self) -> Result<NetworkAuthoritativeState, NetworkSnapshotError> {
            read_network_snapshot(
                &self.root.join("class-net"),
                &self.root.join("route"),
                &self.root.join("resolv.conf"),
            )
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).expect("remove network fixture");
        }
    }

    #[test]
    fn online_requires_up_default_route_and_valid_dns() {
        let fixture = Fixture::new();
        fixture.interface("eth0", "up", false);
        fixture.interface("wlan0", "dormant", true);
        fs::write(
            fixture.root.join("route"),
            "Iface Destination Gateway Flags RefCnt Use Metric Mask MTU Window IRTT\neth0 00000000 0100000A 0003 0 0 100 00000000 0 0 0\n",
        )
        .expect("route fixture");
        fs::write(
            fixture.root.join("resolv.conf"),
            "nameserver 1.1.1.1\nnameserver 2001:4860:4860::8888\nnameserver invalid\nnameserver 1.1.1.1\n",
        )
        .expect("resolver fixture");

        let state = fixture.snapshot().expect("network snapshot");
        assert_eq!(state.health(), NetworkServiceHealth::Online);
        assert_eq!(state.default_route(), Some("eth0"));
        assert_eq!(state.dns_servers().len(), 2);
        assert_eq!(
            state.primary_interface().map(NetworkInterface::name),
            Some("eth0")
        );
        assert_eq!(state.interfaces()[0].kind(), NetworkInterfaceKind::Wired);
        assert_eq!(state.interfaces()[1].kind(), NetworkInterfaceKind::Wireless);
    }

    #[test]
    fn link_without_route_is_configuring() {
        let fixture = Fixture::new();
        fixture.interface("eth0", "up", false);
        fs::write(
            fixture.root.join("route"),
            "Iface Destination Gateway Flags\n",
        )
        .expect("route fixture");

        let state = fixture.snapshot().expect("network snapshot");
        assert_eq!(state.health(), NetworkServiceHealth::Configuring);
        assert!(state.default_route().is_none());
        assert!(state.dns_servers().is_empty());
    }

    #[test]
    fn route_without_dns_is_degraded() {
        let fixture = Fixture::new();
        fixture.interface("eth0", "up", false);
        fs::write(
            fixture.root.join("route"),
            "Iface Destination Gateway Flags\neth0 00000000 0100000A 0003\n",
        )
        .expect("route fixture");
        fs::write(fixture.root.join("resolv.conf"), "nameserver invalid\n")
            .expect("resolver fixture");

        let state = fixture.snapshot().expect("network snapshot");
        assert_eq!(state.health(), NetworkServiceHealth::Degraded);
        assert_eq!(state.default_route(), Some("eth0"));
        assert!(state.dns_servers().is_empty());
    }

    #[test]
    fn down_links_are_offline_without_route_or_resolver_inputs() {
        let fixture = Fixture::new();
        fixture.interface("eth0", "down", false);

        let state = fixture.snapshot().expect("network snapshot");
        assert_eq!(state.health(), NetworkServiceHealth::Offline);
        assert!(state.status_available());
    }

    #[test]
    fn interface_source_failure_is_unavailable_to_the_caller() {
        let fixture = Fixture::new();
        let error = read_network_snapshot(
            &fixture.root.join("missing"),
            &fixture.root.join("route"),
            &fixture.root.join("resolv.conf"),
        )
        .expect_err("missing sysfs source must fail");
        assert!(matches!(error, NetworkSnapshotError::Interfaces(_)));
    }
}
