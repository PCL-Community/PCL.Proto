//! Bounded Range workers with in-session resume and recovery of slow tail segments.
use super::*;
use futures_util::{StreamExt, stream::FuturesUnordered};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Mutex, atomic::AtomicU64},
    time::Instant,
};
use tokio::io::AsyncSeekExt;

const TARGET_SEGMENT_SIZE: u64 = 8 * 1024 * 1024;
const MAX_SEGMENTS: usize = 1024;
#[derive(Debug)]
struct Segment {
    id: usize,
    start: u64,
    end: u64,
    offset: u64,
    failures: u8,
}
impl Segment {
    fn remaining(&self) -> u64 {
        self.end + 1 - self.offset
    }
}
#[derive(Default)]
struct Rates {
    active: HashMap<usize, (f64, Instant)>,
    completed: VecDeque<f64>,
}
impl Rates {
    fn is_slow(&self, id: usize, own: f64) -> bool {
        let mut peers: Vec<_> = self
            .active
            .iter()
            .filter(|(key, (_, at))| **key != id && at.elapsed() <= Duration::from_secs(2))
            .map(|(_, (rate, _))| *rate)
            .filter(|rate| *rate > 0.0)
            .collect();
        if peers.is_empty() {
            peers.extend(self.completed.iter().copied());
        }
        peers.sort_by(f64::total_cmp);
        let Some(fastest) = peers.last() else {
            return false;
        };
        let median = peers[peers.len() / 2];
        (median >= 128.0 * 1024.0 && own * 4.0 < median)
            || (own < 50.0 * 1024.0 && *fastest >= 128.0 * 1024.0 && own * 4.0 < *fastest)
    }
}
struct RateLease<'a> {
    id: usize,
    rates: &'a Mutex<Rates>,
}
impl Drop for RateLease<'_> {
    fn drop(&mut self) {
        self.rates.lock().unwrap().active.remove(&self.id);
    }
}

