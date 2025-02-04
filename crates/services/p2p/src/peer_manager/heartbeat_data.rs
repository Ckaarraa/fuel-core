use fuel_core_types::fuel_types::BlockHeight;
use std::{
    collections::VecDeque,
    time::Duration,
};
use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct HeartbeatData {
    pub block_height: Option<BlockHeight>,
    pub last_heartbeat: Instant,
    // Size of moving average window
    pub window: u32,
    pub durations: VecDeque<Duration>,
}

impl HeartbeatData {
    pub fn new(window: u32) -> Self {
        Self {
            block_height: None,
            last_heartbeat: Instant::now(),
            window,
            durations: VecDeque::with_capacity(window as usize),
        }
    }

    pub fn duration_since_last_heartbeat(&self) -> Duration {
        self.last_heartbeat.elapsed()
    }

    pub fn average_time_between_heartbeats(&self) -> Duration {
        if self.durations.is_empty() {
            Duration::from_secs(0)
        } else {
            self.durations.iter().sum::<Duration>() / self.durations.len() as u32
        }
    }

    fn add_new_duration(&mut self, new_duration: Duration) {
        if self.durations.len() == self.window as usize {
            self.durations.pop_back();
        }
        self.durations.push_front(new_duration);
    }

    pub fn update(&mut self, block_height: BlockHeight) {
        self.block_height = Some(block_height);
        let old_hearbeat = self.last_heartbeat;
        self.last_heartbeat = Instant::now();
        let new_duration = self.last_heartbeat - old_hearbeat;
        self.add_new_duration(new_duration);
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn duration_since_last_heartbeat__reads_correctly() {
        let heartbeat_data = HeartbeatData::new(10);
        tokio::time::advance(Duration::from_secs(10)).await;
        assert_eq!(
            heartbeat_data.duration_since_last_heartbeat(),
            Duration::from_secs(10)
        );
    }

    #[tokio::test(start_paused = true)]
    async fn update__works_with_many() {
        let intervals: Vec<u64> =
            vec![5, 40, 19, 400, 23, 36, 33, 22, 11, 10, 9, 8, 72, 16, 5, 4];
        let mut heartbeat_data = HeartbeatData::new(10);
        for (i, interval) in intervals.clone().into_iter().enumerate() {
            tokio::time::advance(Duration::from_secs(interval)).await;
            heartbeat_data.update(1.into());
            let bottom = if i < 10 { 0 } else { i - 9 };
            let range = &intervals[bottom..=i];
            let expected = range
                .iter()
                .map(|x| Duration::from_secs(*x))
                .sum::<Duration>()
                / range.len() as u32;
            let actual = heartbeat_data.average_time_between_heartbeats();
            assert_eq!(actual, expected);
        }
    }
}
