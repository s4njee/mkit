#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Downloading,
    Paused,
    Completed,
    Queued,
    Error,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Downloading => "Downloading",
            Self::Paused => "Paused",
            Self::Completed => "Completed",
            Self::Queued => "Queued",
            Self::Error => "Error",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Download {
    pub id: usize,
    pub name: String,
    pub host: String,
    pub size: String,
    pub percent: u8,
    pub speed: String,
    pub eta: String,
    pub status: Status,
    pub connections: u8,
    pub queue: String,
    pub save_to: String,
    pub split: u8,
    pub min_split_size: String,
}

impl Download {
    pub fn from_url(id: usize, url: &str) -> Self {
        let without_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
        let host = without_scheme.split('/').next().unwrap_or("Unknown host");
        let name =
            without_scheme.rsplit('/').next().filter(|name| !name.is_empty()).unwrap_or("download");
        Self {
            id,
            name: name.to_owned(),
            host: host.to_owned(),
            size: "Unknown".to_owned(),
            percent: 0,
            speed: "—".to_owned(),
            eta: "—".to_owned(),
            status: Status::Queued,
            connections: 16,
            queue: "main".into(),
            save_to: "~/Downloads".into(),
            split: 16,
            min_split_size: "10M".into(),
        }
    }
}

pub fn fixture_downloads() -> Vec<Download> {
    use Status::*;
    [
        (
            "ubuntu-24.04.2-desktop-amd64.iso",
            "releases.ubuntu.com",
            "6.10 GB",
            42,
            "11.4 MB/s",
            "07:12",
            Downloading,
            16,
        ),
        (
            "Blender-4.5.0-linux-x64.tar.xz",
            "mirror.clarkson.edu",
            "341 MB",
            78,
            "8.2 MB/s",
            "00:09",
            Downloading,
            16,
        ),
        (
            "project-backup-2026-09-01.zip",
            "s3.eu-central-1.amazonaws.com",
            "1.80 GB",
            63,
            "4.9 MB/s",
            "02:14",
            Downloading,
            8,
        ),
        ("imagenet-shard-03.tar", "data.vision.ee.ethz.ch", "12.4 GB", 9, "—", "—", Paused, 8),
        ("macos-recovery-image.dmg", "swcdn.apple.com", "4.20 GB", 0, "—", "—", Queued, 16),
        (
            "nvidia-driver-580.11.run",
            "us.download.nvidia.com",
            "412 MB",
            100,
            "—",
            "—",
            Completed,
            8,
        ),
        ("postgresql-17.2.tar.bz2", "ftp.postgresql.org", "28.4 MB", 100, "—", "—", Completed, 4),
        ("archive-photos-2019.tar.gz", "cdn.fileshare.io", "9.02 GB", 0, "—", "—", Error, 8),
    ]
    .into_iter()
    .enumerate()
    .map(|(id, (name, host, size, percent, speed, eta, status, connections))| Download {
        id,
        name: name.to_owned(),
        host: host.to_owned(),
        size: size.to_owned(),
        percent,
        speed: speed.to_owned(),
        eta: eta.to_owned(),
        status,
        connections,
        queue: "main".into(),
        save_to: "~/Downloads".into(),
        split: connections,
        min_split_size: "10M".into(),
    })
    .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    All,
    Downloading,
    Paused,
    Completed,
    Failed,
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All downloads",
            Self::Downloading => "Downloading",
            Self::Paused => "Paused",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
        }
    }
    pub fn matches(self, status: Status) -> bool {
        match self {
            Self::All => true,
            Self::Downloading => status == Status::Downloading,
            Self::Paused => status == Status::Paused,
            Self::Completed => status == Status::Completed,
            Self::Failed => status == Status::Error,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scene {
    Downloads,
    Detail,
    Queues,
    Settings,
    Capture,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_filter_and_new_download_are_deterministic() {
        let rows = fixture_downloads();
        assert_eq!(rows.len(), 8);
        assert_eq!(rows.iter().filter(|row| Category::Downloading.matches(row.status)).count(), 3);
        assert_eq!(rows.iter().filter(|row| Category::Completed.matches(row.status)).count(), 2);
        let added = Download::from_url(8, "https://example.org/files/demo.iso");
        assert_eq!(
            (added.name.as_str(), added.host.as_str(), added.status),
            ("demo.iso", "example.org", Status::Queued)
        );
    }
}
