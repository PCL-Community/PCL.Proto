//! Maven coordinate parsing, mirroring `PCL.Core.Minecraft.MavenArtifact`.

use std::{
    fmt,
    path::{Path, PathBuf},
};

/// A parsed Maven package identifier.
///
/// Supports the three forms used by Minecraft metadata:
/// `group:artifact:version`, `group:artifact:packaging:version`, and
/// `group:artifact:packaging:classifier:version`. A trailing `@ext` overrides
/// the default `jar` packaging, as vanilla library names allow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MavenArtifact {
    pub group: String,
    pub artifact: String,
    pub packaging: String,
    pub classifier: Option<String>,
    pub version: String,
}

#[allow(dead_code)]
impl MavenArtifact {
    pub fn parse(value: &str) -> Result<Self, MavenArtifactError> {
        let (coordinate, extension) = match value.split_once('@') {
            Some((coordinate, extension)) if !extension.is_empty() => (coordinate, Some(extension)),
            _ => (value, None),
        };
        let parts: Vec<&str> = coordinate.split(':').collect();
        if !(3..=5).contains(&parts.len()) {
            return Err(MavenArtifactError::InvalidLength(parts.len()));
        }
        if parts.iter().any(|part| part.is_empty()) {
            return Err(MavenArtifactError::EmptySegment(value.into()));
        }
        let (group, artifact) = (parts[0], parts[1]);
        let (packaging, classifier, version) = match parts.len() {
            3 => (extension.unwrap_or("jar"), None, parts[2]),
            4 => {
                if is_common_packaging(parts[2]) {
                    // group:artifact:packaging:version
                    (extension.unwrap_or(parts[2]), None, parts[3])
                } else {
                    // group:artifact:version:classifier
                    ("jar", Some(parts[3]), parts[2])
                }
            }
            5 => (parts[2], Some(parts[3]), parts[4]),
            _ => return Err(MavenArtifactError::InvalidLength(parts.len())),
        };
        Ok(Self {
            group: group.into(),
            artifact: artifact.into(),
            packaging: extension.unwrap_or(packaging).into(),
            classifier: classifier.map(str::to_owned),
            version: version.into(),
        })
    }

    /// Relative artifact path, e.g. `org/example/lib/1.0/lib-1.0.jar`.
    pub fn relative_path(&self) -> PathBuf {
        let mut path = PathBuf::new();
        for segment in self.group.split('.') {
            path.push(segment);
        }
        path.push(&self.artifact);
        path.push(&self.version);
        let classifier = self
            .classifier
            .as_deref()
            .map(|classifier| format!("-{classifier}"))
            .unwrap_or_default();
        path.push(format!(
            "{}-{}{}.{}",
            self.artifact, self.version, classifier, self.packaging
        ));
        path
    }

    /// Slash-separated artifact path for URL/`safe_join` use on every OS.
    pub fn relative_slash_path(&self) -> String {
        let classifier = self
            .classifier
            .as_deref()
            .map(|classifier| format!("-{classifier}"))
            .unwrap_or_default();
        format!(
            "{}/{}/{}/{}-{}{}.{}",
            self.group.replace('.', "/"),
            self.artifact,
            self.version,
            self.artifact,
            self.version,
            classifier,
            self.packaging
        )
    }

    /// Resolve the artifact below a repository root.
    pub fn resolve(&self, repository: &Path) -> PathBuf {
        repository.join(self.relative_path())
    }

    /// Key used to de-duplicate libraries that only differ in repository.
    pub fn key(&self) -> String {
        format!(
            "{}:{}:{}:{}@{}",
            self.group,
            self.artifact,
            self.classifier.as_deref().unwrap_or(""),
            self.version,
            self.packaging
        )
    }
}

impl fmt::Display for MavenArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.group, self.artifact, self.version)?;
        if let Some(classifier) = &self.classifier {
            write!(f, ":{classifier}")?;
        }
        if self.packaging != "jar" {
            write!(f, "@{}", self.packaging)?;
        }
        Ok(())
    }
}

fn is_common_packaging(name: &str) -> bool {
    matches!(name, "jar" | "zip" | "pom")
}

#[derive(Debug, thiserror::Error)]
pub enum MavenArtifactError {
    #[error("Invalid Maven package id: {0}")]
    EmptySegment(String),
    #[error("Invalid Maven package id: expected 3 to 5 segments, got {0}")]
    InvalidLength(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_supported_coordinate_forms() {
        let simple = MavenArtifact::parse("org.example:lib:1.0").unwrap();
        assert_eq!(
            simple.relative_path(),
            PathBuf::from("org/example/lib/1.0/lib-1.0.jar")
        );
        assert_eq!(
            MavenArtifact::parse("org.example:lib:zip:1.0")
                .unwrap()
                .relative_path(),
            PathBuf::from("org/example/lib/1.0/lib-1.0.zip")
        );
        assert_eq!(
            MavenArtifact::parse("org.example:lib:1.0:natives-osx")
                .unwrap()
                .relative_path(),
            PathBuf::from("org/example/lib/1.0/lib-1.0-natives-osx.jar")
        );
        assert_eq!(
            MavenArtifact::parse("org.example:lib:1.0:natives-osx")
                .unwrap()
                .relative_slash_path(),
            "org/example/lib/1.0/lib-1.0-natives-osx.jar"
        );
        assert_eq!(
            MavenArtifact::parse("org.example:lib:jar:classifier:1.0")
                .unwrap()
                .relative_path(),
            PathBuf::from("org/example/lib/1.0/lib-1.0-classifier.jar")
        );
        assert_eq!(
            MavenArtifact::parse("org.example:lib:1.0@zip")
                .unwrap()
                .relative_path(),
            PathBuf::from("org/example/lib/1.0/lib-1.0.zip")
        );
    }

    #[test]
    fn rejects_malformed_coordinates() {
        for value in ["", "a", "a:b", "a:b:c:d:e:f", "a::c", "a:b:"] {
            assert!(MavenArtifact::parse(value).is_err(), "{value}");
        }
    }
}