impl Downloader {
    pub(super) async fn probe_range(
        &self,
        target: &RequestTarget,
        expected: Option<u64>,
    ) -> Result<Option<u64>, McApiError> {
        // A one-byte probe avoids opening several full responses on non-Range servers.
        let _lease = self.resources().acquire().await;
        let response = self
            .transport
            .send_request(
                self.resources().settings().http_mode,
                true,
                target,
                Some((0, 0)),
                None,
            )
            .await?;
        if matches!(
            response.status(),
            reqwest::StatusCode::OK | reqwest::StatusCode::RANGE_NOT_SATISFIABLE
        ) {
            return Ok(None);
        }
        let response = response.error_for_status()?;
        let total = response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("bytes 0-0/"))
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|size| *size > 0);
        if response.status() != reqwest::StatusCode::PARTIAL_CONTENT
            || total.is_none_or(|size| expected.is_some_and(|expected| size != expected))
            || response.content_length().is_some_and(|length| length != 1)
            || response
                .headers()
                .get(reqwest::header::CONTENT_ENCODING)
                .is_some_and(|v| v != "identity")
        {
            return Ok(None);
        }
        let mut stream = response.bytes_stream();
        let mut count = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            self.resources().record_bytes(chunk.len());
            count += chunk.len();
            if count > 1 {
                return Ok(None);
            }
            self.resources().throttle(chunk.len()).await;
        }
        let Some(total) = total else {
            return Ok(None);
        };
        Ok((count == 1).then_some(total))
    }

    pub(super) async fn download_ranges(
        &self,
        info: &DownloadFile,
        path: &Path,
        target: &RequestTarget,
        progress: ProgressSink<'_>,
    ) -> Result<(), McApiError> {
        let output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        output.set_len(info.size)?;
        drop(output);
        let count = info
            .size
            .div_ceil(TARGET_SEGMENT_SIZE)
            .clamp(1, MAX_SEGMENTS as u64) as usize;
        let size = info.size.div_ceil(count as u64);
        let mut pending: VecDeque<_> = (0..count)
            .map(|id| {
                let start = id as u64 * size;
                Segment {
                    id,
                    start,
                    offset: start,
                    end: (start + size).min(info.size) - 1,
                    failures: 0,
                }
            })
            .collect();
        let mut next_id = count;
        let downloaded = AtomicU64::new(0);
        let rates = Mutex::new(Rates::default());
        let mut workers = FuturesUnordered::new();
        loop {
            let limit = self.resources().settings().file_connections;
            while workers.len() < limit {
                let Some(segment) = pending.pop_front() else {
                    break;
                };
                workers.push(self.run_segment(
                    segment,
                    info.size,
                    path,
                    target,
                    &downloaded,
                    &rates,
                    progress,
                ));
            }
            let Some((mut segment, result)) = workers.next().await else {
                break;
            };
            if let Err(error) = result {
                if matches!(error, McApiError::RangeUnsupported | McApiError::IoError(_))
                    || permanent_http_error(&error)
                    || segment.failures >= 2
                {
                    return Err(error);
                }
                if segment.remaining() == 0 {
                    return Err(error);
                }
                segment.failures += 1;
                // Preserve already written bytes and divide only the unfinished interval.
                if segment.remaining() >= 2 * 1024 * 1024 && next_id < MAX_SEGMENTS * 2 {
                    let start = segment.offset + segment.remaining() / 2;
                    pending.push_back(Segment {
                        id: next_id,
                        start,
                        offset: start,
                        end: segment.end,
                        failures: segment.failures,
                    });
                    next_id += 1;
                    segment.end = start - 1;
                }
                log::debug!(
                    "Recovering range {}..={} from {} after {error}",
                    segment.start,
                    segment.end,
                    segment.offset
                );
                pending.push_back(segment);
            }
        }
        if downloaded.load(Ordering::Relaxed) != info.size {
            return Err(McApiError::InvalidMetadata(
                "Incomplete Range download".into(),
            ));
        }
        report(progress, info.size, TaskStatus::Running).await;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_segment(
        &self,
        mut segment: Segment,
        total: u64,
        path: &Path,
        target: &RequestTarget,
        downloaded: &AtomicU64,
        rates: &Mutex<Rates>,
        progress: ProgressSink<'_>,
    ) -> (Segment, Result<(), McApiError>) {
        let _rate_lease = RateLease {
            id: segment.id,
            rates,
        };
        let result = async {
            let _lease = self.resources().acquire().await;
            let response = self
                .transport
                .send_request(
                    self.resources().settings().http_mode,
                    true,
                    target,
                    Some((segment.offset, segment.end)),
                    None,
                )
                .await?;
            if matches!(
                response.status(),
                reqwest::StatusCode::OK | reqwest::StatusCode::RANGE_NOT_SATISFIABLE
            ) {
                return Err(McApiError::RangeUnsupported);
            }
            let response = response.error_for_status()?;
            let expected = format!("bytes {}-{}/{total}", segment.offset, segment.end);
            if response.status() != reqwest::StatusCode::PARTIAL_CONTENT
                || response
                    .headers()
                    .get(reqwest::header::CONTENT_RANGE)
                    .and_then(|v| v.to_str().ok())
                    != Some(&expected)
                || response
                    .content_length()
                    .is_some_and(|length| length != segment.remaining())
                || response
                    .headers()
                    .get(reqwest::header::CONTENT_ENCODING)
                    .is_some_and(|v| v != "identity")
            {
                return Err(McApiError::RangeUnsupported);
            }
            let mut output =
                tokio::fs::File::from_std(std::fs::OpenOptions::new().write(true).open(path)?);
            output
                .seek(std::io::SeekFrom::Start(segment.offset))
                .await?;
            let mut output = BufWriter::with_capacity(64 * 1024, output);
            let started = Instant::now();
            let mut received = 0_u64;
            let mut last_sample = Instant::now();
            let mut last_report = Instant::now();
            let result = async {
                let mut stream = response.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    self.resources().record_bytes(chunk.len());
                    if chunk.len() as u64 > segment.remaining() {
                        return Err(McApiError::RangeUnsupported);
                    }
                    self.resources().throttle(chunk.len()).await;
                    output.write_all(&chunk).await?;
                    segment.offset += chunk.len() as u64;
                    received += chunk.len() as u64;
                    let bytes = downloaded.fetch_add(chunk.len() as u64, Ordering::Relaxed)
                        + chunk.len() as u64;
                    if last_report.elapsed() >= Duration::from_millis(100) {
                        report(progress, bytes, TaskStatus::Running).await;
                        last_report = Instant::now();
                    }
                    if last_sample.elapsed() >= Duration::from_millis(500) {
                        let own = received as f64 / started.elapsed().as_secs_f64().max(0.001);
                        let mut rates = rates.lock().unwrap();
                        rates.active.insert(segment.id, (own, Instant::now()));
                        if self.resources().settings().speed_limit == 0
                            && started.elapsed() >= Duration::from_secs(8)
                            && segment.remaining() >= 1024 * 1024
                            && rates.is_slow(segment.id, own)
                        {
                            return Err(McApiError::SlowSegment);
                        }
                        last_sample = Instant::now();
                    }
                }
                if segment.remaining() != 0 {
                    return Err(McApiError::InvalidMetadata(
                        "Range stream ended early".into(),
                    ));
                }
                Ok(())
            }
            .await;
            output.flush().await?;
            if result.is_ok() {
                let mut rates = rates.lock().unwrap();
                rates
                    .completed
                    .push_back(received as f64 / started.elapsed().as_secs_f64().max(0.001));
                while rates.completed.len() > 8 {
                    rates.completed.pop_front();
                }
            }
            result
        }
        .await;
        (segment, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slow_detection_requires_a_clearly_faster_peer() {
        let mut rates = Rates::default();
        assert!(!rates.is_slow(0, 1024.0));
        rates.active.insert(1, (10.0 * 1024.0, Instant::now()));
        assert!(!rates.is_slow(0, 1024.0));
        rates.active.insert(1, (512.0 * 1024.0, Instant::now()));
        assert!(rates.is_slow(0, 1024.0));
        assert!(!rates.is_slow(0, 400.0 * 1024.0));
        rates.active.clear();
        rates.completed.push_back(512.0 * 1024.0);
        assert!(rates.is_slow(0, 1024.0));
    }
}
