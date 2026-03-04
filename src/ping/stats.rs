use super::PingReply;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PingStats {
    pub host: String,
    pub ip: String,
    pub sent: u32,
    pub received: u32,
    pub lost: u32,
    pub min_ms: f64,
    pub max_ms: f64,
    pub avg_ms: f64,
    pub stddev_ms: f64,
    pub jitter_ms: f64,
}

impl PingStats {
    pub fn from_results(host: &str, ip: &str, replies: &[PingReply]) -> Self {
        let rtts: Vec<f64> = replies
            .iter()
            .filter_map(|r| match r {
                PingReply::Ok { rtt_ms } => Some(*rtt_ms),
                _ => None,
            })
            .collect();

        let sent = replies.len() as u32;
        let received = rtts.len() as u32;
        let lost = sent - received;

        if rtts.is_empty() {
            return Self {
                host: host.to_string(),
                ip: ip.to_string(),
                sent,
                received,
                lost,
                min_ms: 0.0,
                max_ms: 0.0,
                avg_ms: 0.0,
                stddev_ms: 0.0,
                jitter_ms: 0.0,
            };
        }

        let min_ms = rtts.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_ms = rtts.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let avg_ms = rtts.iter().sum::<f64>() / rtts.len() as f64;

        let variance = rtts.iter().map(|r| (r - avg_ms).powi(2)).sum::<f64>() / rtts.len() as f64;
        let stddev_ms = variance.sqrt();

        let jitter_ms = if rtts.len() > 1 {
            let diffs: Vec<f64> = rtts
                .windows(2)
                .map(|w| (w[1] - w[0]).abs())
                .collect();
            diffs.iter().sum::<f64>() / diffs.len() as f64
        } else {
            0.0
        };

        Self {
            host: host.to_string(),
            ip: ip.to_string(),
            sent,
            received,
            lost,
            min_ms,
            max_ms,
            avg_ms,
            stddev_ms,
            jitter_ms,
        }
    }

    pub fn loss_percent(&self) -> f64 {
        if self.sent == 0 {
            0.0
        } else {
            (self.lost as f64 / self.sent as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(ms: f64) -> PingReply {
        PingReply::Ok { rtt_ms: ms }
    }

    #[test]
    fn basic_stats() {
        let replies = vec![ok(10.0), ok(20.0), ok(30.0), ok(40.0)];
        let stats = PingStats::from_results("test", "1.2.3.4", &replies);
        assert_eq!(stats.sent, 4);
        assert_eq!(stats.received, 4);
        assert_eq!(stats.lost, 0);
        assert!((stats.min_ms - 10.0).abs() < 0.01);
        assert!((stats.max_ms - 40.0).abs() < 0.01);
        assert!((stats.avg_ms - 25.0).abs() < 0.01);
        assert!((stats.loss_percent()).abs() < 0.01);
    }

    #[test]
    fn with_packet_loss() {
        let replies = vec![ok(10.0), PingReply::Timeout, ok(30.0), PingReply::Timeout];
        let stats = PingStats::from_results("test", "1.2.3.4", &replies);
        assert_eq!(stats.sent, 4);
        assert_eq!(stats.received, 2);
        assert_eq!(stats.lost, 2);
        assert!((stats.loss_percent() - 50.0).abs() < 0.01);
    }

    #[test]
    fn all_timeouts() {
        let replies = vec![PingReply::Timeout, PingReply::Timeout];
        let stats = PingStats::from_results("test", "1.2.3.4", &replies);
        assert_eq!(stats.received, 0);
        assert!((stats.loss_percent() - 100.0).abs() < 0.01);
        assert_eq!(stats.min_ms, 0.0);
    }

    #[test]
    fn empty_results() {
        let replies: Vec<PingReply> = vec![];
        let stats = PingStats::from_results("test", "1.2.3.4", &replies);
        assert_eq!(stats.sent, 0);
        assert!((stats.loss_percent()).abs() < 0.01);
    }

    #[test]
    fn single_ping() {
        let replies = vec![ok(15.5)];
        let stats = PingStats::from_results("test", "1.2.3.4", &replies);
        assert_eq!(stats.received, 1);
        assert!((stats.min_ms - 15.5).abs() < 0.01);
        assert!((stats.max_ms - 15.5).abs() < 0.01);
        assert!((stats.avg_ms - 15.5).abs() < 0.01);
        assert!((stats.stddev_ms).abs() < 0.01);
        assert!((stats.jitter_ms).abs() < 0.01);
    }

    #[test]
    fn jitter_calculation() {
        let replies = vec![ok(10.0), ok(20.0), ok(15.0), ok(25.0)];
        let stats = PingStats::from_results("test", "1.2.3.4", &replies);
        // Diffs: |20-10|=10, |15-20|=5, |25-15|=10 → avg = 25/3 ≈ 8.33
        assert!((stats.jitter_ms - 8.333).abs() < 0.01);
    }

    #[test]
    fn stddev_calculation() {
        let replies = vec![ok(10.0), ok(10.0), ok(10.0)];
        let stats = PingStats::from_results("test", "1.2.3.4", &replies);
        assert!((stats.stddev_ms).abs() < 0.01); // All same → stddev = 0
    }
}
