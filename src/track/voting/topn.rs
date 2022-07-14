use crate::track::ObservationMetricResult;
use crate::voting::Voting;
use itertools::Itertools;

/// TopN winners voting engine that selects Top N vectors with most close distances.
///
/// It calculates winners as:
/// 1. removes all distances that are greater than threshold
/// 2. sorts remaining tracks according to their IDs
/// 3. counts tracks by their ID's
/// 4. sorts groups by frequency decreasingly
/// 5. returns TopN
///
pub struct TopNVoting {
    topn: usize,
    max_distance: f32,
    min_votes: usize,
}

impl TopNVoting {
    /// Constructs new engine
    ///
    /// # Arguments
    /// * `topn` - top winners
    /// * `max_distance` - max distance permitted to participate
    /// * `min_votes` - minimal amount of votes required the track to participate
    ///
    pub fn new(topn: usize, max_distance: f32, min_votes: usize) -> Self {
        Self {
            topn,
            max_distance,
            min_votes,
        }
    }
}

/// Return type fot TopN voting engine
///
#[derive(Default, Debug, PartialEq, Eq)]
pub struct TopNVotingElt {
    /// winning track
    pub track_id: u64,
    /// number of votes it gathered
    pub votes: usize,
}

impl TopNVotingElt {
    pub fn new(track_id: u64, votes: usize) -> Self {
        Self { track_id, votes }
    }
}

impl Voting<TopNVotingElt, f32> for TopNVoting {
    fn winners(&self, distances: &[ObservationMetricResult<f32>]) -> Vec<TopNVotingElt> {
        let mut tracks: Vec<_> = distances
            .iter()
            .filter(
                |ObservationMetricResult(_track, _f_attr_dist, feat_dist)| match feat_dist {
                    Some(e) => *e <= self.max_distance,
                    _ => false,
                },
            )
            .map(|ObservationMetricResult(track, _f_attr_dist, _feat_dist)| track)
            .collect();
        tracks.sort_unstable();
        let mut counts = tracks
            .into_iter()
            .counts()
            .into_iter()
            .filter(|(_, count)| *count >= self.min_votes)
            .map(|(e, c)| TopNVotingElt {
                track_id: *e,
                votes: c,
            })
            .collect::<Vec<_>>();
        counts.sort_by(|l, r| r.votes.partial_cmp(&l.votes).unwrap());
        counts.truncate(self.topn);
        counts
    }
}

#[cfg(test)]
mod tests {
    use crate::track::voting::topn::{TopNVoting, TopNVotingElt, Voting};
    use crate::track::ObservationMetricResult;

    #[test]
    fn default_voting() {
        let v = TopNVoting {
            topn: 5,
            max_distance: 0.32,
            min_votes: 1,
        };

        let candidates = v.winners(&vec![ObservationMetricResult(1, Some(0.0), Some(0.2))]);
        assert_eq!(candidates, vec![TopNVotingElt::new(1, 1)]);

        let candidates = v.winners(&vec![
            ObservationMetricResult(1, Some(0.0), Some(0.2)),
            ObservationMetricResult(1, Some(0.0), Some(0.3)),
        ]);
        assert_eq!(candidates, vec![TopNVotingElt::new(1, 2)]);

        let candidates = v.winners(&vec![
            ObservationMetricResult(1, Some(0.0), Some(0.2)),
            ObservationMetricResult(1, Some(0.0), Some(0.4)),
        ]);
        assert_eq!(candidates, vec![TopNVotingElt::new(1, 1)]);

        let mut candidates = v.winners(&vec![
            ObservationMetricResult(1, Some(0.0), Some(0.2)),
            ObservationMetricResult(2, Some(0.0), Some(0.2)),
        ]);
        candidates.sort_by(|l, r| l.track_id.partial_cmp(&r.track_id).unwrap());
        assert_eq!(
            candidates,
            vec![TopNVotingElt::new(1, 1), TopNVotingElt::new(2, 1)]
        );

        let mut candidates = v.winners(&vec![
            ObservationMetricResult(1, Some(0.0), Some(0.2)),
            ObservationMetricResult(1, Some(0.0), Some(0.22)),
            ObservationMetricResult(2, Some(0.0), Some(0.21)),
            ObservationMetricResult(2, Some(0.0), Some(0.2)),
            ObservationMetricResult(3, Some(0.0), Some(0.22)),
            ObservationMetricResult(3, Some(0.0), Some(0.2)),
            ObservationMetricResult(4, Some(0.0), Some(0.23)),
            ObservationMetricResult(4, Some(0.0), Some(0.3)),
            ObservationMetricResult(5, Some(0.0), Some(0.24)),
            ObservationMetricResult(5, Some(0.0), Some(0.3)),
            ObservationMetricResult(6, Some(0.0), Some(0.25)),
            ObservationMetricResult(6, Some(0.0), Some(0.5)),
        ]);
        candidates.sort_by(|l, r| l.track_id.partial_cmp(&r.track_id).unwrap());
        assert_eq!(
            candidates,
            vec![
                TopNVotingElt::new(1, 2),
                TopNVotingElt::new(2, 2),
                TopNVotingElt::new(3, 2),
                TopNVotingElt::new(4, 2),
                TopNVotingElt::new(5, 2)
            ]
        );
    }
}
